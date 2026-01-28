use hnefatafl_arena::{Bot, Match, MatchConfig, PluginBot, Variant};
use std::time::Duration;

fn main() {
    println!("Alpha-Beta Bot Demo");
    println!("===================\n");

    // Load the alpha-beta plugin
    let plugin_path = if cfg!(target_os = "linux") {
        "plugins/alphabeta_bot_plugin/target/release/libalphabeta_bot_plugin.so"
    } else if cfg!(target_os = "macos") {
        "plugins/alphabeta_bot_plugin/target/release/libalphabeta_bot_plugin.dylib"
    } else if cfg!(target_os = "windows") {
        "plugins/alphabeta_bot_plugin/target/release/alphabeta_bot_plugin.dll"
    } else {
        panic!("Unsupported platform");
    };

    let alphabeta_bot = match PluginBot::load(plugin_path) {
        Ok(bot) => {
            println!("✅ Successfully loaded plugin: {}", bot.name());
            Box::new(bot) as Box<dyn Bot>
        }
        Err(e) => {
            eprintln!("❌ Failed to load plugin: {}", e);
            eprintln!("\nMake sure to compile the plugin first:");
            eprintln!("  ./build_plugin.sh alphabeta_bot_plugin");
            std::process::exit(1);
        }
    };

    // Load the greedy bot for comparison
    let greedy_path = if cfg!(target_os = "linux") {
        "plugins/greedy_bot_plugin/target/release/libgreedy_bot_plugin.so"
    } else if cfg!(target_os = "macos") {
        "plugins/greedy_bot_plugin/target/release/libgreedy_bot_plugin.dylib"
    } else if cfg!(target_os = "windows") {
        "plugins/greedy_bot_plugin/target/release/greedy_bot_plugin.dll"
    } else {
        panic!("Unsupported platform");
    };

    let greedy_bot = match PluginBot::load(greedy_path) {
        Ok(bot) => {
            println!("✅ Successfully loaded plugin: {}", bot.name());
            Box::new(bot) as Box<dyn Bot>
        }
        Err(e) => {
            eprintln!("❌ Failed to load greedy plugin: {}", e);
            eprintln!("\nMake sure to compile the plugin first:");
            eprintln!("  ./build_plugin.sh greedy_bot_plugin");
            std::process::exit(1);
        }
    };

    println!("\n🎮 Testing Alpha-Beta search bot");
    println!("   Using Brandubh variant (7x7) for faster games\n");

    let config = MatchConfig {
        time_per_move: Duration::from_secs(5),
        max_moves: 50,

    };

    println!("{}", "=".repeat(60));
    let mut game = Match::with_variant(greedy_bot, alphabeta_bot, config, true, Variant::Brandubh);

    let result = game.play();
    println!("{}", "=".repeat(60));

    match result.winner() {
        Some(winner) => println!("\n🎉 Winner: {}", winner),
        None => println!("\n🤝 Game ended in a draw"),
    }

    println!("\n💡 Alpha-Beta Bot Features:");
    println!("   • Minimax search with alpha-beta pruning");
    println!("   • Iterative deepening for time management");
    println!("   • Position evaluation heuristics");
}
