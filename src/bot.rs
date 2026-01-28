use crate::game::{GameState, Move, Player};
use std::time::Duration;

/// Trait that all bots must implement
pub trait Bot: Send {
    /// Get the name of the bot
    fn name(&self) -> &str;

    /// Get the next move for the current game state
    /// The bot has a time limit to respond
    fn get_move(&mut self, state: &GameState, time_limit: Duration) -> Option<Move>;

    /// Notified when the game starts
    fn game_start(&mut self, _player: Player) {}

    /// Notified when a move is made (by either player)
    fn notify_move(&mut self, _mv: Move) {}

    /// Notified when the game ends
    fn game_end(&mut self) {}
}

/// A simple random bot for testing
pub struct RandomBot {
    name: String,
}

impl RandomBot {
    pub fn new(name: String) -> Self {
        RandomBot { name }
    }
}

impl Bot for RandomBot {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_move(&mut self, state: &GameState, _time_limit: Duration) -> Option<Move> {
        let moves = state.legal_moves();
        if moves.is_empty() {
            None
        } else {
            // In a real implementation, use rand crate
            // For now, just return the first move
            Some(moves[0])
        }
    }
}

/// A simple greedy bot that tries to capture pieces
pub struct GreedyBot {
    name: String,
}

impl GreedyBot {
    pub fn new(name: String) -> Self {
        GreedyBot { name }
    }

    fn evaluate_move(&self, state: &GameState, mv: Move) -> i32 {
        let mut temp_state = state.clone();
        let _ = temp_state.make_move(mv);

        // Count pieces for each side
        let mut attacker_count = 0;
        let mut defender_count = 0;
        let mut king_alive = false;

        for row in 0..state.board_size() {
            for col in 0..state.board_size() {
                let pos = crate::game::Position::new(row, col);
                match temp_state.get_piece(pos) {
                    Some(crate::game::Piece::Attacker) => attacker_count += 1,
                    Some(crate::game::Piece::Defender) => defender_count += 1,
                    Some(crate::game::Piece::King) => king_alive = true,
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

impl Bot for GreedyBot {
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
}
