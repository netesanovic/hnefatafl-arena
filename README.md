# Hnefatafl Arena

A Rust-based tournament system for Hnefatafl bots. Students can write their own bots that play the ancient Viking board game Hnefatafl against each other.

## Features

✨ **Plugin System** - Compile bots as shared libraries to hide source code  
🎯 **Multiple Variants** - Copenhagen (11x11) and Brandubh (7x7)  
🏆 **Tournament Mode** - Round-robin competitions  
📊 **Match Statistics** - Track wins, timeouts, and illegal moves  

## Quick Start

### Option 1: Regular Bot (Source Code Visible)

```rust
use hnefatafl_arena::{Bot, GameState, Move};

pub struct MyBot { name: String }

impl Bot for MyBot {
    fn name(&self) -> &str { &self.name }
    
    fn get_move(&mut self, state: &GameState, _time_limit: Duration) -> Option<Move> {
        state.legal_moves().first().copied()
    }
}
```

### Option 2: Plugin Bot (Source Code Hidden) 🔒

Create `plugins/my_bot/Cargo.toml`:
```toml
[package]
name = "my_bot_plugin"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
hnefatafl-arena = { path = "../.." }
```

Implement your bot in `src/lib.rs` and export it:
```rust
hnefatafl_arena::export_bot!(MyBot);
```

Compile and load:
```bash
cargo build --release
```

```rust
let bot = PluginBot::load("target/release/libmy_bot_plugin.so")?;
```

📖 **See [PLUGIN_GUIDE.md](PLUGIN_GUIDE.md) for complete instructions**

## Supported Variants

- **Copenhagen Hnefatafl** (11x11) - Traditional Viking game
- **Brandubh** (7x7) - Irish variant, faster gameplay

See [BRANDUBH.md](BRANDUBH.md) for details on the Irish variant.

## What is Hnefatafl?

Hnefatafl is an asymmetric strategy board game where:
- **Attackers** surround the board and try to capture the king
- **Defenders** (including the king) try to help the king escape to a corner

### Copenhagen Rules (Default)

1. **Setup**: 11x11 board with 24 attackers vs 12 defenders + king
2. **Movement**: All pieces move like rooks in chess (any distance in straight lines)
3. **Capture**: Sandwich an opponent's piece between two of your pieces
4. **King Capture**: King must be surrounded on all four sides
5. **Win Conditions**:
   - Defenders win if the king reaches any corner
   - Attackers win if they capture the king

### Brandubh Rules (7x7)

- Smaller 7x7 board
- 8 attackers vs 4 defenders + king
- Same rules, faster gameplay
- See [BRANDUBH.md](BRANDUBH.md)

## Example Bots Included

- **Random Bot**: Makes random legal moves
- **Greedy Bot**: Evaluates one move ahead (plugin example)
- **Alpha-Beta Bot**: Advanced minimax search with pruning (strong AI example)

## Project Structure

```
src/
├── lib.rs          # Library exports
├── main.rs         # Example tournament runner
├── game.rs         # Game logic and rules
├── bot.rs          # Bot trait and example bots
├── arena.rs        # Match and tournament management
└── plugin.rs       # Plugin system for compiled bots

plugins/
├── greedy_bot_plugin/      # Simple example plugin
└── alphabeta_bot_plugin/   # Advanced AI with minimax search
```

## Creating Your Own Bot

Implement the `Bot` trait:

```rust
use hnefatafl_arena::{Bot, GameState, Move, Player};
use std::time::Duration;

pub struct MyBot {
    name: String,
}

impl MyBot {
    pub fn new(name: String) -> Self {
        MyBot { name }
    }
}

impl Bot for MyBot {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn get_move(&mut self, state: &GameState, time_limit: Duration) -> Option<Move> {
        // Your bot logic here!
        // Return None if no moves available
        let moves = state.legal_moves();
        if moves.is_empty() {
            None
        } else {
            // Example: return first legal move
            Some(moves[0])
        }
    }
    
    // Optional: Called when game starts
    fn game_start(&mut self, player: Player) {
        println!("{} playing as {:?}", self.name, player);
    }
    
    // Optional: Notified of all moves (including opponent's)
    fn notify_move(&mut self, mv: Move) {
        // Track game state if needed
    }
    
    // Optional: Called when game ends
    fn game_end(&mut self) {
        // Cleanup if needed
    }
}
```

