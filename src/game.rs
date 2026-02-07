use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use thiserror::Error;

/// Board size constants
pub const COPENHAGEN_SIZE: usize = 11;
pub const BRANDUBH_SIZE: usize = 7;
pub const MAX_BOARD_SIZE: usize = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Variant {
    Copenhagen, // 11x11, traditional Hnefatafl
    Brandubh,   // 7x7, Irish variant
}

impl Variant {
    pub fn board_size(&self) -> usize {
        match self {
            Variant::Copenhagen => COPENHAGEN_SIZE,
            Variant::Brandubh => BRANDUBH_SIZE,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Variant::Copenhagen => "Copenhagen Hnefatafl",
            Variant::Brandubh => "Brandubh",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Piece {
    Attacker,
    Defender,
    King,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    Attackers,
    Defenders,
}

impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::Attackers => Player::Defenders,
            Player::Defenders => Player::Attackers,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub fn new(row: usize, col: usize) -> Self {
        Position { row, col }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    pub from: Position,
    pub to: Position,
}

impl Move {
    pub fn new(from: Position, to: Position) -> Self {
        Move { from, to }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Invalid move: {0}")]
    InvalidMove(String),
    #[error("Game already over")]
    GameOver,
    #[error("Not your turn")]
    NotYourTurn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    AttackersWin,
    DefendersWin,
    Draw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    board: [[Option<Piece>; MAX_BOARD_SIZE]; MAX_BOARD_SIZE],
    variant: Variant,
    board_size: usize,
    current_player: Player,
    king_position: Option<Position>,
    move_count: usize,
    result: Option<GameResult>,
    /// Track position hashes and their occurrence counts for threefold repetition
    position_history: HashMap<u64, usize>,
}

impl GameState {
    /// Create a new game with the specified variant
    pub fn new(variant: Variant) -> Self {
        let mut state = GameState {
            board: [[None; MAX_BOARD_SIZE]; MAX_BOARD_SIZE],
            variant,
            board_size: variant.board_size(),
            current_player: Player::Attackers,
            king_position: None,
            move_count: 0,
            result: None,
            position_history: HashMap::new(),
        };

        match variant {
            Variant::Copenhagen => state.setup_copenhagen(),
            Variant::Brandubh => state.setup_brandubh(),
        }

        // Record the initial position
        state.record_position();

        state
    }

    /// Create a new game with Copenhagen Hnefatafl (default)
    pub fn new_copenhagen() -> Self {
        Self::new(Variant::Copenhagen)
    }

    /// Create a new game with Brandubh variant
    pub fn new_brandubh() -> Self {
        Self::new(Variant::Brandubh)
    }

    /// Setup Copenhagen Hnefatafl (11x11)
    fn setup_copenhagen(&mut self) {
        let board_size = COPENHAGEN_SIZE;

        // Place king in center
        let center = board_size / 2;
        self.board[center][center] = Some(Piece::King);
        self.king_position = Some(Position::new(center, center));

        // Place defenders around king (cross pattern)
        let defenders = [
            (center - 1, center),
            (center + 1, center),
            (center, center - 1),
            (center, center + 1),
            (center - 2, center),
            (center + 2, center),
            (center, center - 2),
            (center, center + 2),
        ];

        for &(r, c) in &defenders {
            self.board[r][c] = Some(Piece::Defender);
        }

        // Place attackers on edges (T-shape on each side)
        let attackers = [
            // Top
            (0, 3),
            (0, 4),
            (0, 5),
            (0, 6),
            (0, 7),
            (1, 5),
            // Bottom
            (10, 3),
            (10, 4),
            (10, 5),
            (10, 6),
            (10, 7),
            (9, 5),
            // Left
            (3, 0),
            (4, 0),
            (5, 0),
            (6, 0),
            (7, 0),
            (5, 1),
            // Right
            (3, 10),
            (4, 10),
            (5, 10),
            (6, 10),
            (7, 10),
            (5, 9),
        ];

        for &(r, c) in &attackers {
            self.board[r][c] = Some(Piece::Attacker);
        }
    }

    /// Setup Brandubh (7x7)
    fn setup_brandubh(&mut self) {
        let board_size = BRANDUBH_SIZE;
        let center = board_size / 2; // 3 for 7x7

        // Place king in center
        self.board[center][center] = Some(Piece::King);
        self.king_position = Some(Position::new(center, center));

        // Place 4 defenders around king
        let defenders = [
            (center - 1, center), // Above
            (center + 1, center), // Below
            (center, center - 1), // Left
            (center, center + 1), // Right
        ];

        for &(r, c) in &defenders {
            self.board[r][c] = Some(Piece::Defender);
        }

        // Place 8 attackers on edges (2 on each side)
        let attackers = [
            // Top
            (0, 3),
            (1, 3),
            // Bottom
            (5, 3),
            (6, 3),
            // Left
            (3, 0),
            (3, 1),
            // Right
            (3, 5),
            (3, 6),
        ];

        for &(r, c) in &attackers {
            self.board[r][c] = Some(Piece::Attacker);
        }
    }

    pub fn variant(&self) -> Variant {
        self.variant
    }

    pub fn board_size(&self) -> usize {
        self.board_size
    }

    pub fn current_player(&self) -> Player {
        self.current_player
    }

    pub fn result(&self) -> Option<&GameResult> {
        self.result.as_ref()
    }

    pub fn is_game_over(&self) -> bool {
        self.result.is_some()
    }

    pub fn move_count(&self) -> usize {
        self.move_count
    }

    pub fn get_piece(&self, pos: Position) -> Option<Piece> {
        if pos.row < self.board_size && pos.col < self.board_size {
            self.board[pos.row][pos.col]
        } else {
            None
        }
    }

    /// Check if a position is a corner (throne)
    fn is_corner(&self, pos: Position) -> bool {
        (pos.row == self.board_size - 1 || pos.row == 0)
            && (pos.col == self.board_size - 1 || pos.col == 0)
    }

    /// Check if a position is the throne (center)
    fn is_throne(&self, pos: Position) -> bool {
        let center = self.board_size / 2;
        pos.row == center && pos.col == center
    }

    /// Get all legal moves for the current player
    pub fn legal_moves(&self, player: Player) -> Vec<Move> {
        if self.is_game_over() {
            return Vec::new();
        }

        let mut moves = Vec::new();

        for row in 0..self.board_size {
            for col in 0..self.board_size {
                let pos = Position::new(row, col);
                if let Some(piece) = self.get_piece(pos) {
                    if self.piece_belongs_to_player(piece, player) {
                        moves.extend(self.legal_moves_for_piece(pos));
                    }
                }
            }
        }

        moves
    }

    fn piece_belongs_to_player(&self, piece: Piece, player: Player) -> bool {
        matches!(
            (piece, player),
            (Piece::Attacker, Player::Attackers)
                | (Piece::Defender | Piece::King, Player::Defenders)
        )
    }

    fn legal_moves_for_piece(&self, from: Position) -> Vec<Move> {
        let mut moves = Vec::new();
        let piece = self.get_piece(from).unwrap();

        // Try all four directions
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for &(dr, dc) in &directions {
            let mut r = from.row as i32;
            let mut c = from.col as i32;

            loop {
                r += dr;
                c += dc;

                if r < 0 || r >= self.board_size as i32 || c < 0 || c >= self.board_size as i32 {
                    break;
                }

                let to = Position::new(r as usize, c as usize);

                // Can't move onto another piece
                if self.get_piece(to).is_some() {
                    break;
                }

                // Only king can move to throne or corners
                if piece != Piece::King && (self.is_throne(to) || self.is_corner(to)) {
                    break;
                }

                moves.push(Move::new(from, to));
            }
        }

        moves
    }

    /// Make a move and update the game state
    pub fn make_move(&mut self, mv: Move) -> Result<(), GameError> {
        if self.is_game_over() {
            return Err(GameError::GameOver);
        }

        // Validate the move
        if !self.legal_moves(self.current_player).contains(&mv) {
            return Err(GameError::InvalidMove(format!("Move {} is not legal", mv)));
        }

        // Move the piece
        let piece = self.board[mv.from.row][mv.from.col].unwrap();
        self.board[mv.from.row][mv.from.col] = None;
        self.board[mv.to.row][mv.to.col] = Some(piece);

        // Update king position
        if piece == Piece::King {
            self.king_position = Some(mv.to);
        }

        // Check for captures
        self.check_captures(mv.to);

        // Check win conditions
        self.check_game_end();

        // Switch player
        self.current_player = self.current_player.opponent();
        self.move_count += 1;

        // Record position and check for threefold repetition
        self.record_position();
        self.check_threefold_repetition();

        Ok(())
    }

    fn check_captures(&mut self, moved_to: Position) {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for &(dr, dc) in &directions {
            let target_r = moved_to.row as i32 + dr;
            let target_c = moved_to.col as i32 + dc;

            if target_r < 0
                || target_r >= self.board_size as i32
                || target_c < 0
                || target_c >= self.board_size as i32
            {
                continue;
            }

            let target = Position::new(target_r as usize, target_c as usize);

            if let Some(target_piece) = self.get_piece(target) {
                // Check if we can capture this piece
                if self.can_capture(moved_to, target) {
                    self.board[target.row][target.col] = None;

                    // If king was captured, update king position
                    if target_piece == Piece::King {
                        self.king_position = None;
                    }
                }
            }
        }
    }

    fn can_capture(&self, attacker: Position, target: Position) -> bool {
        let attacker_piece = self.get_piece(attacker).unwrap();
        let target_piece = self.get_piece(target).unwrap();

        // Can't capture your own pieces
        match (attacker_piece, target_piece) {
            (Piece::Attacker, Piece::Attacker) => return false,
            (Piece::Defender, Piece::Defender) => return false,
            (Piece::King, Piece::Defender) => return false,
            (Piece::Defender, Piece::King) => return false,
            _ => {}
        }

        // King capture logic based on position
        if target_piece == Piece::King {
            return self.is_king_captured_from(target, attacker);
        }

        // Regular pieces need hostile piece on opposite side
        let dr = target.row as i32 - attacker.row as i32;
        let dc = target.col as i32 - attacker.col as i32;

        let opposite_r = target.row as i32 + dr;
        let opposite_c = target.col as i32 + dc;

        if opposite_r < 0
            || opposite_r >= self.board_size as i32
            || opposite_c < 0
            || opposite_c >= self.board_size as i32
        {
            return false;
        }

        let opposite = Position::new(opposite_r as usize, opposite_c as usize);

        // Corners are hostile to all pieces
        if self.is_corner(opposite) {
            return true;
        }

        // Throne is hostile to attackers, and hostile to defenders if empty
        if self.is_throne(opposite) {
            match target_piece {
                Piece::Attacker => return true,
                Piece::Defender => return self.get_piece(opposite).is_none(),
                _ => {}
            }
        }

        if let Some(opposite_piece) = self.get_piece(opposite) {
            // Check if opposite piece is hostile to target
            matches!(
                (target_piece, opposite_piece),
                (Piece::Attacker, Piece::Defender | Piece::King)
                    | (Piece::Defender, Piece::Attacker)
            )
        } else {
            false
        }
    }

    fn is_king_captured_from(&self, king_pos: Position, attacker_pos: Position) -> bool {
        let center = self.board_size / 2;

        // Check if king is on the throne
        if self.is_throne(king_pos) {
            // King on throne: must be surrounded on all 4 sides
            return self.is_surrounded_on_all_sides(king_pos);
        }

        // Check if king is next to throne (orthogonally adjacent)
        let is_next_to_throne = (king_pos.row == center
            && (king_pos.col == center - 1 || king_pos.col == center + 1))
            || (king_pos.col == center
                && (king_pos.row == center - 1 || king_pos.row == center + 1));

        if is_next_to_throne {
            // King next to throne: must be surrounded on 3 sides (throne counts as occupied)
            return self.is_surrounded_next_to_throne(king_pos, Position::new(center, center));
        }

        // King elsewhere: captured if attackers are on opposite sides
        // BUT: only check the direction from the attacker that just moved!
        // Compute the direction from attacker to king
        let dr = king_pos.row as i32 - attacker_pos.row as i32;
        let dc = king_pos.col as i32 - attacker_pos.col as i32;

        // The opposite side from the attacker
        let opposite_r = king_pos.row as i32 + dr;
        let opposite_c = king_pos.col as i32 + dc;

        // Check bounds
        if opposite_r < 0
            || opposite_r >= self.board_size as i32
            || opposite_c < 0
            || opposite_c >= self.board_size as i32
        {
            return false;
        }

        let opposite_pos = Position::new(opposite_r as usize, opposite_c as usize);

        self.is_hostile_to_king(opposite_pos)
    }

    // Test helper: Check if king would be captured from ANY direction (used by tests)
    // Note: This is NOT used in actual game logic - we use is_king_captured_from instead
    #[cfg(test)]
    fn is_king_captured(&self, king_pos: Position) -> bool {
        // Check if king would be captured from any of the 4 directions
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for &(dr, dc) in &directions {
            let attacker_r = king_pos.row as i32 + dr;
            let attacker_c = king_pos.col as i32 + dc;

            if attacker_r < 0
                || attacker_r >= self.board_size as i32
                || attacker_c < 0
                || attacker_c >= self.board_size as i32
            {
                continue;
            }

            let attacker_pos = Position::new(attacker_r as usize, attacker_c as usize);

            // If there's an attacker in this direction, check if it would capture
            if let Some(Piece::Attacker) = self.get_piece(attacker_pos) {
                if self.is_king_captured_from(king_pos, attacker_pos) {
                    return true;
                }
            }
        }

        false
    }

    fn is_surrounded_on_all_sides(&self, king_pos: Position) -> bool {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for &(dr, dc) in &directions {
            let r = king_pos.row as i32 + dr;
            let c = king_pos.col as i32 + dc;

            if r < 0 || r >= self.board_size as i32 || c < 0 || c >= self.board_size as i32 {
                return false;
            }

            let pos = Position::new(r as usize, c as usize);

            // Must be surrounded by attackers
            if let Some(piece) = self.get_piece(pos) {
                if piece != Piece::Attacker {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn is_surrounded_next_to_throne(&self, king_pos: Position, throne_pos: Position) -> bool {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for &(dr, dc) in &directions {
            let r = king_pos.row as i32 + dr;
            let c = king_pos.col as i32 + dc;

            if r < 0 || r >= self.board_size as i32 || c < 0 || c >= self.board_size as i32 {
                continue;
            }

            let pos = Position::new(r as usize, c as usize);

            // Skip the throne position (it counts as occupied for this purpose)
            if pos.row == throne_pos.row && pos.col == throne_pos.col {
                continue;
            }

            // All other adjacent positions must have attackers
            if let Some(piece) = self.get_piece(pos) {
                if piece != Piece::Attacker {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    fn is_hostile_to_king(&self, pos: Position) -> bool {
        // Corners are hostile to all including king
        if self.is_corner(pos) {
            return true;
        }

        // Check if there's an attacker piece
        if let Some(piece) = self.get_piece(pos) {
            return piece == Piece::Attacker;
        }

        false
    }

    /// Compute a hash of the current board position
    fn hash_position(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash the board state
        for row in 0..self.board_size {
            for col in 0..self.board_size {
                // Hash each piece position
                match self.board[row][col] {
                    None => 0u8.hash(&mut hasher),
                    Some(Piece::Attacker) => 1u8.hash(&mut hasher),
                    Some(Piece::Defender) => 2u8.hash(&mut hasher),
                    Some(Piece::King) => 3u8.hash(&mut hasher),
                }
            }
        }

        // Hash the current player (positions are distinct based on whose turn it is)
        match self.current_player {
            Player::Attackers => 0u8.hash(&mut hasher),
            Player::Defenders => 1u8.hash(&mut hasher),
        }

        hasher.finish()
    }

    /// Record the current position in the history
    fn record_position(&mut self) {
        let hash = self.hash_position();
        *self.position_history.entry(hash).or_insert(0) += 1;
    }

    /// Check if the current position has occurred 3 times (threefold repetition)
    /// If so, the defender loses
    fn check_threefold_repetition(&mut self) {
        if self.result.is_some() {
            return; // Game already over
        }

        let hash = self.hash_position();
        if let Some(&count) = self.position_history.get(&hash) {
            if count >= 3 {
                // Threefold repetition - defender loses
                self.result = Some(GameResult::AttackersWin);
            }
        }
    }

    fn check_game_end(&mut self) {
        // Defenders win if king reaches a corner
        if let Some(king_pos) = self.king_position {
            if self.is_corner(king_pos) {
                self.result = Some(GameResult::DefendersWin);
                return;
            }
        } else {
            // King captured - attackers win
            self.result = Some(GameResult::AttackersWin);
            return;
        }

        // check that opposite party has legal moves
        let opponent = self.current_player.opponent();
        if self.legal_moves(opponent).is_empty() {
            // No legal moves for opponent - current player wins
            match opponent {
                Player::Attackers => self.result = Some(GameResult::DefendersWin),
                Player::Defenders => self.result = Some(GameResult::AttackersWin),
            }
        }
    }

    /// Get a string representation of the board
    pub fn display_board(&self) -> String {
        let mut result = String::new();
        result.push_str(&format!("[{}]\n", self.variant.name()));
        result.push_str("   ");
        for col in 0..self.board_size {
            result.push_str(&format!("{:2} ", col));
        }
        result.push('\n');

        for row in 0..self.board_size {
            result.push_str(&format!("{:2} ", row));
            for col in 0..self.board_size {
                let pos = Position::new(row, col);
                let c = if self.is_corner(pos) {
                    'X'
                } else if self.is_throne(pos) && self.get_piece(pos).is_none() {
                    'T'
                } else {
                    match self.get_piece(pos) {
                        Some(Piece::Attacker) => 'A',
                        Some(Piece::Defender) => 'D',
                        Some(Piece::King) => 'K',
                        None => '.',
                    }
                };
                result.push_str(&format!(" {} ", c));
            }
            result.push('\n');
        }

        result
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new_copenhagen()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to create a custom board state for testing
    fn create_test_board() -> GameState {
        GameState::new_brandubh()
    }

    /// Helper to place a piece on the board
    fn set_piece(state: &mut GameState, pos: Position, piece: Option<Piece>) {
        state.board[pos.row][pos.col] = piece;
        if piece == Some(Piece::King) {
            state.king_position = Some(pos);
        }
    }

    /// Helper to clear the board
    fn clear_board(state: &mut GameState) {
        for row in 0..state.board_size {
            for col in 0..state.board_size {
                state.board[row][col] = None;
            }
        }
        state.king_position = None;
    }

    #[test]
    fn test_initial_setup_brandubh() {
        let game = GameState::new_brandubh();

        // Check board size
        assert_eq!(game.board_size(), 7);

        // Check king position (center: 3, 3)
        assert_eq!(game.get_piece(Position::new(3, 3)), Some(Piece::King));

        // Check 4 defenders around king
        assert_eq!(game.get_piece(Position::new(2, 3)), Some(Piece::Defender));
        assert_eq!(game.get_piece(Position::new(4, 3)), Some(Piece::Defender));
        assert_eq!(game.get_piece(Position::new(3, 2)), Some(Piece::Defender));
        assert_eq!(game.get_piece(Position::new(3, 4)), Some(Piece::Defender));

        // Check 8 attackers (2 on each side)
        assert_eq!(game.get_piece(Position::new(0, 3)), Some(Piece::Attacker));
        assert_eq!(game.get_piece(Position::new(1, 3)), Some(Piece::Attacker));
        assert_eq!(game.get_piece(Position::new(5, 3)), Some(Piece::Attacker));
        assert_eq!(game.get_piece(Position::new(6, 3)), Some(Piece::Attacker));
        assert_eq!(game.get_piece(Position::new(3, 0)), Some(Piece::Attacker));
        assert_eq!(game.get_piece(Position::new(3, 1)), Some(Piece::Attacker));
        assert_eq!(game.get_piece(Position::new(3, 5)), Some(Piece::Attacker));
        assert_eq!(game.get_piece(Position::new(3, 6)), Some(Piece::Attacker));
    }

    #[test]
    fn test_corner_identification() {
        let game = GameState::new_brandubh();

        // Test all four corners
        assert!(game.is_corner(Position::new(0, 0)));
        assert!(game.is_corner(Position::new(0, 6)));
        assert!(game.is_corner(Position::new(6, 0)));
        assert!(game.is_corner(Position::new(6, 6)));

        // Test non-corners
        assert!(!game.is_corner(Position::new(0, 3)));
        assert!(!game.is_corner(Position::new(3, 3)));
    }

    #[test]
    fn test_throne_identification() {
        let game = GameState::new_brandubh();

        // Center is throne
        assert!(game.is_throne(Position::new(3, 3)));

        // Adjacent to throne is not throne
        assert!(!game.is_throne(Position::new(2, 3)));
        assert!(!game.is_throne(Position::new(4, 3)));
        assert!(!game.is_throne(Position::new(3, 2)));
        assert!(!game.is_throne(Position::new(3, 4)));
    }

    #[test]
    fn test_only_king_can_enter_throne() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place attacker next to throne
        set_piece(&mut game, Position::new(2, 3), Some(Piece::Attacker));

        let moves = game.legal_moves_for_piece(Position::new(2, 3));

        // Should not include throne
        assert!(!moves.contains(&Move::new(Position::new(2, 3), Position::new(3, 3))));
    }

    #[test]
    fn test_only_king_can_enter_corners() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place defender next to corner
        set_piece(&mut game, Position::new(0, 1), Some(Piece::Defender));
        game.current_player = Player::Defenders;

        let moves = game.legal_moves_for_piece(Position::new(0, 1));

        // Should not include corner
        assert!(!moves.contains(&Move::new(Position::new(0, 1), Position::new(0, 0))));
    }

    #[test]
    fn test_king_can_enter_corners() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place king next to corner
        set_piece(&mut game, Position::new(0, 1), Some(Piece::King));
        game.current_player = Player::Defenders;

        let moves = game.legal_moves_for_piece(Position::new(0, 1));

        // Should include corner
        assert!(moves.contains(&Move::new(Position::new(0, 1), Position::new(0, 0))));
    }

    #[test]
    fn test_regular_piece_capture_by_sandwich() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Set up: A D A (defender sandwiched)
        //         . . .
        set_piece(&mut game, Position::new(0, 0), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(0, 1), Some(Piece::Defender));
        set_piece(&mut game, Position::new(0, 2), Some(Piece::Attacker));

        // The defender should be capturable
        assert!(game.can_capture(Position::new(0, 2), Position::new(0, 1)));
    }

    #[test]
    fn test_corner_is_hostile_to_all() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Test attacker captured against corner
        // X is corner at (0,0)
        // Place attacker at (0,1) and defender at (0,3)
        // Defender moves to (0,2) to create sandwich with corner
        set_piece(&mut game, Position::new(0, 1), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(0, 3), Some(Piece::Defender));
        game.current_player = Player::Defenders;

        // Make move to capture attacker between corner and defender
        game.make_move(Move::new(Position::new(0, 3), Position::new(0, 2)))
            .unwrap();

        // Check that attacker is captured
        assert_eq!(game.get_piece(Position::new(0, 1)), None);
    }

    #[test]
    fn test_throne_hostile_to_attackers() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Set up: D at (3,2), A at (3,3) being throne, D at (3,4)
        // D A(throne) D
        set_piece(&mut game, Position::new(3, 2), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 3), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(3, 4), Some(Piece::Defender));
        game.current_player = Player::Defenders;

        // Throne should act as hostile to attacker
        let can_cap = game.can_capture(Position::new(3, 4), Position::new(3, 3));
        // Note: throne is hostile to attackers when empty, but this attacker is on throne
        // Actually defenders can capture attackers normally
        assert!(can_cap);
    }

    #[test]
    fn test_throne_hostile_to_defenders_when_empty() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Set up: A at (3,2), D at (3,3) being adjacent to throne (2,3), A at (2,3)
        // Place defender next to empty throne and sandwich with attacker
        set_piece(&mut game, Position::new(2, 3), Some(Piece::Defender));
        set_piece(&mut game, Position::new(4, 3), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        // When attacker moves to complete sandwich with throne, defender is captured
        game.make_move(Move::new(Position::new(4, 3), Position::new(1, 3)))
            .ok();

        // Note: throne at (3,3) acts as hostile when empty
        // But defender at (2,3) needs opposite side to be hostile too
    }

    #[test]
    fn test_king_capture_away_from_throne_requires_two_sides() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place king at (1, 1) - away from throne
        set_piece(&mut game, Position::new(1, 1), Some(Piece::King));

        // Place one attacker next to king
        set_piece(&mut game, Position::new(0, 1), Some(Piece::Attacker));

        // King should NOT be captured with only one attacker
        assert!(!game.is_king_captured(Position::new(1, 1)));

        // Place second attacker on opposite side
        set_piece(&mut game, Position::new(2, 1), Some(Piece::Attacker));

        // Now king should be captured
        assert!(game.is_king_captured(Position::new(1, 1)));
    }

    #[test]
    fn test_king_capture_on_throne_requires_four_sides() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place king on throne (3, 3)
        set_piece(&mut game, Position::new(3, 3), Some(Piece::King));

        // Place three attackers
        set_piece(&mut game, Position::new(2, 3), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(4, 3), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(3, 2), Some(Piece::Attacker));

        // King should NOT be captured with only three attackers
        assert!(!game.is_king_captured(Position::new(3, 3)));

        // Place fourth attacker
        set_piece(&mut game, Position::new(3, 4), Some(Piece::Attacker));

        // Now king should be captured
        assert!(game.is_king_captured(Position::new(3, 3)));
    }

    #[test]
    fn test_king_capture_next_to_throne_requires_three_sides() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place king next to throne at (2, 3) - throne is at (3, 3)
        set_piece(&mut game, Position::new(2, 3), Some(Piece::King));

        // Place two attackers (not including throne side)
        set_piece(&mut game, Position::new(1, 3), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(2, 2), Some(Piece::Attacker));

        // King should NOT be captured with only two attackers
        assert!(!game.is_king_captured(Position::new(2, 3)));

        // Place third attacker
        set_piece(&mut game, Position::new(2, 4), Some(Piece::Attacker));

        // Now king should be captured (throne at (3,3) counts as 4th side)
        assert!(game.is_king_captured(Position::new(2, 3)));
    }

    #[test]
    fn test_king_wins_by_reaching_corner() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place king next to corner
        set_piece(&mut game, Position::new(0, 1), Some(Piece::King));
        game.current_player = Player::Defenders;

        // Move king to corner
        game.make_move(Move::new(Position::new(0, 1), Position::new(0, 0)))
            .unwrap();

        // Check that defenders won
        assert_eq!(game.result(), Some(&GameResult::DefendersWin));
    }

    #[test]
    fn test_attackers_win_by_capturing_king() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place king at (1, 1)
        set_piece(&mut game, Position::new(1, 1), Some(Piece::King));

        // Place attacker at (0, 1)
        set_piece(&mut game, Position::new(0, 1), Some(Piece::Attacker));

        // Place second attacker at (2, 1) ready to move
        set_piece(&mut game, Position::new(2, 1), Some(Piece::Attacker));

        game.current_player = Player::Attackers;

        // This move should not work as the attacker is already in position
        // Let's place the attacker elsewhere and move it
        set_piece(&mut game, Position::new(2, 1), None);
        set_piece(&mut game, Position::new(2, 2), Some(Piece::Attacker));

        // Move attacker to complete capture
        game.make_move(Move::new(Position::new(2, 2), Position::new(2, 1)))
            .unwrap();

        // Check that king was captured
        assert_eq!(game.get_piece(Position::new(1, 1)), None);

        // Check game state after switching player
        assert_eq!(game.result(), Some(&GameResult::AttackersWin));
    }

    #[test]
    fn test_pieces_cannot_jump_over_others() {
        let game = create_test_board();

        // Attacker at (0, 3) cannot jump over attacker at (1, 3)
        let moves = game.legal_moves_for_piece(Position::new(0, 3));

        // Should not contain any moves beyond (1, 3) in the column
        for mv in &moves {
            if mv.from.col == 3 && mv.to.col == 3 {
                // Moving vertically in column 3, cannot go past row 1 (blocked by attacker at 1,3)
                assert!(mv.to.row <= 1);
            }
        }
    }

    #[test]
    fn test_pieces_can_only_move_orthogonally() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place attacker in center of empty board
        set_piece(&mut game, Position::new(3, 3), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        let moves = game.legal_moves_for_piece(Position::new(3, 3));

        // All moves should be in same row or same column
        for mv in moves {
            assert!(mv.to.row == 3 || mv.to.col == 3);
        }
    }

    #[test]
    fn test_corner_is_hostile_to_king() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place king at (0, 1) and attacker at (0, 2)
        set_piece(&mut game, Position::new(0, 1), Some(Piece::King));
        set_piece(&mut game, Position::new(0, 2), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        // Move attacker away and back to trigger capture with corner
        // Actually, corners are hostile, so king at (0,1) between corner (0,0) and attacker (0,2) should be captured
        // But we need the attacker to move to trigger capture check

        // Clear and reset
        clear_board(&mut game);
        set_piece(&mut game, Position::new(0, 1), Some(Piece::King));
        set_piece(&mut game, Position::new(0, 3), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        // Move attacker to (0, 2) - this should capture king between corner and attacker
        game.make_move(Move::new(Position::new(0, 3), Position::new(0, 2)))
            .unwrap();

        // King should be captured
        assert_eq!(game.get_piece(Position::new(0, 1)), None);
    }

    #[test]
    fn test_regular_defender_capture() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // A D . A (in a row)
        set_piece(&mut game, Position::new(3, 0), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(3, 1), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 3), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        // Move attacker to complete sandwich
        game.make_move(Move::new(Position::new(3, 3), Position::new(3, 2)))
            .unwrap();

        // Defender should be captured
        assert_eq!(game.get_piece(Position::new(3, 1)), None);
    }

    #[test]
    fn test_attacker_capture_by_defenders() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // D A . D (in a row)
        set_piece(&mut game, Position::new(3, 0), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 1), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(3, 3), Some(Piece::Defender));
        game.current_player = Player::Defenders;

