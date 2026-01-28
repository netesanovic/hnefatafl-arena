# API Quick Reference

## Core Types

### GameState
```rust
let state = GameState::new();  // Create initial game

// Query state
state.current_player() -> Player
state.legal_moves() -> Vec<Move>
state.is_game_over() -> bool
state.result() -> Option<&GameResult>
state.move_count() -> usize
state.get_piece(pos) -> Option<Piece>
state.display_board() -> String

// Make move
state.make_move(mv) -> Result<(), GameError>
```

### Player
```rust
pub enum Player {
    Attackers,  // 24 pieces, start first, win by capturing king
    Defenders,  // 12 pieces + king, win by getting king to corner
}

player.opponent() -> Player  // Get opposite player
```

### Piece
```rust
pub enum Piece {
    Attacker,  // Belongs to Attackers
    Defender,  // Belongs to Defenders
    King,      // Belongs to Defenders, must reach corner to win
}
```

### Position
```rust
let pos = Position::new(row, col);  // row and col are 0-11
pos.row  // usize
pos.col  // usize
```

### Move
```rust
let mv = Move::new(from, to);
mv.from  // Position
mv.to    // Position
```

## Bot Trait

```rust
pub trait Bot {
    fn name(&self) -> &str;
    
    // Main function - return your move
    fn get_move(&mut self, state: &GameState, time_limit: Duration) -> Option<Move>;
    
    // Optional callbacks
    fn game_start(&mut self, player: Player) {}
    fn notify_move(&mut self, mv: Move) {}
    fn game_end(&mut self) {}
}
```

## Match Setup

```rust
// Create bots
let bot1 = Box::new(MyBot::new("Bot1".to_string()));
let bot2 = Box::new(MyBot::new("Bot2".to_string()));

// Configure match
let config = MatchConfig {
    time_per_move: Duration::from_secs(5),
    max_moves: 200,
};

// Play match
let mut match_game = Match::new(bot1, bot2, config, true);
let result = match_game.play();
```

## Plugin Bots (NEW!)

Load bots from compiled shared libraries:

```rust
use hnefatafl_arena::PluginBot;

// Load a bot plugin
let bot = PluginBot::load("path/to/libmy_bot.so")?;
let bot = Box::new(bot);

// Use it like any other bot
let mut game = Match::new(bot, opponent, config, true);
```

See [PLUGIN_GUIDE.md](PLUGIN_GUIDE.md) for detailed plugin creation instructions.

## Match Result

```rust
pub enum MatchResult {
    AttackersWin { winner_name: String, moves: usize },
    DefendersWin { winner_name: String, moves: usize },
    Draw { moves: usize },
    Timeout { violator: String, winner: String },
    IllegalMove { violator: String, winner: String },
}

result.winner() -> Option<&str>
```

## Common Patterns

### Evaluate moves
```rust
fn evaluate_move(&self, state: &GameState, mv: Move) -> i32 {
    let mut temp_state = state.clone();
    temp_state.make_move(mv).ok()?;
    
    // Count pieces, check king position, etc.
    // Return score (higher is better)
}
```

### Find best move
```rust
let moves = state.legal_moves();
let best_move = moves.into_iter()
    .max_by_key(|&mv| self.evaluate_move(state, mv));
```

### Check if move captures
```rust
let original_state = state.clone();
let mut new_state = state.clone();
new_state.make_move(mv)?;

// Count pieces to see if any were captured
```

### Simple evaluation function
```rust
fn evaluate(&self, state: &GameState) -> i32 {
    let mut score = 0;
    
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            let pos = Position::new(row, col);
            match state.get_piece(pos) {
                Some(Piece::Attacker) => score -= 1,
                Some(Piece::Defender) => score += 2,
                Some(Piece::King) => score += 10,
                None => {}
            }
        }
    }
    
    score
}
```

## Board Constants

```rust
pub const BOARD_SIZE: usize = 11;  // 11x11 board

// Corners are at: (0,0), (0,10), (10,0), (10,10)
// Throne is at: (5,5)
```

## Movement Rules

- All pieces move like rooks (any distance in straight lines)
- Cannot jump over other pieces
- Only the king can land on throne or corners
- Pieces are captured by sandwiching between two opponent pieces
- King must be surrounded on all 4 sides to be captured

## Win Conditions

- **Defenders win**: King reaches any corner
- **Attackers win**: King is captured
- **Draw**: Max moves reached or no legal moves
