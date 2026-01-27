use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

use crate::bot::{Bot, GreedyBot, RandomBot};
use crate::game::{GameState, Move, Piece, Player, Position, Variant};

#[derive(Clone, Copy, Debug)]
enum BotType {
    Greedy,
    Random,
}

#[derive(Clone)]
pub struct AppState {
    game: Arc<Mutex<WebGame>>,
}

#[derive(Clone)]
struct WebGame {
    state: GameState,
    player_side: Player,
    bot_type: BotType,
    game_over: bool,
    winner: Option<Player>,
}

#[derive(Serialize, Deserialize)]
pub struct NewGameRequest {
    variant: String,
    player_side: String,
    bot_type: String,
}

#[derive(Serialize)]
pub struct GameResponse {
    board: Vec<Vec<String>>,
    current_player: String,
    legal_moves: Vec<MoveResponse>,
    game_over: bool,
    winner: Option<String>,
    variant: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct MoveRequest {
    from_row: usize,
    from_col: usize,
    to_row: usize,
    to_col: usize,
}

#[derive(Serialize, Clone)]
pub struct MoveResponse {
    from_row: usize,
    from_col: usize,
    to_row: usize,
    to_col: usize,
}

impl AppState {
    pub fn new() -> Self {
        let game = WebGame {
            state: GameState::new(Variant::Brandubh),
            player_side: Player::Defenders,
            bot_type: BotType::Greedy,
            game_over: false,
            winner: None,
        };
        AppState {
            game: Arc::new(Mutex::new(game)),
        }
    }
}

fn piece_to_string(piece: Option<Piece>) -> String {
    match piece {
        None => ".".to_string(),
        Some(Piece::King) => "K".to_string(),
        Some(Piece::Attacker) => "A".to_string(),
        Some(Piece::Defender) => "D".to_string(),
    }
}

fn player_to_string(player: Player) -> String {
    match player {
        Player::Attackers => "Attackers".to_string(),
        Player::Defenders => "Defenders".to_string(),
    }
}

fn string_to_player(s: &str) -> Player {
    match s.to_lowercase().as_str() {
        "attackers" => Player::Attackers,
        "defenders" => Player::Defenders,
        _ => Player::Defenders,
    }
}

fn string_to_variant(s: &str) -> Variant {
    match s.to_lowercase().as_str() {
        "copenhagen" => Variant::Copenhagen,
        "brandubh" => Variant::Brandubh,
        _ => Variant::Brandubh,
    }
}

fn create_bot(bot_type: &str) -> BotType {
    match bot_type.to_lowercase().as_str() {
        "greedy" => BotType::Greedy,
        "random" => BotType::Random,
        _ => BotType::Greedy,
    }
}

fn get_bot_instance(bot_type: BotType) -> Box<dyn Bot> {
    match bot_type {
        BotType::Greedy => Box::new(GreedyBot::new("Greedy Bot".to_string())),
        BotType::Random => Box::new(RandomBot::new("Random Bot".to_string())),
    }
}

#[axum::debug_handler]
async fn new_game(State(app_state): State<AppState>, Json(req): Json<NewGameRequest>) -> Response {
    let variant = string_to_variant(&req.variant);
    let player_side = string_to_player(&req.player_side);
    let bot_type = create_bot(&req.bot_type);

    let message = {
        let mut game = app_state.game.lock().unwrap();
        game.state = GameState::new(variant);
        game.player_side = player_side;
        game.bot_type = bot_type;
        game.game_over = false;
        game.winner = None;

        // If bot goes first, make its move
        if game.state.current_player() != player_side {
            let state_clone = game.state.clone();
            let mut bot = get_bot_instance(bot_type);
            if let Some(bot_move) = bot.get_move(&state_clone, std::time::Duration::from_secs(5)) {
                let _ = game.state.make_move(bot_move);
                if let Some(_result) = game.state.result() {
                    game.game_over = true;
                    game.winner = Some(game.state.current_player().opponent());
                }
                format!("Bot played: {} -> {}", bot_move.from, bot_move.to)
            } else {
                "Bot failed to make a move".to_string()
            }
        } else {
            "Your turn!".to_string()
        }
    }; // MutexGuard dropped here

    let Json(mut game_response) = get_game_state(State(app_state)).await;
    game_response.message = message;
    Json(game_response).into_response()
}

#[axum::debug_handler]
async fn make_move(State(app_state): State<AppState>, Json(req): Json<MoveRequest>) -> Response {
    let bot_message = {
        let mut game = app_state.game.lock().unwrap();

        if game.game_over {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Game is over"
                })),
            )
                .into_response();
        }

        // Check if it's the player's turn
        if game.state.current_player() != game.player_side {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Not your turn"
                })),
            )
                .into_response();
        }

        // Apply player's move
        let player_move = Move::new(
            Position::new(req.from_row, req.from_col),
            Position::new(req.to_row, req.to_col),
        );

        if let Err(e) = game.state.make_move(player_move) {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("Invalid move: {}", e)
                })),
            )
                .into_response();
        }

        // Check if game is over after player's move
        if let Some(_result) = game.state.result() {
            let winner = game.state.current_player().opponent();
            game.game_over = true;
            game.winner = Some(winner);
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "message": format!("{} wins!", player_to_string(winner))
                })),
            )
                .into_response();
        }

        // Bot's turn
        if !game.game_over {
            let state_clone = game.state.clone();
            let bot_type = game.bot_type;
            drop(game); // Drop before calling bot

            let mut bot = get_bot_instance(bot_type);
            if let Some(bot_move) = bot.get_move(&state_clone, std::time::Duration::from_secs(5)) {
                // Re-lock to apply bot move
                let mut game = app_state.game.lock().unwrap();
                let _ = game.state.make_move(bot_move);
                if let Some(_result) = game.state.result() {
                    game.game_over = true;
                    game.winner = Some(game.state.current_player().opponent());
                }
                format!("Bot played: {} -> {}", bot_move.from, bot_move.to)
            } else {
                "Bot failed to make a move".to_string()
            }
        } else {
            String::new()
        }
    }; // Guard dropped here

    let Json(mut game_response) = get_game_state(State(app_state)).await;
    game_response.message = bot_message;
    Json(game_response).into_response()
}

async fn get_game_state(State(app_state): State<AppState>) -> Json<GameResponse> {
    let game = app_state.game.lock().unwrap();

    let size = game.state.variant().board_size();
    let mut board = vec![vec![String::new(); size]; size];

    for row in 0..size {
        for col in 0..size {
            board[row][col] = piece_to_string(game.state.get_piece(Position::new(row, col)));
        }
    }

    let legal_moves: Vec<MoveResponse> =
        if !game.game_over && game.state.current_player() == game.player_side {
            game.state
                .legal_moves()
                .iter()
                .map(|m| MoveResponse {
                    from_row: m.from.row,
                    from_col: m.from.col,
                    to_row: m.to.row,
                    to_col: m.to.col,
                })
                .collect()
        } else {
            Vec::new()
        };

    Json(GameResponse {
        board,
        current_player: player_to_string(game.state.current_player()),
        legal_moves,
        game_over: game.game_over,
        winner: game.winner.map(player_to_string),
        variant: format!("{:?}", game.state.variant()),
        message: String::new(),
    })
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState::new();

    let app = Router::new()
        .route("/api/new-game", post(new_game))
        .route("/api/move", post(make_move))
        .route("/api/game-state", get(get_game_state))
        .nest_service("/", ServeDir::new("static"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("🌐 Web server running at http://127.0.0.1:3000");
    println!("   Open your browser and start playing!");

    axum::serve(listener, app).await?;
    Ok(())
}
