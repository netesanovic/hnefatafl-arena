use hnefatafl_arena::{Bot, GameState, Move, Player, Piece, Position};
use std::time::Duration;

/// A greedy bot that tries to capture pieces
pub struct GreedyBotPlugin {
    name: String,
    last_state: Option<GameState>,
}

impl Default for GreedyBotPlugin {
    fn default() -> Self {
        Self {
            name: "GreedyPlugin".to_string(),
            last_state: None,
        }
    }
}

impl GreedyBotPlugin {
    fn evaluate_move(&self, state: &GameState, mv: Move) -> i32 {
        let mut temp_state = state.clone();
        let _ = temp_state.make_move(mv);

        // Count pieces for each side
        let mut attacker_count = 0;
        let mut defender_count = 0;
        let mut king_alive = false;

        for row in 0..state.board_size() {
            for col in 0..state.board_size() {
                let pos = Position::new(row, col);
                match temp_state.get_piece(pos) {
                    Some(Piece::Attacker) => attacker_count += 1,
                    Some(Piece::Defender) => defender_count += 1,
                    Some(Piece::King) => king_alive = true,
                    None => {}
                }
            }
        }

        // Simple evaluation based on piece count
        match state.current_player() {
            Player::Attackers => {
                if !king_alive {
                    return 1000; // King captured is winning
                }
                attacker_count - defender_count * 2
            }
            Player::Defenders => {
                if !king_alive {
                    return -1000; // King lost is losing
                }
                defender_count * 2 - attacker_count
            }
        }
    }
}

impl Bot for GreedyBotPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_move(&mut self, state: &GameState, _time_limit: Duration) -> Option<Move> {
        let moves = state.legal_moves();
        if moves.is_empty() {
            return None;
        }

        // Find the move with the best evaluation
        moves
            .into_iter()
            .max_by_key(|&mv| self.evaluate_move(state, mv))
    }

    fn game_start(&mut self, _player: Player) {
        self.last_state = None;
    }

    fn notify_move(&mut self, _mv: Move) {
        // Could update internal state here
    }
}

// Export the bot plugin using the macro
hnefatafl_arena::export_bot!(GreedyBotPlugin);
