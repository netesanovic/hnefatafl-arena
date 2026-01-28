use hnefatafl_arena::*;
use std::time::Duration;

fn main() {
    println!("Simple Hnefatafl Match Demo\n");

    // Create two bots
    let bot1 = Box::new(RandomBot::new("Random Alice".to_string()));
    let bot2 = Box::new(GreedyBot::new("Greedy Bob".to_string()));

    // Configure match with shorter time limits
    let config = MatchConfig {
        time_per_move: Duration::from_millis(500),
        max_moves: 10,
    };

    // Run the match with verbose output
    let mut match_game = Match::new(bot1, bot2, config, true);
    let result = match_game.play();

    // Print summary
    println!("\n{}", "=".repeat(60));
    match &result {
        MatchResult::AttackersWin { winner_name, moves } => {
            println!("🎉 {} won as Attackers in {} moves!", winner_name, moves);
        }
        MatchResult::DefendersWin { winner_name, moves } => {
            println!("🎉 {} won as Defenders in {} moves!", winner_name, moves);
        }
        MatchResult::Draw { moves } => {
            println!("🤝 Draw after {} moves", moves);
        }
        MatchResult::Timeout { violator, winner } => {
            println!("⏱️  {} wins! {} timed out", winner, violator);
        }
        MatchResult::IllegalMove { violator, winner } => {
            println!("❌ {} wins! {} made an illegal move", winner, violator);
        }
    }
    println!("{}", "=".repeat(60));
}