## Running Matches

### Single Match (Copenhagen)

```rust
use hnefatafl_arena::*;
use std::time::Duration;

fn main() {
    let bot1 = Box::new(MyBot::new("Bot1".to_string()));
    let bot2 = Box::new(MyBot::new("Bot2".to_string()));
    
    let config = MatchConfig {
        time_per_move: Duration::from_secs(5),
        max_moves: 200,
    };
    
    // Default is Copenhagen variant
    let mut match_game = Match::new(bot1, bot2, config, true);
    let result = match_game.play();
}
```

### Using Plugin Bots

```rust
// Load compiled bot plugins (source code hidden)
let bot1 = PluginBot::load("plugins/bot1.so")?;
let bot2 = PluginBot::load("plugins/bot2.so")?;

let config = MatchConfig::default();

let mut game = Match::new(Box::new(bot1), Box::new(bot2), config, true);
```
```

### Single Match (Brandubh)

```rust
// Play the smaller Brandubh variant
let mut match_game = Match::with_variant(
    bot1, 
    bot2, 
    config, 
    true,
    Variant::Brandubh
);
let result = match_game.play();
```

### Tournament (Round-Robin)

```rust
let mut tournament = Tournament::new(config, true);
tournament.add_bot("Bot1".to_string(), Box::new(MyBot::new("Bot1".to_string())));
tournament.add_bot("Bot2".to_string(), Box::new(MyBot::new("Bot2".to_string())));
// Add more bots...

let results = tournament.run_round_robin();
results.display();
```

## API Reference

### GameState

```rust
// Get current player
state.current_player() -> Player

// Get all legal moves
state.legal_moves() -> Vec<Move>

// Get piece at position
state.get_piece(pos: Position) -> Option<Piece>

// Make a move
state.make_move(mv: Move) -> Result<(), GameError>

// Check game status
state.is_game_over() -> bool
state.result() -> Option<&GameResult>
state.move_count() -> usize

// Display board (for debugging)
state.display_board() -> String
```

### Types

```rust
pub enu the arena
cargo build --release

# Run example match
cargo run --example simple_match

# Run plugin example (compile plugin first!)
cd plugins/greedy_bot_plugin && cargo build
cd ../..
cargo run --example plugin_match

# Run tests
cargo test
```

## Documentation

- **[API_REFERENCE.md](API_REFERENCE.md)** - Complete API documentation
- **[PLUGIN_GUIDE.md](PLUGIN_GUIDE.md)** - Creating plugin bots
- **[BRANDUBH.md](BRANDUBH.md)** - Irish variant rules
- **[TOURNAMENT.md](TOURNAMENT.md)** - Tournament system guide
```

## Board Representation

When displayed:
- `A` = Attacker
- `D` = Defender
- `K` = King
- `T` = Throne (empty center)
- `X` = Corner (goal for king)
- `.` = Empty square

## Building and Running

```bash
# Build
cargo build --release

# Run example
cargo run --release

# Run tests
cargo test

# Add as dependency in your project
cargo add --path /path/to/hnefatafl-arena
```

## Example Bots Included

1. **RandomBot**: Randomly selects from legal moves
2. **GreedyBot**: Tries to maximize piece captures

## Tips for Bot Development

1. **Time Management**: You have a time limit per move - make sure your bot responds in time
2. **Evaluation**: Consider piece counts, king safety, and board control
3. **Strategy**: Attackers should coordinate to trap the king; Defenders should create escape paths
4. **Testing**: Test against the example bots to verify your implementation

## Match Configuration

```rust
pub struct MatchConfig {
    pub time_per_move: Duration,  // Time limit per move
    pub max_moves: usize,          // Maximum moves before draw
}
```

## Match Results

```rust
pub enum MatchResult {
    AttackersWin { winner_name: String, moves: usize },
    DefendersWin { winner_name: String, moves: usize },
    Draw { moves: usize },
    Timeout { violator: String, winner: String },
    IllegalMove { violator: String, winner: String },
}
```

## License

MIT License - Feel free to use for educational purposes!
