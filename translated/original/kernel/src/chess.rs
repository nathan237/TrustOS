//! TrustChess — Chess game engine for TrustOS
//!
//! Complete chess implementation with:
//! - Full piece movement rules (including castling, en passant, promotion)
//! - Check/checkmate/stalemate detection
//! - Simple AI (material evaluation + minimax depth 3)
//! - Graphical board rendering for the desktop

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

// ═══════════════════════════════════════════════════════════════
// PIECE CONSTANTS — encoded as i8: sign = color, abs = piece type
// Positive = White, Negative = Black
// ═══════════════════════════════════════════════════════════════
pub const EMPTY: i8 = 0;
pub const W_PAWN: i8 = 1;
pub const W_KNIGHT: i8 = 2;
pub const W_BISHOP: i8 = 3;
pub const W_ROOK: i8 = 4;
pub const W_QUEEN: i8 = 5;
pub const W_KING: i8 = 6;
pub const B_PAWN: i8 = -1;
pub const B_KNIGHT: i8 = -2;
pub const B_BISHOP: i8 = -3;
pub const B_ROOK: i8 = -4;
pub const B_QUEEN: i8 = -5;
pub const B_KING: i8 = -6;

/// Returns true if piece is white
fn is_white(p: i8) -> bool { p > 0 }
/// Returns true if piece is black  
fn is_black(p: i8) -> bool { p < 0 }
/// Get the type of piece (1-6) regardless of color
fn piece_type(p: i8) -> i8 { if p < 0 { -p } else { p } }

/// Piece value for AI evaluation
fn piece_value(p: i8) -> i32 {
    match piece_type(p) {
        1 => 100,   // Pawn
        2 => 320,   // Knight
        3 => 330,   // Bishop
        4 => 500,   // Rook
        5 => 900,   // Queen
        6 => 20000, // King
        _ => 0,
    }
}

/// Unicode piece characters for rendering
pub fn piece_char(p: i8) -> char {
    match p {
        1  => '\u{2659}', // ♙ White Pawn
        2  => '\u{2658}', // ♘ White Knight
        3  => '\u{2657}', // ♗ White Bishop
        4  => '\u{2656}', // ♖ White Rook
        5  => '\u{2655}', // ♕ White Queen
        6  => '\u{2654}', // ♔ White King
        -1 => '\u{265F}', // ♟ Black Pawn
        -2 => '\u{265E}', // ♞ Black Knight
        -3 => '\u{265D}', // ♝ Black Bishop
        -4 => '\u{265C}', // ♜ Black Rook
        -5 => '\u{265B}', // ♛ Black Queen
        -6 => '\u{265A}', // ♚ Black King
        _  => ' ',
    }
}

/// Simple ASCII piece chars (fallback for 8x8 font)
pub fn piece_ascii(p: i8) -> char {
    match p {
        1  => 'P', 2  => 'N', 3  => 'B', 4  => 'R', 5  => 'Q', 6  => 'K',
        -1 => 'p', -2 => 'n', -3 => 'b', -4 => 'r', -5 => 'q', -6 => 'k',
        _  => ' ',
    }
}

/// Color for piece rendering
pub fn piece_color(p: i8) -> u32 {
    if is_white(p) { 0xFFFFFFFF } else { 0xFF1A1A1A }
}

