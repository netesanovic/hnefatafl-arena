use hnefatafl_arena::{Bot, GreedyBot, Match, MatchConfig, PluginBot, RandomBot};
use std::time::Duration;

fn main() {
    // Create a specific folder for this iteration tier
    // e.g., "benchmark_200000_iters"
    let folder = format!("mcts_vs_greedy");
    std::fs::create_dir_all(&folder).ok();
    println!("============================================================");
    println!("Starting Test");
    println!("Config A: {} @ 200k iters", "MCTS");
    println!("Config B: {:?} ", "Greedy");
    println!("============================================================");

    // === PHASE 1: A is White, B is Black ===
    println!("Phase 1: A (White) vs B (Black)");
    for i in 0..10 {
        println!("Phase 1: Starting game {}", i);

        let mcts_bot = PluginBot::load("plugins/mcts_bot_plugin/target/release/libmcts_bot_plugin.so")
            .expect("Failed to load bot");

        // let opponent = PluginBot::load("plugins/greedy_bot_plugin/target/release/libgreedy_bot_plugin.so")
        //     .expect("Failed to load bot");
        //let opponent = GreedyBot::new("Greedy".to_string());
        let opponent = PluginBot::load("plugins/alphabeta_bot_plugin/target/release/libalphabeta_bot_plugin.so")
            .expect("Failed to load bot");

        let file_name = format!("{}/mcts_white_{}.txt", folder, i);

        let config = MatchConfig {
            time_per_move: Duration::from_secs(600),
            ..Default::default()
        };

        let mut game = Match::new(Box::new(opponent), Box::new(mcts_bot), config, true);
        let result = game.play();

        println!("Phase 1 Game {} result: {:?}", i,  result.winner());
    }
    println!("\nPhase 1 Complete.");

    // === PHASE 2: B is White, A is Black ===
    println!("Phase 2: B (White) vs A (Black)");

    for i in 0..10 {
        println!("Phase 2: Starting game {}", i);

        let mcts_bot = PluginBot::load("plugins/mcts_bot_plugin/target/release/libmcts_bot_plugin.so")
            .expect("Failed to load bot");

        // let opponent = PluginBot::load("plugins/greedy_bot_plugin/target/release/libgreedy_bot_plugin.so")
        //     .expect("Failed to load bot");
        //let opponent = GreedyBot::new("Greedy".to_string());
        let opponent = PluginBot::load("plugins/alphabeta_bot_plugin/target/release/libalphabeta_bot_plugin.so")
            .expect("Failed to load bot");

        let file_name = format!("{}/mcts_black_{}.txt", folder, i);

        let config = MatchConfig {
            time_per_move: Duration::from_secs(600),
            ..Default::default()
        };

        let mut game = Match::new(Box::new(mcts_bot), Box::new(opponent), config, true);
        let result = game.play();

        println!("Phase 2 Game {} result: {:?}", i,  result.winner());
    }

}