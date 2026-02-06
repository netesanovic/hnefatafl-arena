use hnefatafl_arena::{Bot, GreedyBot, Match, MatchConfig, PluginBot, RandomBot};
use std::time::Duration;

fn main() {
    let mcts_bot = PluginBot::load("plugins/mcts_bot_plugin/target/release/libmcts_bot_plugin.so")
        .expect("Failed to load bot");

    // let opponent = Box::new(GreedyBot::new("Greedy".to_string()));
    let opponent = PluginBot::load("plugins/alphabeta_bot_plugin/target/release/libalphabeta_bot_plugin.so")
        .expect("Failed to load bot");

    let config = MatchConfig {
        time_per_move: Duration::from_secs(60),
        ..Default::default()
    };

    let mut game = Match::new(Box::new(mcts_bot), Box::new(opponent),config, true);
    let result = game.play();

    println!("\nResult: {:?}", result.winner());
}