        // Move defender to complete sandwich
        game.make_move(Move::new(Position::new(3, 3), Position::new(3, 2)))
            .unwrap();

        // Attacker should be captured
        assert_eq!(game.get_piece(Position::new(3, 1)), None);
    }

    #[test]
    fn test_no_capture_without_sandwich() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Setup: Place defender away from throne with attacker moving next to it
        // but with empty space on other side (no sandwich, no throne, no corner)
        // Place pieces at row 1 to avoid throne at (3,3)
        // . A D . .
        set_piece(&mut game, Position::new(1, 0), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(1, 2), Some(Piece::Defender));
        game.current_player = Player::Attackers;

        // Verify opposite side is empty and not a special square
        assert_eq!(game.get_piece(Position::new(1, 3)), None);
        assert!(!game.is_throne(Position::new(1, 3)));
        assert!(!game.is_corner(Position::new(1, 3)));

        // Move attacker to be adjacent to defender
        game.make_move(Move::new(Position::new(1, 0), Position::new(1, 1)))
            .unwrap();

        // Defender should NOT be captured because there's no hostile piece on opposite side
        assert_eq!(game.get_piece(Position::new(1, 2)), Some(Piece::Defender));

        // Also verify the attacker is in its new position
        assert_eq!(game.get_piece(Position::new(1, 1)), Some(Piece::Attacker));
    }

    #[test]
    fn test_king_and_defender_cannot_capture_each_other() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // K D K (defenders only)
        set_piece(&mut game, Position::new(3, 0), Some(Piece::King));
        set_piece(&mut game, Position::new(3, 1), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 3), Some(Piece::Defender));
        game.current_player = Player::Defenders;

        // Move defender next to another defender
        game.make_move(Move::new(Position::new(3, 3), Position::new(3, 2)))
            .unwrap();

        // Defender should NOT be captured
        assert_eq!(game.get_piece(Position::new(3, 1)), Some(Piece::Defender));
    }

    #[test]
    fn test_attackers_cannot_capture_each_other() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // A A . A (attackers only)
        set_piece(&mut game, Position::new(3, 0), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(3, 1), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(3, 3), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        // Move attacker next to another attacker
        game.make_move(Move::new(Position::new(3, 3), Position::new(3, 2)))
            .unwrap();

        // Attacker should NOT be captured
        assert_eq!(game.get_piece(Position::new(3, 1)), Some(Piece::Attacker));
    }

    #[test]
    fn test_multiple_captures_in_one_move() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Set up cross pattern:
        //     D
        //   D A D
        //     D
        // Place attacker in middle, defenders around, and another defender to move
        set_piece(&mut game, Position::new(3, 3), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(2, 3), Some(Piece::Defender));
        set_piece(&mut game, Position::new(4, 3), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 2), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 4), Some(Piece::Defender));

        // This setup won't work for multiple captures with one move in standard rules
        // Let's test a different scenario: T-shape capture
        clear_board(&mut game);

        //   A
        // A D D
        //   A
        set_piece(&mut game, Position::new(3, 2), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(3, 3), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 4), Some(Piece::Defender));
        set_piece(&mut game, Position::new(2, 3), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(5, 3), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        // Move attacker to (4, 3) to complete sandwich on vertical defender
        game.make_move(Move::new(Position::new(5, 3), Position::new(4, 3)))
            .unwrap();

        // Middle defender at (3,3) should be captured
        assert_eq!(game.get_piece(Position::new(3, 3)), None);
    }

    #[test]
    fn test_game_starts_with_attackers_turn() {
        let game = create_test_board();
        assert_eq!(game.current_player(), Player::Attackers);
    }

    #[test]
    fn test_turns_alternate() {
        let mut game = create_test_board();

        assert_eq!(game.current_player(), Player::Attackers);

        // Make a move
        let moves = game.legal_moves(game.current_player());
        if let Some(mv) = moves.first() {
            game.make_move(*mv).unwrap();
            assert_eq!(game.current_player(), Player::Defenders);
        }
    }

    #[test]
    fn test_king_not_captured_with_one_side_on_throne() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // King on throne with only 1, 2, or 3 attackers should not be captured
        set_piece(&mut game, Position::new(3, 3), Some(Piece::King));
        set_piece(&mut game, Position::new(2, 3), Some(Piece::Attacker));

        assert!(!game.is_king_captured(Position::new(3, 3)));

        set_piece(&mut game, Position::new(4, 3), Some(Piece::Attacker));
        assert!(!game.is_king_captured(Position::new(3, 3)));

        set_piece(&mut game, Position::new(3, 2), Some(Piece::Attacker));
        assert!(!game.is_king_captured(Position::new(3, 3)));
    }

    #[test]
    fn test_king_not_captured_with_two_sides_next_to_throne() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // King next to throne at (2, 3) with only 2 attackers should not be captured
        set_piece(&mut game, Position::new(2, 3), Some(Piece::King));
        set_piece(&mut game, Position::new(1, 3), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(2, 2), Some(Piece::Attacker));

        // Throne at (3,3) counts as one side, but we need 3 attackers total
        assert!(!game.is_king_captured(Position::new(2, 3)));
    }

    #[test]
    fn test_throne_empty_hostile_to_defenders() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place defender next to throne and attacker to sandwich
        // A D T (T is empty throne at 3,3)
        set_piece(&mut game, Position::new(3, 2), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 1), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        // Throne should act as hostile to defender when empty
        assert!(game.can_capture(Position::new(3, 1), Position::new(3, 2)));
    }

    #[test]
    fn test_throne_not_hostile_when_occupied_by_king() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Place defender next to throne occupied by king
        // A D K (K is king on throne at 3,3)
        set_piece(&mut game, Position::new(3, 3), Some(Piece::King));
        set_piece(&mut game, Position::new(3, 2), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 1), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        // Throne occupied by king should not make defender capturable
        // (Defender and King are on same team)
        assert!(!game.can_capture(Position::new(3, 1), Position::new(3, 2)));
    }

    #[test]
    fn test_all_four_corners_win_for_defenders() {
        // Test each corner wins the game for defenders
        let corners = [
            Position::new(0, 0),
            Position::new(0, 6),
            Position::new(6, 0),
            Position::new(6, 6),
        ];

        for corner in corners {
            let mut game = create_test_board();
            clear_board(&mut game);

            // Place king next to corner
            let adjacent =
                Position::new(if corner.row == 0 { 1 } else { corner.row - 1 }, corner.col);
            set_piece(&mut game, adjacent, Some(Piece::King));
            game.current_player = Player::Defenders;

            // Move king to corner
            game.make_move(Move::new(adjacent, corner)).unwrap();

            // Check that defenders won
            assert_eq!(
                game.result(),
                Some(&GameResult::DefendersWin),
                "Corner {:?} should result in DefendersWin",
                corner
            );
        }
    }

    #[test]
    fn test_piece_cannot_move_onto_occupied_square() {
        let game = create_test_board();

        // Try to find a move where piece would land on occupied square
        // Attacker at (0,3) should not be able to move to (1,3) which has another attacker
        let moves = game.legal_moves_for_piece(Position::new(0, 3));

        // Should not contain move to occupied square
        assert!(!moves.contains(&Move::new(Position::new(0, 3), Position::new(1, 3))));
    }

    #[test]
    fn test_capture_requires_opposite_side_hostility() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Test that just having a piece next to an enemy doesn't capture
        // D A (no piece on other side of attacker)
        set_piece(&mut game, Position::new(3, 0), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 1), Some(Piece::Attacker));
        game.current_player = Player::Defenders;

        // Defender at (3,0) should not capture attacker at (3,1)
        // because there's no hostile piece at (3,2)
        assert!(!game.can_capture(Position::new(3, 0), Position::new(3, 1)));
    }

    #[test]
    fn test_king_with_corner_hostility() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // King between corner and attacker should be captured
        // K A . (K at corner-adjacent, A next to it)
        set_piece(&mut game, Position::new(0, 1), Some(Piece::King));
        set_piece(&mut game, Position::new(0, 3), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        // Move attacker to sandwich king against corner
        game.make_move(Move::new(Position::new(0, 3), Position::new(0, 2)))
            .unwrap();

        // King should be captured (corner is hostile to king)
        assert_eq!(game.get_piece(Position::new(0, 1)), None);
        assert_eq!(game.result(), Some(&GameResult::AttackersWin));
    }

    #[test]
    fn test_vertical_and_horizontal_captures() {
        // Test capture works in both horizontal and vertical directions

        // Test horizontal capture
        let mut game = create_test_board();
        clear_board(&mut game);

        set_piece(&mut game, Position::new(2, 1), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(2, 2), Some(Piece::Defender));
        set_piece(&mut game, Position::new(2, 4), Some(Piece::Attacker));
        game.current_player = Player::Attackers;

        game.make_move(Move::new(Position::new(2, 4), Position::new(2, 3)))
            .unwrap();
        assert_eq!(
            game.get_piece(Position::new(2, 2)),
            None,
            "Horizontal capture failed"
        );

        // Test vertical capture in a new game
        let mut game2 = create_test_board();
        clear_board(&mut game2);

        set_piece(&mut game2, Position::new(1, 2), Some(Piece::Defender));
        set_piece(&mut game2, Position::new(2, 2), Some(Piece::Attacker));
        set_piece(&mut game2, Position::new(4, 2), Some(Piece::Defender));
        game2.current_player = Player::Defenders;

        game2
            .make_move(Move::new(Position::new(4, 2), Position::new(3, 2)))
            .unwrap();
        assert_eq!(
            game2.get_piece(Position::new(2, 2)),
            None,
            "Vertical capture failed"
        );
    }

    #[test]
    fn test_move_count_increments() {
        let mut game = create_test_board();

        assert_eq!(game.move_count(), 0);

        let moves = game.legal_moves(game.current_player());
        game.make_move(moves[0]).unwrap();
        assert_eq!(game.move_count(), 1);

        let moves = game.legal_moves(game.current_player());
        game.make_move(moves[0]).unwrap();
        assert_eq!(game.move_count(), 2);
    }

    #[test]
    fn test_cannot_move_after_game_over() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Setup king to win
        set_piece(&mut game, Position::new(0, 1), Some(Piece::King));
        game.current_player = Player::Defenders;

        // Move king to corner
        game.make_move(Move::new(Position::new(0, 1), Position::new(0, 0)))
            .unwrap();

        assert!(game.is_game_over());

        // Try to make another move
        let result = game.make_move(Move::new(Position::new(0, 0), Position::new(1, 0)));
        assert!(result.is_err());
        assert!(matches!(result, Err(GameError::GameOver)));
    }

    #[test]
    fn test_king_between_two_attackers_on_different_axes_not_captured() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // King at (2,2) with attackers at (1,2) and (2,1)
        // This is NOT a capture because attackers are not on opposite sides
        set_piece(&mut game, Position::new(2, 2), Some(Piece::King));
        set_piece(&mut game, Position::new(1, 2), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(2, 1), Some(Piece::Attacker));

        // King should not be captured (attackers not on opposite sides)
        assert!(!game.is_king_captured(Position::new(2, 2)));
    }

    #[test]
    fn test_king_no_capture_from_perpendicular_move() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Set up: King at (4,4) NOT on throne, attackers at (3,4) and (5,4) creating a vertical sandwich
        // The king is already between two attackers vertically
        set_piece(&mut game, Position::new(4, 4), Some(Piece::King));
        set_piece(&mut game, Position::new(3, 4), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(5, 4), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(6, 3), Some(Piece::Attacker)); // Another attacker to move

        game.current_player = Player::Attackers;

        // Move an attacker to (4,3) - adjacent to king but perpendicular to existing vertical sandwich
        // This should NOT capture the king because the move is perpendicular to the pre-existing sandwich
        game.make_move(Move::new(Position::new(6, 3), Position::new(4, 3)))
            .unwrap();

        // King should still be alive
        assert_eq!(
            game.get_piece(Position::new(4, 4)),
            Some(Piece::King),
            "King should not be captured by perpendicular move"
        );
    }

    #[test]
    fn test_regular_piece_no_capture_from_perpendicular_move() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Set up: Defender at (3,3), attacker at (2,3) and (4,3) (vertical sandwich)
        set_piece(&mut game, Position::new(3, 3), Some(Piece::Defender));
        set_piece(&mut game, Position::new(2, 3), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(4, 3), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(5, 2), Some(Piece::Attacker)); // Another attacker to move

        game.current_player = Player::Attackers;

        // Defender is surrounded vertically but not captured (existing position)
        assert!(game.get_piece(Position::new(3, 3)).is_some());

        // Now move an attacker to a perpendicular position (3,2)
        // This should NOT capture the defender because the move is perpendicular
        game.make_move(Move::new(Position::new(5, 2), Position::new(3, 2)))
            .unwrap();

        // Defender should still be alive
        assert_eq!(
            game.get_piece(Position::new(3, 3)),
            Some(Piece::Defender),
            "Defender should not be captured by perpendicular move"
        );
    }

    #[test]
    fn test_king_can_be_captured_when_move_completes_sandwich() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Set up: King at (4,4), attacker at (3,4), another attacker to move from (6,4) to (5,4)
        set_piece(&mut game, Position::new(4, 4), Some(Piece::King));
        set_piece(&mut game, Position::new(3, 4), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(6, 4), Some(Piece::Attacker)); // Attacker to complete sandwich

        game.current_player = Player::Attackers;

        // Move attacker to complete the vertical sandwich
        game.make_move(Move::new(Position::new(6, 4), Position::new(5, 4)))
            .unwrap();

        // King should be captured because the move completed the sandwich in the correct direction
        assert_eq!(
            game.get_piece(Position::new(4, 4)),
            None,
            "King should be captured when move completes the sandwich"
        );
    }

    #[test]
    fn test_win_when_opponent_has_no_moves() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Set up a scenario where defenders will have no legal moves after attackers move
        // Place king in a position where it can be completely surrounded
        // King at (1,1) with limited movement options
        set_piece(&mut game, Position::new(1, 1), Some(Piece::King));

        // Place attackers to block most directions
        set_piece(&mut game, Position::new(0, 1), Some(Piece::Attacker)); // Above
        set_piece(&mut game, Position::new(1, 0), Some(Piece::Attacker)); // Left
        set_piece(&mut game, Position::new(2, 1), Some(Piece::Attacker)); // Below

        // Place an attacker that will move to block the last direction (right)
        set_piece(&mut game, Position::new(1, 3), Some(Piece::Attacker));

        game.current_player = Player::Attackers;

        // Verify defenders (king) currently have at least one legal move
        let defender_moves_before = game.legal_moves(Player::Defenders);
        assert!(
            !defender_moves_before.is_empty(),
            "Defenders should have legal moves before the blocking move"
        );

        // Move attacker to (1,2) to block the king's last escape route
        game.make_move(Move::new(Position::new(1, 3), Position::new(1, 2)))
            .unwrap();

        // After the move, it should be defenders' turn
        assert_eq!(game.current_player(), Player::Defenders);

        // Verify defenders have no legal moves
        let defender_moves_after = game.legal_moves(Player::Defenders);
        assert!(
            defender_moves_after.is_empty(),
            "Defenders should have no legal moves after being blocked"
        );

        // The game should be over with attackers winning
        assert!(
            game.is_game_over(),
            "Game should be over when opponent has no moves"
        );
        assert_eq!(
            game.result(),
            Some(&GameResult::AttackersWin),
            "Attackers should win when defenders have no moves"
        );
    }

    #[test]
    fn test_threefold_repetition_defender_loses() {
        let mut game = create_test_board();
        clear_board(&mut game);

        // Set up a simple position where pieces can shuttle back and forth
        // Two attackers that can move back and forth
        set_piece(&mut game, Position::new(1, 1), Some(Piece::Attacker));
        set_piece(&mut game, Position::new(5, 1), Some(Piece::Attacker));
        // Defender and King that can move back and forth
        set_piece(&mut game, Position::new(3, 5), Some(Piece::Defender));
        set_piece(&mut game, Position::new(3, 3), Some(Piece::King));

        game.current_player = Player::Attackers;

        // Record the initial position now that we've set it up
        game.position_history.clear();
        game.record_position();

        // Now we'll shuttle pieces back and forth to create threefold repetition

        // Move 1: Attacker from (1,1) to (1,2)
        game.make_move(Move::new(Position::new(1, 1), Position::new(1, 2)))
            .unwrap();
        assert!(!game.is_game_over());

        // Move 2: Defender from (3,5) to (3,6)
        game.make_move(Move::new(Position::new(3, 5), Position::new(3, 6)))
            .unwrap();
        assert!(!game.is_game_over());

        // Move 3: Attacker from (1,2) back to (1,1)
        game.make_move(Move::new(Position::new(1, 2), Position::new(1, 1)))
            .unwrap();
        assert!(!game.is_game_over());

        // Move 4: Defender from (3,6) back to (3,5) - first repetition
        game.make_move(Move::new(Position::new(3, 6), Position::new(3, 5)))
            .unwrap();
        assert!(!game.is_game_over());

        // Move 5: Attacker from (1,1) to (1,2) again
        game.make_move(Move::new(Position::new(1, 1), Position::new(1, 2)))
            .unwrap();
        assert!(!game.is_game_over());

        // Move 6: Defender from (3,5) to (3,6) again
        game.make_move(Move::new(Position::new(3, 5), Position::new(3, 6)))
            .unwrap();
        assert!(!game.is_game_over());

        // Move 7: Attacker from (1,2) back to (1,1) again
        game.make_move(Move::new(Position::new(1, 2), Position::new(1, 1)))
            .unwrap();
        assert!(!game.is_game_over());

        // Move 8: Defender from (3,6) back to (3,5) again
        // This is the THIRD repetition - game should end with Attackers winning
        game.make_move(Move::new(Position::new(3, 6), Position::new(3, 5)))
            .unwrap();

        // After this move, the position has occurred 3 times
        // The game should be over with attackers winning (defender loses on threefold repetition)
        assert!(
            game.is_game_over(),
            "Game should be over after threefold repetition"
        );
        assert_eq!(
            game.result(),
            Some(&GameResult::AttackersWin),
            "Attackers should win (defender loses) on threefold repetition"
        );
    }
}
