/// Template for students to create their own Hnefatafl bot
/// 
/// Instructions:
/// 1. Copy this file and rename it (e.g., my_bot.rs)
/// 2. Implement the `get_move` function with your strategy
/// 3. Optionally implement game_start, notify_move, and game_end for more advanced features
/// 4. Test your bot against the example bots
///
/// To use in your project:
/// ```rust
/// use hnefatafl_arena::*;
/// 
/// // Include your bot file
/// mod my_bot;
/// use my_bot::MyBot;
/// 
/// fn main() {
///     let my_bot = Box::new(MyBot::new("MyBot".to_string()));
///     let opponent = Box::new(GreedyBot::new("Greedy".to_string()));
///     
///     let config = MatchConfig::default();
///     let mut match_game = Match::new(my_bot, opponent, config, true);
///     let result = match_game.play();
/// }
/// ```

use hnefatafl_arena::*;
use std::time::Duration;

pub struct MyBot {
    name: String,
    // Add any fields you need to track state
    // For example:
    // my_player: Option<Player>,
    // move_history: Vec<Move>,
}

impl MyBot {
    pub fn new(name: String) -> Self {
        MyBot {
            name,
            // Initialize your fields here
        }
    }
    
    // Helper function: evaluate how good a position is for us
    fn evaluate_position(&self, state: &GameState) -> i32 {
        // TODO: Implement your position evaluation
        // Ideas:
        // - Count pieces for each side
        // - Consider king position and safety
        // - Evaluate control of key squares (corners, throne)
        // - Consider mobility (number of legal moves)
        
        0 // Placeholder
    }
    
    // Helper function: check if a move captures any pieces
    fn captures_piece(&self, state: &GameState, mv: Move) -> bool {
        // TODO: Check if making this move would capture an opponent's piece
        // You can make a copy of the state and try the move:
        // let mut temp_state = state.clone();
        // temp_state.make_move(mv).ok()?;
        // Then compare piece counts
        
        false // Placeholder
    }
}

impl Bot for MyBot {
    fn name(&self) -> &str {
        &self.name
    }
    
    /// This is where you implement your bot's strategy!
    /// 
    /// Parameters:
    /// - state: The current game state (read-only)
    /// - time_limit: How much time you have to make a move
    /// 
    /// You must return:
    /// - Some(Move) if you have a move to make
    /// - None if there are no legal moves (you lose)
    /// 
    /// Tips:
    /// - Use state.legal_moves() to get all possible moves
    /// - Use state.current_player() to know if you're attackers or defenders
    /// - Use state.get_piece(position) to check what's on a square
    /// - You can clone the state to try out moves without affecting the real game
    fn get_move(&mut self, state: &GameState, time_limit: Duration) -> Option<Move> {
        let moves = state.legal_moves();
        
        if moves.is_empty() {
            return None;
        }
        
        // TODO: Implement your strategy here!
        // 
        // Simple strategies you could try:
        // 
        // 1. Random (baseline):
        //    Just pick a random move from the legal moves
        // 
        // 2. Greedy captures:
        //    Prefer moves that capture opponent pieces
        // 
        // 3. King safety (defenders):
        //    If playing as defenders, keep king away from attackers
        //    and move towards corners
        // 
        // 4. King hunting (attackers):
        //    If playing as attackers, surround the king
        // 
        // 5. Minimax/Alpha-Beta:
        //    Look ahead several moves and choose the best one
        // 
        // 6. Monte Carlo Tree Search:
        //    Simulate random games and pick the move that wins most often
        
        // For now, just return the first legal move (you should improve this!)
        Some(moves[0])
    }
    
    /// Called when the game starts
    /// Use this to initialize any game-specific state
    fn game_start(&mut self, player: Player) {
        // TODO: Store which side you're playing as if needed
        println!("{} starting as {:?}", self.name, player);
    }
    
    /// Called after each move (including opponent's moves)
    /// Use this to track the game state if needed
    fn notify_move(&mut self, mv: Move) {
        // TODO: Update your internal state based on the move
        // For example, track move history:
        // self.move_history.push(mv);
    }
    
    /// Called when the game ends
    /// Use this for cleanup or statistics
    fn game_end(&mut self) {
        // TODO: Any cleanup or final statistics
        println!("{} game ended", self.name);
    }
}

// Example test function
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_bot_plays() {
        let mut bot = MyBot::new("TestBot".to_string());
        let state = GameState::new(Variant::Copenhagen);
        let mv = bot.get_move(&state, Duration::from_secs(1));
        assert!(mv.is_some(), "Bot should return a move");
    }
}

// Main function for testing
fn main() {
    let bot1 = Box::new(MyBot::new("MyBot".to_string()));
    let bot2 = Box::new(RandomBot::new("RandomBot".to_string()));
    
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
