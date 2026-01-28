# Plugin Bot System

This guide explains how to create bots as compiled plugins where the source code is not visible to other students.

## Overview

The plugin system allows you to:
- **Compile your bot as a shared library** (`.so`, `.dll`, or `.dylib`)
- **Hide your source code** - only distribute the compiled binary
- **Use full Rust capabilities** in your bot implementation

## Why Use Plugins?

1. **Privacy**: Students can't see each other's strategies
2. **Performance**: Native compiled code runs at full speed
3. **Distribution**: Easy to share compiled bots without source code

## Example Plugin Bots

Two complete examples are included:

- **Greedy Bot** (`plugins/greedy_bot_plugin/`) - Simple single-move evaluation
- **Alpha-Beta Bot** (`plugins/alphabeta_bot_plugin/`) - Advanced minimax search
  - Wins as defenders in ~8 moves vs greedy bot
  - Uses exponential king position evaluation
  - Implements immediate win detection
  - Great reference for competitive bots!

## Creating a Plugin Bot

### Step 1: Set up the Plugin Project

Create a new directory in `plugins/`:

```bash
mkdir -p plugins/my_bot
cd plugins/my_bot
```

### Step 2: Create `Cargo.toml`

```toml
[package]
name = "my_bot_plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]  # This creates a shared library

[dependencies]
hnefatafl-arena = { path = "../.." }
```

**Important**: `crate-type = ["cdylib"]` tells Rust to compile as a dynamic library.

### Step 3: Implement Your Bot

Create `src/lib.rs`:

```rust
use hnefatafl_arena::{Bot, GameState, Move, Player};
use std::time::Duration;

pub struct MyBot {
    name: String,
    // Add your internal state here
}

impl Default for MyBot {
    fn default() -> Self {
        Self {
            name: "MyBot".to_string(),
        }
    }
}

impl Bot for MyBot {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_move(&mut self, state: &GameState, time_limit: Duration) -> Option<Move> {
        // Your strategy here
        let moves = state.legal_moves();
        if moves.is_empty() {
            None
        } else {
            Some(moves[0])  // Replace with your logic
        }
    }

    fn game_start(&mut self, _player: Player) {
        // Called when game starts - initialize your state
    }

    fn notify_move(&mut self, _mv: Move) {
        // Called when any move is made (yours or opponent's)
        // Update your internal game state here
    }
}

// REQUIRED: Export your bot using this macro
hnefatafl_arena::export_bot!(MyBot);
```

### Step 4: Compile Your Plugin

```bash
cd plugins/my_bot
cargo build --release
```

This creates:
- **Linux**: `target/release/libmy_bot_plugin.so`
- **macOS**: `target/release/libmy_bot_plugin.dylib`
- **Windows**: `target/release/my_bot_plugin.dll`

### Step 5: Distribute Your Bot

Share only the compiled library file - **not** the source code! Students can use your bot without seeing how it works.

## Using Plugin Bots

### Loading a Plugin

```rust
use hnefatafl_arena::{PluginBot, Match, MatchConfig};
use std::time::Duration;

fn main() {
    // Load the plugin (adjust path for your OS)
    let my_bot = PluginBot::load("path/to/libmy_bot_plugin.so")
        .expect("Failed to load plugin");
    
    let opponent = Box::new(RandomBot::new("Opponent".to_string()));
    
    let config = MatchConfig {
        time_per_move: Duration::from_secs(5),
        max_moves: 200,
    };
    
    let mut game = Match::new(
        Box::new(my_bot),
        opponent,
        config,
        true  // verbose
    );
    
    let result = game.play();
}
```

### Running the Example

```bash
# First, compile the example plugin
cd plugins/greedy_bot_plugin
cargo build

# Then run the example
cd ../..
cargo run --example plugin_match
```

## Troubleshooting

### Plugin Won't Load

```
Error: Failed to load library: libmy_bot_plugin.so
```

**Solution**: Make sure the file exists and is in the correct location. Use absolute paths if needed.

### Missing Symbol

```
Error: Failed to find create_bot function
```

**Solution**: Make sure you added `hnefatafl_arena::export_bot!(YourBot);` at the end of your lib.rs.

### Wrong Library Type

```
Error: crate-type must be cdylib
```

**Solution**: Add `crate-type = ["cdylib"]` to `[lib]` section in Cargo.toml.

## Example Plugin Bots

See the included examples for complete implementations:
- `plugins/greedy_bot_plugin/` - Simple single-move evaluation
- `plugins/alphabeta_bot_plugin/` - Advanced minimax search with pruning

## Security Note

While plugin bots hide source code, remember:
- Compiled code can be reverse-engineered (though it's difficult)
- For true security in competitions, run bots in sandboxed environments
- The plugin system provides privacy, not cryptographic security

## Learning Resources

- **Game Tree Search**: Minimax, alpha-beta pruning
- **Transposition Tables**: Common in game tree search
- **FFI**: Rust's Foreign Function Interface for interop

## Next Steps

1. Create your own plugin bot
2. Implement a strategic evaluation function
3. Test against the example bots
4. Enter tournaments!