// ═══════════════════════════════════════════════════════════════
// CHESS STATE
// ═══════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq)]
pub enum GamePhase {
    Playing,
    Check,
    Checkmate,
    Stalemate,
    Promotion,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputMode {
    SelectPiece,   // Player selects a piece to move
    SelectTarget,  // Player selects destination square
}

pub struct ChessState {
    pub board: [i8; 64],
    pub white_turn: bool,
    pub selected: Option<usize>,     // Currently selected square
    pub cursor: usize,               // Cursor position (0-63)
    pub valid_moves: Vec<usize>,     // Valid target squares for selected piece
    pub phase: GamePhase,
    pub input_mode: InputMode,
    pub message: String,
    pub move_history: Vec<String>,
    // Castling rights
    pub white_king_moved: bool,
    pub black_king_moved: bool,
    pub white_rook_a_moved: bool,
    pub white_rook_h_moved: bool,
    pub black_rook_a_moved: bool,
    pub black_rook_h_moved: bool,
    // En passant
    pub en_passant_target: Option<usize>, // Square where en passant capture is possible
    // Promotion
    pub promotion_square: Option<usize>,
    // AI
    pub vs_ai: bool,
    pub ai_thinking: bool,
    pub ai_depth: i32,
    pub rng_state: u32,
    // Last move highlight
    pub last_move_from: Option<usize>,
    pub last_move_to: Option<usize>,
    // Mouse drag & drop
    pub drag_from: Option<usize>,     // Square being dragged from
    pub dragging_piece: Option<i8>,   // Piece being dragged
    pub drag_pixel_x: i32,           // Current mouse X during drag
    pub drag_pixel_y: i32,           // Current mouse Y during drag
    // Timer
    pub white_time_ms: u64,
    pub black_time_ms: u64,
    pub timer_enabled: bool,
    pub timer_initial_ms: u64,       // Initial time per player (e.g. 600_000 = 10min)
    pub last_tick_ms: u64,           // Last tick timestamp
    pub timer_started: bool,         // Timer starts after first move
}

impl ChessState {
    pub fn new() -> Self {
        let mut state = ChessState {
            board: [EMPTY; 64],
            white_turn: true,
            selected: None,
            cursor: 52, // e2 — common first move
            valid_moves: Vec::new(),
            phase: GamePhase::Playing,
            input_mode: InputMode::SelectPiece,
            message: String::from("White to move"),
            move_history: Vec::new(),
            white_king_moved: false,
            black_king_moved: false,
            white_rook_a_moved: false,
            white_rook_h_moved: false,
            black_rook_a_moved: false,
            black_rook_h_moved: false,
            en_passant_target: None,
            promotion_square: None,
            vs_ai: true,
            ai_thinking: false,
            ai_depth: 2,
            rng_state: 12345,
            last_move_from: None,
            last_move_to: None,
            // Mouse drag & drop
            drag_from: None,
            dragging_piece: None,
            drag_pixel_x: 0,
            drag_pixel_y: 0,
            // Timer
            white_time_ms: 600_000,  // 10 minutes default
            black_time_ms: 600_000,
            timer_enabled: false,
            timer_initial_ms: 600_000,
            last_tick_ms: 0,
            timer_started: false,
        };
        state.init_board();
        state
    }

    fn init_board(&mut self) {
        // Black pieces (row 0 = top = rank 8)
        self.board[0] = B_ROOK;
        self.board[1] = B_KNIGHT;
        self.board[2] = B_BISHOP;
        self.board[3] = B_QUEEN;
        self.board[4] = B_KING;
        self.board[5] = B_BISHOP;
        self.board[6] = B_KNIGHT;
        self.board[7] = B_ROOK;
        for i in 8..16 {
            self.board[i] = B_PAWN;
        }
        // White pieces (row 7 = bottom = rank 1)
        for i in 48..56 {
            self.board[i] = W_PAWN;
        }
        self.board[56] = W_ROOK;
        self.board[57] = W_KNIGHT;
        self.board[58] = W_BISHOP;
        self.board[59] = W_QUEEN;
        self.board[60] = W_KING;
        self.board[61] = W_BISHOP;
        self.board[62] = W_KNIGHT;
        self.board[63] = W_ROOK;
    }

    // ── Coordinate helpers ──

    fn row(sq: usize) -> usize { sq / 8 }
    fn col(sq: usize) -> usize { sq % 8 }
    fn sq(row: usize, col: usize) -> usize { row * 8 + col }

    fn square_name(sq: usize) -> String {
        let file = (b'a' + Self::col(sq) as u8) as char;
        let rank = (b'1' + (7 - Self::row(sq)) as u8) as char;
        format!("{}{}", file, rank)
    }

    // ── Input handling ──

    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT};

        if self.phase == GamePhase::Checkmate || self.phase == GamePhase::Stalemate {
            if key == b'r' || key == b'R' {
                *self = ChessState::new();
            }
            return;
        }

        // Promotion selection
        if self.phase == GamePhase::Promotion {
            if let Some(sq) = self.promotion_square {
                let color_sign: i8 = if Self::row(sq) == 0 { 1 } else { -1 };
                match key {
                    b'q' | b'Q' | b'1' => { self.board[sq] = W_QUEEN * color_sign; self.finish_promotion(); }
                    b'r' | b'R' | b'2' => { self.board[sq] = W_ROOK * color_sign; self.finish_promotion(); }
                    b'b' | b'B' | b'3' => { self.board[sq] = W_BISHOP * color_sign; self.finish_promotion(); }
                    b'n' | b'N' | b'4' => { self.board[sq] = W_KNIGHT * color_sign; self.finish_promotion(); }
                    _ => {}
                }
            }
            return;
        }

