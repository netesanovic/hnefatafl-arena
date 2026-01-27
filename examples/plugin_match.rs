use hnefatafl_arena::{Bot, Match, MatchConfig, PluginBot, RandomBot};
use std::time::Duration;

fn main() {
    println!("Loading plugin bot...");

    // Load the plugin bot from the compiled shared library
    let plugin_path = if cfg!(target_os = "linux") {
        "plugins/greedy_bot_plugin/target/release/libgreedy_bot_plugin.so"
    } else if cfg!(target_os = "macos") {
        "plugins/greedy_bot_plugin/target/release/libgreedy_bot_plugin.dylib"
    } else if cfg!(target_os = "windows") {
        "plugins/greedy_bot_plugin/target/release/greedy_bot_plugin.dll"
    } else {
        panic!("Unsupported platform");
    };

    let greedy_bot = match PluginBot::load(plugin_path) {
        Ok(bot) => {
            println!("Successfully loaded plugin: {}", bot.name());
            Box::new(bot) as Box<dyn hnefatafl_arena::Bot>
        }
        Err(e) => {
            eprintln!("Failed to load plugin: {}", e);
            eprintln!("\nMake sure to compile the plugin first:");
            eprintln!("  cd plugins/greedy_bot_plugin");
            eprintln!("  cargo build");
            std::process::exit(1);
        }
    };

    let random_bot = Box::new(RandomBot::new("Random".to_string()));

    let config = MatchConfig {
        time_per_move: Duration::from_secs(5),
        max_moves: 10,
        enable_pondering: true,
    };

    let mut game = Match::new(greedy_bot, random_bot, config, true);

    println!("\n{}", "=".repeat(60));
    let result = game.play();
    println!("{}", "=".repeat(60));

    match result.winner() {
        Some(winner) => println!("\n🎉 Winner: {}", winner),
        None => println!("\n🤝 Game ended in a draw"),
    }
}
