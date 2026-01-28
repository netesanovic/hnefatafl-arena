use hnefatafl_arena::*;
use std::time::Duration;

fn main() {
    println!("Brandubh Match Demo (7x7 Irish Variant)\n");
    
    // Create two bots
    let bot1 = Box::new(RandomBot::new("Random Alice".to_string()));
    let bot2 = Box::new(GreedyBot::new("Greedy Bob".to_string()));
    
    // Configure match with shorter time limits
    let config = MatchConfig {
        time_per_move: Duration::from_millis(500),
        max_moves: 100,

    };
    
    // Create a Brandubh game state
    let brandubh_state = GameState::new_brandubh();
    
    println!("Starting board for Brandubh:");
    println!("{}\n", brandubh_state.display_board());
    println!("Brandubh is played on a 7x7 board with:");
    println!("- 1 King (K) in the center");
    println!("- 4 Defenders (D) around the king");
    println!("- 8 Attackers (A) on the edges");
    println!("\nRules are the same as Copenhagen Hnefatafl:");
    println!("- Defenders win if king reaches any corner");
    println!("- Attackers win if they capture the king\n");
    
    // Create a Brandubh match
    println!("Running Brandubh match...\n");
    
    let mut match_game = Match::with_variant(bot1, bot2, config, true, Variant::Brandubh);
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
