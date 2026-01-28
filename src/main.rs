use hnefatafl_arena::*;
use std::time::Duration;

fn main() {
    println!("Hnefatafl Arena - Bot Tournament System");
    println!("========================================\n");
    
    // Create some example bots
    let bot1 = Box::new(RandomBot::new("RandomBot1".to_string()));
    let bot2 = Box::new(GreedyBot::new("GreedyBot1".to_string()));
    
    // Configure match
    let config = MatchConfig {
        time_per_move: Duration::from_secs(2),
        max_moves: 150,
    };
    
    // Play a match
    let mut match_game = Match::new(bot1, bot2, config, true);
    let result = match_game.play();
    
    // Display result
    println!("\n========================================");
    println!("Match Result:");
    match result {
        MatchResult::AttackersWin { winner_name, moves } => {
            println!("  {} wins as Attackers in {} moves!", winner_name, moves);
        }
        MatchResult::DefendersWin { winner_name, moves } => {
            println!("  {} wins as Defenders in {} moves!", winner_name, moves);
        }
        MatchResult::Draw { moves } => {
            println!("  Draw after {} moves", moves);
        }
        MatchResult::Timeout { violator, winner } => {
            println!("  {} wins by timeout (opponent: {})", winner, violator);
        }
        MatchResult::IllegalMove { violator, winner } => {
            println!("  {} wins by illegal move (opponent: {})", winner, violator);
        }
    }
    println!("========================================");
}
