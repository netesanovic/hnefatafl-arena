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
use crate::plugin::PluginBot;

#[derive(Clone, Debug)]
enum BotType {
    Greedy,
    Random,
    Plugin(String), // Plugin path
}

#[derive(Clone)]
pub struct AppState {
    game: Arc<Mutex<WebGame>>,
}

struct WebGame {
    state: GameState,
    player_side: Player,
    bot_type: BotType,
    bot_instance: Option<Box<dyn Bot>>,
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
            bot_instance: None,
            game_over: false,
            winner: None,
        };
        AppState {
            game: Arc::new(Mutex::new(game)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
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
    if bot_type.starts_with("plugin:") {
        let path = bot_type.strip_prefix("plugin:").unwrap().to_string();
        BotType::Plugin(path)
    } else {
        match bot_type.to_lowercase().as_str() {
            "greedy" => BotType::Greedy,
            "random" => BotType::Random,
            _ => BotType::Greedy,
        }
    }
}

fn get_bot_instance(bot_type: &BotType) -> Result<Box<dyn Bot>, String> {
    match bot_type {
        BotType::Greedy => Ok(Box::new(GreedyBot::new("Greedy Bot".to_string()))),
        BotType::Random => Ok(Box::new(RandomBot::new("Random Bot".to_string()))),
        BotType::Plugin(path) => PluginBot::load(path).map(|bot| Box::new(bot) as Box<dyn Bot>),
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
        game.bot_type = bot_type.clone();
        game.game_over = false;
        game.winner = None;

        // Create and initialize the bot
        match get_bot_instance(&bot_type) {
            Ok(mut bot) => {
                // Initialize the bot with game_start
                let bot_side = player_side.opponent();
                bot.game_start(bot_side);

                // If bot goes first, make its move
                let message = if game.state.current_player() != player_side {
                    let state_clone = game.state.clone();
                    if let Some(bot_move) =
                        bot.get_move(&state_clone, std::time::Duration::from_secs(5))
                    {
                        let _ = game.state.make_move(bot_move);
                        bot.notify_move(bot_move);
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
                };

                game.bot_instance = Some(bot);
                message
            }
            Err(e) => {
                game.bot_instance = None;
                format!("Failed to load bot: {}", e)
            }
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
            // Notify bot of player's move
            if let Some(ref mut bot) = game.bot_instance {
                bot.notify_move(player_move);
            }

            let state_clone = game.state.clone();

            // Get the bot's move (separate borrow scope)
            let bot_move_opt = if let Some(ref mut bot) = game.bot_instance {
                bot.get_move(&state_clone, std::time::Duration::from_secs(5))
            } else {
                None
            };

            // Now apply the move and update game state
            if let Some(bot_move) = bot_move_opt {
                let _ = game.state.make_move(bot_move);

                // Notify bot of its own move
                if let Some(ref mut bot) = game.bot_instance {
                    bot.notify_move(bot_move);
                }

                if let Some(_result) = game.state.result() {
                    game.game_over = true;
                    game.winner = Some(game.state.current_player().opponent());
                }
                format!("Bot played: {} -> {}", bot_move.from, bot_move.to)
            } else if game.bot_instance.is_none() {
                "Bot instance not found".to_string()
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

    for (row_idx, row) in board.iter_mut().enumerate() {
        for (col_idx, field) in row.iter_mut().enumerate() {
            *field = piece_to_string(game.state.get_piece(Position::new(row_idx, col_idx)));
        }
    }

    let legal_moves: Vec<MoveResponse> =
        if !game.game_over && game.state.current_player() == game.player_side {
            game.state
                .legal_moves(game.state.current_player())
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

#[derive(Serialize)]
struct PluginInfo {
    id: String,
    name: String,
    path: String,
}

#[derive(Serialize)]
struct AvailableBotsResponse {
    built_in: Vec<String>,
    plugins: Vec<PluginInfo>,
}

async fn list_bots() -> Json<AvailableBotsResponse> {
    use std::fs;

    let mut plugins = Vec::new();

    // Scan for plugin libraries in the plugins directory
    if let Ok(entries) = fs::read_dir("plugins") {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let plugin_name = entry.file_name().to_string_lossy().to_string();

                // Check for both debug and release builds
                for build_type in ["debug", "release"] {
                    #[cfg(target_os = "linux")]
                    let lib_path = format!(
                        "plugins/{}/target/{}/lib{}.so",
                        plugin_name, build_type, plugin_name
                    );
                    #[cfg(target_os = "macos")]
                    let lib_path = format!(
                        "plugins/{}/target/{}/lib{}.dylib",
                        plugin_name, build_type, plugin_name
                    );
                    #[cfg(target_os = "windows")]
                    let lib_path = format!(
                        "plugins/{}\\target\\{}\\{}.dll",
                        plugin_name, build_type, plugin_name
                    );

                    if std::path::Path::new(&lib_path).exists() {
                        plugins.push(PluginInfo {
                            id: format!("plugin:{}", lib_path),
                            name: plugin_name.replace("_", " ").replace("-", " "),
                            path: lib_path,
                        });
                        break; // Found one, don't check other build types
                    }
                }
            }
        }
    }

    Json(AvailableBotsResponse {
        built_in: vec!["Greedy".to_string(), "Random".to_string()],
        plugins,
    })
}

pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let app_state = AppState::new();

    let app = Router::new()
        .route("/api/new-game", post(new_game))
        .route("/api/move", post(make_move))
        .route("/api/game-state", get(get_game_state))
        .route("/api/bots", get(list_bots))
        .nest_service("/", ServeDir::new("static"))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("🌐 Web server running at http://127.0.0.1:3000");
    println!("   Open your browser and start playing!");

    axum::serve(listener, app).await?;
    Ok(())
}
