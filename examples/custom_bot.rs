/// Example of how students can create their own bot

use hnefatafl_arena::*;
use std::time::Duration;

/// A custom bot that prefers moves closer to the center (simple heuristic)
pub struct CenterBot {
    name: String,
}

impl CenterBot {
    pub fn new(name: String) -> Self {
        CenterBot { name }
    }
    
    fn distance_to_center(&self, pos: Position, board_size: usize) -> f64 {
        let center = board_size as f64 / 2.0;
        let dr = pos.row as f64 - center;
        let dc = pos.col as f64 - center;
        (dr * dr + dc * dc).sqrt()
    }
    
    fn evaluate_move(&self, mv: Move, board_size: usize) -> f64 {
        // Prefer moves that move pieces toward or away from center
        // depending on role (would need to track player role)
        -self.distance_to_center(mv.to, board_size)
    }
}

impl Bot for CenterBot {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn get_move(&mut self, state: &GameState, _time_limit: Duration) -> Option<Move> {
        let moves = state.legal_moves();
        if moves.is_empty() {
            return None;
        }
        
        let board_size = state.board_size();
        
        // Find best move according to our heuristic
        moves.into_iter()
            .max_by(|a, b| {
                let score_a = self.evaluate_move(*a, board_size);
                let score_b = self.evaluate_move(*b, board_size);
                score_a.partial_cmp(&score_b).unwrap()
            })
    }
    
    fn game_start(&mut self, player: Player) {
        println!("{} starting as {:?}", self.name, player);
    }
}

fn main() {
    println!("Custom Bot Example\n");
    
    let bot1 = Box::new(CenterBot::new("CenterBot".to_string()));
    let bot2 = Box::new(GreedyBot::new("GreedyBot".to_string()));
    
    let config = MatchConfig {
        time_per_move: Duration::from_secs(1),
        max_moves: 150,
    };
    
    let mut match_game = Match::new(bot1, bot2, config, true);
    let result = match_game.play();
    
    println!("\nMatch completed!");
    if let Some(winner) = result.winner() {
        println!("Winner: {}", winner);
    } else {
        println!("Draw!");
    }
}