        match key {
            KEY_UP    => { if self.cursor >= 8 { self.cursor -= 8; } },
            KEY_DOWN  => { if self.cursor < 56 { self.cursor += 8; } },
            KEY_LEFT  => { if self.cursor % 8 > 0 { self.cursor -= 1; } },
            KEY_RIGHT => { if self.cursor % 8 < 7 { self.cursor += 1; } },
            0x0D | b' ' => { // Enter or Space
                self.handle_select();
            },
            0x1B => { // Escape — deselect
                self.selected = None;
                self.valid_moves.clear();
                self.input_mode = InputMode::SelectPiece;
                self.message = if self.white_turn {
                    String::from("White to move")
                } else {
                    String::from("Black to move")
                };
            },
            b'r' | b'R' => {
                *self = ChessState::new();
            },
            b't' | b'T' => {
                self.toggle_timer();
            },
            b'+' | b'=' => {
                if self.timer_enabled {
                    self.cycle_timer_preset();
                }
            },
            b'd' | b'D' => {
                // Cycle AI difficulty: 1 → 2 → 3 → 1
                self.ai_depth = match self.ai_depth {
                    1 => 2,
                    2 => 3,
                    _ => 1,
                };
                self.message = match self.ai_depth {
                    1 => String::from("AI: Easy (depth 1)"),
                    2 => String::from("AI: Medium (depth 2)"),
                    _ => String::from("AI: Hard (depth 3)"),
                };
            },
            _ => {}
        }
    }

    fn handle_select(&mut self) {
        match self.input_mode {
            InputMode::SelectPiece => {
                let piece = self.board[self.cursor];
                if piece == EMPTY { return; }
                // Check if it's the current player's piece
                if self.white_turn && !is_white(piece) { 
                    self.message = String::from("Select a white piece");
                    return; 
                }
                if !self.white_turn && !is_black(piece) { 
                    self.message = String::from("Select a black piece");
                    return; 
                }
                // Compute valid moves
                self.valid_moves = self.get_legal_moves(self.cursor);
                if self.valid_moves.is_empty() {
                    self.message = String::from("No legal moves for this piece");
                    return;
                }
                self.selected = Some(self.cursor);
                self.input_mode = InputMode::SelectTarget;
                let name = Self::square_name(self.cursor);
                self.message = format!("Move {} from {}", piece_ascii(piece), name);
            },
            InputMode::SelectTarget => {
                let from = match self.selected {
                    Some(sq) => sq,
                    None => { self.input_mode = InputMode::SelectPiece; return; }
                };
                
                // If clicking own piece, reselect
                let target_piece = self.board[self.cursor];
                if target_piece != EMPTY {
                    let is_own = (self.white_turn && is_white(target_piece)) || 
                                 (!self.white_turn && is_black(target_piece));
                    if is_own {
                        // Reselect this piece instead
                        self.valid_moves = self.get_legal_moves(self.cursor);
                        if !self.valid_moves.is_empty() {
                            self.selected = Some(self.cursor);
                            let name = Self::square_name(self.cursor);
                            self.message = format!("Move {} from {}", piece_ascii(target_piece), name);
                        }
                        return;
                    }
                }

                if !self.valid_moves.contains(&self.cursor) {
                    self.message = String::from("Invalid move");
                    return;
                }

                // Execute the move
                self.make_move(from, self.cursor);
            },
        }
    }

    fn make_move(&mut self, from: usize, to: usize) {
        let piece = self.board[from];
        let captured = self.board[to];
        
        // Record move
        let move_str = format!("{}{}{}", piece_ascii(piece), Self::square_name(from), Self::square_name(to));
        self.move_history.push(move_str);
        
        // Update last move highlight
        self.last_move_from = Some(from);
        self.last_move_to = Some(to);

        // Handle en passant capture
        if piece_type(piece) == 1 && Some(to) == self.en_passant_target {
            // Remove the captured pawn
            if is_white(piece) {
                self.board[to + 8] = EMPTY; // Black pawn was below
            } else {
                self.board[to - 8] = EMPTY; // White pawn was above
            }
        }

        // Handle castling
        if piece_type(piece) == 6 {
            let col_diff = Self::col(to) as i32 - Self::col(from) as i32;
            if col_diff == 2 {
                // Kingside castle
                self.board[to - 1] = self.board[to + 1]; // Move rook
                self.board[to + 1] = EMPTY;
            } else if col_diff == -2 {
                // Queenside castle
                self.board[to + 1] = self.board[to - 2]; // Move rook
                self.board[to - 2] = EMPTY;
            }
        }

        // Move the piece
        self.board[to] = piece;
        self.board[from] = EMPTY;

        // Update castling rights
        if piece_type(piece) == 6 {
            if is_white(piece) { self.white_king_moved = true; }
            else { self.black_king_moved = true; }
        }
        if piece_type(piece) == 4 {
            if from == 56 { self.white_rook_a_moved = true; }
            if from == 63 { self.white_rook_h_moved = true; }
            if from == 0 { self.black_rook_a_moved = true; }
            if from == 7 { self.black_rook_h_moved = true; }
        }

        // En passant target
        self.en_passant_target = None;
        if piece_type(piece) == 1 {
            let row_diff = Self::row(to) as i32 - Self::row(from) as i32;
            if row_diff.abs() == 2 {
                // Double pawn push — set en passant target
                self.en_passant_target = Some(((from as i32 + to as i32) / 2) as usize);
            }
        }

        // Handle promotion
        if piece_type(piece) == 1 && (Self::row(to) == 0 || Self::row(to) == 7) {
            self.phase = GamePhase::Promotion;
            self.promotion_square = Some(to);
            self.message = String::from("Promote: Q/R/B/N");
            self.selected = None;
            self.valid_moves.clear();
            self.input_mode = InputMode::SelectPiece;
            return;
        }

        self.finish_turn();
    }

    fn finish_promotion(&mut self) {
        self.promotion_square = None;
        self.finish_turn();
    }

    fn finish_turn(&mut self) {
        self.white_turn = !self.white_turn;
        self.selected = None;
        self.valid_moves.clear();
        self.input_mode = InputMode::SelectPiece;

        // Start timer after first move
        if self.timer_enabled && !self.timer_started {
            self.timer_started = true;
        }

        // Check game state
        let in_check = self.is_in_check(self.white_turn);
        let has_moves = self.has_any_legal_moves(self.white_turn);

        if in_check && !has_moves {
            self.phase = GamePhase::Checkmate;
            self.message = if self.white_turn {
                String::from("Checkmate! Black wins!")
            } else {
                String::from("Checkmate! White wins!")
            };
        } else if !in_check && !has_moves {
            self.phase = GamePhase::Stalemate;
            self.message = String::from("Stalemate — Draw!");
        } else if in_check {
            self.phase = GamePhase::Check;
            self.message = if self.white_turn {
                String::from("White in check!")
            } else {
                String::from("Black in check!")
            };
        } else {
            self.phase = GamePhase::Playing;
            self.message = if self.white_turn {
                String::from("White to move")
            } else {
                String::from("Black to move")
            };
        }

        // AI turn
        if self.vs_ai && !self.white_turn && self.phase == GamePhase::Playing || 
           (self.vs_ai && !self.white_turn && self.phase == GamePhase::Check) {
            self.ai_move();
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // MOVE GENERATION
    // ═══════════════════════════════════════════════════════════════

    /// Get all pseudo-legal moves for a piece at `sq`
    fn get_moves(&self, sq: usize) -> Vec<usize> {
        let piece = self.board[sq];
        if piece == EMPTY { return Vec::new(); }
        let is_w = is_white(piece);
        let mut moves = Vec::new();
        let row = Self::row(sq);
        let col = Self::col(sq);

        match piece_type(piece) {
            1 => { // Pawn
                let dir: i32 = if is_w { -1 } else { 1 };
                let start_row = if is_w { 6 } else { 1 };
                
                // Forward one
                let fwd = sq as i32 + dir * 8;
                if fwd >= 0 && fwd < 64 && self.board[fwd as usize] == EMPTY {
                    moves.push(fwd as usize);
                    // Forward two from start
                    if row == start_row {
                        let fwd2 = sq as i32 + dir * 16;
                        if fwd2 >= 0 && fwd2 < 64 && self.board[fwd2 as usize] == EMPTY {
                            moves.push(fwd2 as usize);
                        }
                    }
                }
                // Diagonal captures
                for dc in [-1i32, 1] {
                    let nc = col as i32 + dc;
                    if nc >= 0 && nc < 8 {
                        let target = (row as i32 + dir) * 8 + nc;
                        if target >= 0 && target < 64 {
                            let t = target as usize;
                            let tp = self.board[t];
                            if (tp != EMPTY && is_white(tp) != is_w) || Some(t) == self.en_passant_target {
                                moves.push(t);
                            }
                        }
                    }
                }
            },
            2 => { // Knight
                let offsets: [(i32, i32); 8] = [
                    (-2,-1),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)
                ];
                for (dr, dc) in offsets {
                    let nr = row as i32 + dr;
                    let nc = col as i32 + dc;
                    if nr >= 0 && nr < 8 && nc >= 0 && nc < 8 {
                        let t = Self::sq(nr as usize, nc as usize);
                        let tp = self.board[t];
                        if tp == EMPTY || is_white(tp) != is_w {
                            moves.push(t);
                        }
                    }
                }
            },
            3 => { // Bishop
                self.slider_moves(sq, is_w, &[(-1,-1),(-1,1),(1,-1),(1,1)], &mut moves);
            },
            4 => { // Rook
                self.slider_moves(sq, is_w, &[(-1,0),(1,0),(0,-1),(0,1)], &mut moves);
            },
            5 => { // Queen
                self.slider_moves(sq, is_w, &[(-1,-1),(-1,1),(1,-1),(1,1),(-1,0),(1,0),(0,-1),(0,1)], &mut moves);
            },
            6 => { // King
                let offsets: [(i32, i32); 8] = [
                    (-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)
                ];
                for (dr, dc) in offsets {
                    let nr = row as i32 + dr;
                    let nc = col as i32 + dc;
                    if nr >= 0 && nr < 8 && nc >= 0 && nc < 8 {
                        let t = Self::sq(nr as usize, nc as usize);
                        let tp = self.board[t];
                        if tp == EMPTY || is_white(tp) != is_w {
                            moves.push(t);
                        }
                    }
                }
                // Castling
                if is_w && !self.white_king_moved && sq == 60 {
                    // Kingside
                    if !self.white_rook_h_moved && self.board[61] == EMPTY && self.board[62] == EMPTY {
                        if !self.is_square_attacked(60, false) && !self.is_square_attacked(61, false) && !self.is_square_attacked(62, false) {
                            moves.push(62);
                        }
                    }
                    // Queenside
                    if !self.white_rook_a_moved && self.board[59] == EMPTY && self.board[58] == EMPTY && self.board[57] == EMPTY {
                        if !self.is_square_attacked(60, false) && !self.is_square_attacked(59, false) && !self.is_square_attacked(58, false) {
                            moves.push(58);
                        }
                    }
                }
                if !is_w && !self.black_king_moved && sq == 4 {
                    // Kingside
                    if !self.black_rook_h_moved && self.board[5] == EMPTY && self.board[6] == EMPTY {
                        if !self.is_square_attacked(4, true) && !self.is_square_attacked(5, true) && !self.is_square_attacked(6, true) {
                            moves.push(6);
                        }
                    }
                    // Queenside
                    if !self.black_rook_a_moved && self.board[3] == EMPTY && self.board[2] == EMPTY && self.board[1] == EMPTY {
                        if !self.is_square_attacked(4, true) && !self.is_square_attacked(3, true) && !self.is_square_attacked(2, true) {
                            moves.push(2);
                        }
                    }
                }
            },
            _ => {},
        }
        moves
    }

    fn slider_moves(&self, sq: usize, is_w: bool, dirs: &[(i32, i32)], moves: &mut Vec<usize>) {
        let row = Self::row(sq) as i32;
        let col = Self::col(sq) as i32;
        for &(dr, dc) in dirs {
            let mut r = row + dr;
            let mut c = col + dc;
            while r >= 0 && r < 8 && c >= 0 && c < 8 {
                let t = Self::sq(r as usize, c as usize);
                let tp = self.board[t];
                if tp == EMPTY {
                    moves.push(t);
                } else {
                    if is_white(tp) != is_w {
                        moves.push(t); // Can capture
                    }
                    break; // Blocked
                }
                r += dr;
                c += dc;
            }
        }
    }

    /// Get legal moves (filters out moves that leave own king in check)
    fn get_legal_moves(&self, sq: usize) -> Vec<usize> {
        let piece = self.board[sq];
        if piece == EMPTY { return Vec::new(); }
        let is_w = is_white(piece);
        let pseudo = self.get_moves(sq);
        let mut legal = Vec::new();
        for &target in &pseudo {
            // Try the move on a copy
            let mut copy = self.board;
            // Handle en passant in the copy
            if piece_type(piece) == 1 && Some(target) == self.en_passant_target {
                if is_w { copy[target + 8] = EMPTY; } else { copy[target - 8] = EMPTY; }
            }
            // Handle castling rook
            if piece_type(piece) == 6 {
                let cd = Self::col(target) as i32 - Self::col(sq) as i32;
                if cd == 2 { copy[target - 1] = copy[target + 1]; copy[target + 1] = EMPTY; }
                if cd == -2 { copy[target + 1] = copy[target - 2]; copy[target - 2] = EMPTY; }
            }
            copy[target] = piece;
            copy[sq] = EMPTY;
            if !Self::is_in_check_board(&copy, is_w) {
                legal.push(target);
            }
        }
        legal
    }

    /// Check if the given side's king is attacked
    fn is_in_check(&self, white_king: bool) -> bool {
        Self::is_in_check_board(&self.board, white_king)
    }

    fn is_in_check_board(board: &[i8; 64], white_king: bool) -> bool {
        // Find king
        let king = if white_king { W_KING } else { B_KING };
        let king_sq = match board.iter().position(|&p| p == king) {
            Some(sq) => sq,
            None => return false,
        };
        Self::is_square_attacked_board(board, king_sq, !white_king)
    }

    /// Check if a square is attacked by the given side
    fn is_square_attacked(&self, sq: usize, by_white: bool) -> bool {
        Self::is_square_attacked_board(&self.board, sq, by_white)
    }

    fn is_square_attacked_board(board: &[i8; 64], sq: usize, by_white: bool) -> bool {
        for i in 0..64 {
            let p = board[i];
            if p == EMPTY { continue; }
            if is_white(p) != by_white { continue; }
            // Check if this piece can reach `sq`
            let row = i / 8;
            let col = i % 8;
            let tr = sq / 8;
            let tc = sq % 8;

            match piece_type(p) {
                1 => { // Pawn attacks diagonally
                    let dir: i32 = if is_white(p) { -1 } else { 1 };
                    if tr as i32 == row as i32 + dir && (tc as i32 - col as i32).abs() == 1 {
                        return true;
                    }
                },
                2 => { // Knight
                    let dr = (tr as i32 - row as i32).abs();
                    let dc = (tc as i32 - col as i32).abs();
                    if (dr == 2 && dc == 1) || (dr == 1 && dc == 2) {
                        return true;
                    }
                },
                3 => { // Bishop
                    if Self::attacks_diagonal(board, i, sq) { return true; }
                },
                4 => { // Rook
                    if Self::attacks_straight(board, i, sq) { return true; }
                },
                5 => { // Queen
                    if Self::attacks_diagonal(board, i, sq) || Self::attacks_straight(board, i, sq) {
                        return true;
                    }
                },
                6 => { // King
                    let dr = (tr as i32 - row as i32).abs();
                    let dc = (tc as i32 - col as i32).abs();
                    if dr <= 1 && dc <= 1 && (dr + dc) > 0 {
                        return true;
                    }
                },
                _ => {},
            }
        }
        false
    }

    fn attacks_diagonal(board: &[i8; 64], from: usize, to: usize) -> bool {
        let (fr, fc) = (from / 8, from % 8);
        let (tr, tc) = (to / 8, to % 8);
        let dr = tr as i32 - fr as i32;
        let dc = tc as i32 - fc as i32;
        if dr.abs() != dc.abs() || dr == 0 { return false; }
        let sr = if dr > 0 { 1 } else { -1 };
        let sc = if dc > 0 { 1 } else { -1 };
        let mut r = fr as i32 + sr;
        let mut c = fc as i32 + sc;
        while (r, c) != (tr as i32, tc as i32) {
            if board[(r * 8 + c) as usize] != EMPTY { return false; }
            r += sr;
            c += sc;
        }
        true
    }

    fn attacks_straight(board: &[i8; 64], from: usize, to: usize) -> bool {
        let (fr, fc) = (from / 8, from % 8);
        let (tr, tc) = (to / 8, to % 8);
        if fr != tr && fc != tc { return false; }
        if fr == tr {
            let (lo, hi) = if fc < tc { (fc, tc) } else { (tc, fc) };
            for c in (lo + 1)..hi {
                if board[fr * 8 + c] != EMPTY { return false; }
            }
        } else {
            let (lo, hi) = if fr < tr { (fr, tr) } else { (tr, fr) };
            for r in (lo + 1)..hi {
                if board[r * 8 + fc] != EMPTY { return false; }
            }
        }
        true
    }

    fn has_any_legal_moves(&self, white_side: bool) -> bool {
        for sq in 0..64 {
            let p = self.board[sq];
            if p == EMPTY { continue; }
            if is_white(p) != white_side { continue; }
            if !self.get_legal_moves(sq).is_empty() {
                return true;
            }
        }
        false
    }

    // ═══════════════════════════════════════════════════════════════
    // AI — Simple minimax with alpha-beta pruning
    // ═══════════════════════════════════════════════════════════════

    fn evaluate(&self) -> i32 {
        let mut score: i32 = 0;
        for sq in 0..64 {
            let p = self.board[sq];
            if p == EMPTY { continue; }
            let val = piece_value(p);
            if is_white(p) { score += val; } else { score -= val; }
            
            // Center control bonus
            let r = Self::row(sq);
            let c = Self::col(sq);
            let center_bonus = match (r, c) {
                (3, 3) | (3, 4) | (4, 3) | (4, 4) => 15,
                (2, 2) | (2, 5) | (5, 2) | (5, 5) => 8,
                _ => 0,
            };
            if is_white(p) { score += center_bonus; } else { score -= center_bonus; }
        }
        score
    }

    fn minimax(&mut self, depth: i32, mut alpha: i32, mut beta: i32, maximizing: bool) -> i32 {
        if depth == 0 {
            return self.evaluate();
        }

        let side = maximizing; // true = white
        let mut best;
        
        if maximizing {
            best = -100000;
            'outer_max: for sq in 0..64 {
                let p = self.board[sq];
                if p == EMPTY || !is_white(p) { continue; }
                let moves = self.get_legal_moves(sq);
                for &target in &moves {
                    let saved = self.board;
                    let saved_ep = self.en_passant_target;
                    
                    // Handle en passant
                    if piece_type(p) == 1 && Some(target) == self.en_passant_target {
                        self.board[target + 8] = EMPTY;
                    }
                    self.board[target] = p;
                    self.board[sq] = EMPTY;
                    
                    // Update en passant
                    self.en_passant_target = None;
                    if piece_type(p) == 1 && (Self::row(target) as i32 - Self::row(sq) as i32).abs() == 2 {
                        self.en_passant_target = Some(((sq + target) / 2) as usize);
                    }
                    
                    let score = self.minimax(depth - 1, alpha, beta, false);
                    
                    self.board = saved;
                    self.en_passant_target = saved_ep;
                    
                    if score > best { best = score; }
                    if score > alpha { alpha = score; }
                    if beta <= alpha { break 'outer_max; }
                }
            }
        } else {
            best = 100000;
            'outer_min: for sq in 0..64 {
                let p = self.board[sq];
                if p == EMPTY || !is_black(p) { continue; }
                let moves = self.get_legal_moves(sq);
                for &target in &moves {
                    let saved = self.board;
                    let saved_ep = self.en_passant_target;
                    
                    if piece_type(p) == 1 && Some(target) == self.en_passant_target {
                        self.board[target - 8] = EMPTY;
                    }
                    self.board[target] = p;
                    self.board[sq] = EMPTY;
                    
                    self.en_passant_target = None;
                    if piece_type(p) == 1 && (Self::row(target) as i32 - Self::row(sq) as i32).abs() == 2 {
                        self.en_passant_target = Some(((sq + target) / 2) as usize);
                    }
                    
                    let score = self.minimax(depth - 1, alpha, beta, true);
                    
                    self.board = saved;
                    self.en_passant_target = saved_ep;
                    
                    if score < best { best = score; }
                    if score < beta { beta = score; }
                    if beta <= alpha { break 'outer_min; }
                }
            }
        }
        best
    }

    fn ai_move(&mut self) {
        let mut best_from: Option<usize> = None;
        let mut best_to: Option<usize> = None;
        let mut best_score = 100000i32; // AI is black, minimizing
        
        for sq in 0..64 {
            let p = self.board[sq];
            if p == EMPTY || !is_black(p) { continue; }
            let moves = self.get_legal_moves(sq);
            for &target in &moves {
                let saved = self.board;
                let saved_ep = self.en_passant_target;
                
                if piece_type(p) == 1 && Some(target) == self.en_passant_target {
                    self.board[target - 8] = EMPTY;
                }
                self.board[target] = p;
                self.board[sq] = EMPTY;
                
                self.en_passant_target = None;
                if piece_type(p) == 1 && (Self::row(target) as i32 - Self::row(sq) as i32).abs() == 2 {
                    self.en_passant_target = Some(((sq + target) / 2) as usize);
                }
                
                let score = self.minimax(self.ai_depth, -100000, 100000, true);
                
                self.board = saved;
                self.en_passant_target = saved_ep;
                
                if score < best_score {
                    best_score = score;
                    best_from = Some(sq);
                    best_to = Some(target);
                }
            }
        }
        
        if let (Some(from), Some(to)) = (best_from, best_to) {
            self.make_move(from, to);
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // SCORE — Material advantage
    // ═══════════════════════════════════════════════════════════════

    /// Calculate material advantage (positive = white ahead, negative = black ahead)
    pub fn material_score(&self) -> i32 {
        let mut white_mat = 0i32;
        let mut black_mat = 0i32;
        for sq in 0..64 {
            let p = self.board[sq];
            if p == EMPTY { continue; }
            let val = piece_value(p);
            // Don't count kings — they're always present
            if piece_type(p) == 6 { continue; }
            if is_white(p) { white_mat += val; } else { black_mat += val; }
        }
        white_mat - black_mat
    }

    // ═══════════════════════════════════════════════════════════════
    // MOUSE INPUT — Click & Drag-and-Drop
    // ═══════════════════════════════════════════════════════════════

    /// Handle mouse click on board square (col 0-7, row 0-7)
    /// Returns true if the click was handled
    pub fn handle_mouse_click(&mut self, col: i32, row: i32) -> bool {
        if col < 0 || col > 7 || row < 0 || row > 7 { return false; }
        let sq = row as usize * 8 + col as usize;
        
        if self.phase == GamePhase::Checkmate || self.phase == GamePhase::Stalemate {
            return false;
        }
        if self.phase == GamePhase::Promotion {
            return false; // Promotion handled by keyboard
        }

        let piece = self.board[sq];
        
        match self.input_mode {
            InputMode::SelectPiece => {
                if piece == EMPTY { return false; }
                // Check if it's the current player's piece
                let is_own = (self.white_turn && is_white(piece)) || (!self.white_turn && is_black(piece));
                if !is_own { return false; }
                
                // Select this piece
                self.valid_moves = self.get_legal_moves(sq);
                if self.valid_moves.is_empty() {
                    self.message = String::from("No legal moves for this piece");
                    return true;
                }
                self.selected = Some(sq);
                self.cursor = sq;
                self.input_mode = InputMode::SelectTarget;
                let name = Self::square_name(sq);
                self.message = format!("Move {} from {}", piece_ascii(piece), name);
                
                // Start drag
                self.drag_from = Some(sq);
                self.dragging_piece = Some(piece);
                return true;
            },
            InputMode::SelectTarget => {
                // If clicking own piece → reselect
                if piece != EMPTY {
                    let is_own = (self.white_turn && is_white(piece)) || (!self.white_turn && is_black(piece));
                    if is_own {
                        self.valid_moves = self.get_legal_moves(sq);
                        if !self.valid_moves.is_empty() {
                            self.selected = Some(sq);
                            self.cursor = sq;
                            let name = Self::square_name(sq);
                            self.message = format!("Move {} from {}", piece_ascii(piece), name);
                            // Start drag on new piece
                            self.drag_from = Some(sq);
                            self.dragging_piece = Some(piece);
                        }
                        return true;
                    }
                }
                
                // Check if target is valid
                if self.valid_moves.contains(&sq) {
                    if let Some(from) = self.selected {
                        self.drag_from = None;
                        self.dragging_piece = None;
                        self.make_move(from, sq);
                        return true;
                    }
                } else {
                    self.message = String::from("Invalid move");
                }
                return true;
            },
        }
    }

    /// Handle mouse release — complete drag & drop
    pub fn handle_mouse_release(&mut self, col: i32, row: i32) {
        if self.drag_from.is_none() || self.dragging_piece.is_none() {
            return;
        }
        
        let from = self.drag_from.unwrap();
        self.drag_from = None;
        self.dragging_piece = None;
        
        if col < 0 || col > 7 || row < 0 || row > 7 {
            // Dropped outside board — cancel
            return;
        }
        
        let sq = row as usize * 8 + col as usize;
        
        // If dropped on same square — keep selection (was just a click)
        if sq == from { return; }
        
        // Check if it's a valid move
        if self.valid_moves.contains(&sq) {
            self.make_move(from, sq);
        }
        // Otherwise — keep piece selected, invalid drop just snaps back
    }

    /// Update drag pixel position during mouse move
    pub fn update_drag_position(&mut self, px: i32, py: i32) {
        self.drag_pixel_x = px;
        self.drag_pixel_y = py;
    }

    // ═══════════════════════════════════════════════════════════════
    // TIMER
    // ═══════════════════════════════════════════════════════════════

    /// Toggle timer on/off
    pub fn toggle_timer(&mut self) {
        self.timer_enabled = !self.timer_enabled;
        if self.timer_enabled {
            self.white_time_ms = self.timer_initial_ms;
            self.black_time_ms = self.timer_initial_ms;
            self.timer_started = false;
            self.message = format!("Timer ON — {}min/side", self.timer_initial_ms / 60_000);
        } else {
            self.message = String::from("Timer OFF");
        }
    }

    /// Cycle timer preset: 1min, 3min, 5min, 10min, 15min, 30min
    pub fn cycle_timer_preset(&mut self) {
        let presets: [u64; 6] = [60_000, 180_000, 300_000, 600_000, 900_000, 1_800_000];
        let current_idx = presets.iter().position(|&t| t == self.timer_initial_ms).unwrap_or(3);
        let next_idx = (current_idx + 1) % presets.len();
        self.timer_initial_ms = presets[next_idx];
        self.white_time_ms = self.timer_initial_ms;
        self.black_time_ms = self.timer_initial_ms;
        self.timer_started = false;
        let mins = self.timer_initial_ms / 60_000;
        let secs = (self.timer_initial_ms % 60_000) / 1000;
        if secs > 0 {
            self.message = format!("Timer: {}m{}s/side", mins, secs);
        } else {
            self.message = format!("Timer: {}min/side", mins);
        }
    }

    /// Tick timer — call periodically with elapsed ms since last tick
    pub fn tick_timer(&mut self, elapsed_ms: u64) {
        if !self.timer_enabled || !self.timer_started { return; }
        if self.phase == GamePhase::Checkmate || self.phase == GamePhase::Stalemate { return; }
        
        if self.white_turn {
            self.white_time_ms = self.white_time_ms.saturating_sub(elapsed_ms);
            if self.white_time_ms == 0 {
                self.phase = GamePhase::Checkmate; // reuse for game over
                self.message = String::from("Time's up! Black wins!");
            }
        } else {
            self.black_time_ms = self.black_time_ms.saturating_sub(elapsed_ms);
            if self.black_time_ms == 0 {
                self.phase = GamePhase::Checkmate;
                self.message = String::from("Time's up! White wins!");
            }
        }
    }

    /// Format time as MM:SS
    pub fn format_time(ms: u64) -> String {
        let total_secs = ms / 1000;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}
