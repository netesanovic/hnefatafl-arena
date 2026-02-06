pub mod hnefatafl;
pub mod zobrist;
pub mod transposition;
pub mod mcts;

use hnefatafl_arena::{Bot, GameState, Move, Player, Position, Piece};
use std::time::Duration;
use std::io::Write;
use crate::mcts::{SimulationType, MCTS};

pub struct McstBot {
    name: String,
    engine: MCTS,
    game: hnefatafl::GameState,
    current_player: Option<Player>,
}

impl Default for McstBot {
    fn default() -> Self {
        let engine = MCTS::new(0xCAFEBABE, 200_000, SimulationType::ParallelHeavy(8));
        let game = hnefatafl::GameState::new(&engine.z_table);
        Self {
            name: "McstBot".to_string(),
            engine,
            game,
            current_player: None,
        }
    }
}

impl McstBot {
    /// Convert internal move format [row, col, row, col] to arena Move
    #[inline]
    fn convert_move(&self, internal_move: &[usize; 4]) -> Move {
        Move {
            from: Position::new(internal_move[0], internal_move[1]),
            to: Position::new(internal_move[2], internal_move[3]),
        }
    }

    /// Convert arena Move to internal move format [row, col, row, col]
    #[inline]
    fn convert_to_internal(&self, mv: Move) -> [usize; 4] {
        [mv.from.row, mv.from.col, mv.to.row, mv.to.col]
    }
}

impl Bot for McstBot {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_move(&mut self, state: &GameState, _time_limit: Duration) -> Option<Move> {
        // Reset game state to match arena state on each move ?
        // pretty stupid, i dont wanna lose my game state every turn by creating a new one ?!
        // self.game = hnefatafl::GameState::new(&self.engine.z_table);

        if !self.game.has_legal_move(self.game.player) {
            return None;
        }

        // Set correct player (convert arena Player to internal 'B'/'W')
        let internal_player = match state.current_player() {
            Player::Attackers => 'B',  // Attackers are Black in Brandubh
            Player::Defenders => 'W',  // Defenders are White in Brandubh
        };
        self.game.player = internal_player;

        // Use stderr for debug output (safe in plugin context)
        let mut stderr = std::io::stderr();

        // Run MCTS search to find best move
        let played_move = self.engine.computer_move(&mut self.game, &mut stderr);
        Some(self.convert_move(&played_move))
    }

    fn game_start(&mut self, player: Player) {
        self.current_player = Some(player);

        // Reset engine and game state
        // self.engine = MCTS::new(0xCAFEBABE, 200_000, SimulationType::ParallelHeavy(8));
        // self.game = hnefatafl::GameState::new(&self.engine.z_table);
        
        // Set starting player
        self.game.player = match player {
            Player::Attackers => 'B',
            Player::Defenders => 'W',
        };
    }

    fn notify_move(&mut self, mv: Move) {
        // Update internal game state with the move
        let internal_mv = self.convert_to_internal(mv);
        
        // Apply move to internal state
        let mut stderr = std::io::stderr();
        let _ = writeln!(stderr, "Notified of move: {:?}", internal_mv);
        
        self.game.move_piece(&internal_mv, &self.engine.z_table, true, &mut stderr);
    }
}

// REQUIRED: Export your bot using this macro
hnefatafl_arena::export_bot!(McstBot);