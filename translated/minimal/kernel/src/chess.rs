







use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;





pub const Q: i8 = 0;
pub const DFO_: i8 = 1;
pub const ALU_: i8 = 2;
pub const ALT_: i8 = 3;
pub const ALV_: i8 = 4;
pub const BLJ_: i8 = 5;
pub const BLI_: i8 = 6;
pub const BOH_: i8 = -1;
pub const APS_: i8 = -2;
pub const APQ_: i8 = -3;
pub const APT_: i8 = -4;
pub const BOI_: i8 = -5;
pub const APR_: i8 = -6;


fn xr(aa: i8) -> bool { aa > 0 }

fn bhi(aa: i8) -> bool { aa < 0 }

fn agw(aa: i8) -> i8 { if aa < 0 { -aa } else { aa } }


fn iuu(aa: i8) -> i32 {
    match agw(aa) {
        1 => 100,   
        2 => 320,   
        3 => 330,   
        4 => 500,   
        5 => 900,   
        6 => 20000, 
        _ => 0,
    }
}


pub fn qqi(aa: i8) -> char {
    match aa {
        1  => '\u{2659}', 
        2  => '\u{2658}', 
        3  => '\u{2657}', 
        4  => '\u{2656}', 
        5  => '\u{2655}', 
        6  => '\u{2654}', 
        -1 => '\u{265F}', 
        -2 => '\u{265E}', 
        -3 => '\u{265D}', 
        -4 => '\u{265C}', 
        -5 => '\u{265B}', 
        -6 => '\u{265A}', 
        _  => ' ',
    }
}


pub fn dwn(aa: i8) -> char {
    match aa {
        1  => 'P', 2  => 'N', 3  => 'B', 4  => 'R', 5  => 'Q', 6  => 'K',
        -1 => 'p', -2 => 'n', -3 => 'b', -4 => 'r', -5 => 'q', -6 => 'k',
        _  => ' ',
    }
}


pub fn qqj(aa: i8) -> u32 {
    if xr(aa) { 0xFFFFFFFF } else { 0xFF1A1A1A }
}





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
    SelectPiece,   
    SelectTarget,  
}

pub struct ChessState {
    pub board: [i8; 64],
    pub white_turn: bool,
    pub selected: Option<usize>,     
    pub cursor: usize,               
    pub valid_moves: Vec<usize>,     
    pub phase: GamePhase,
    pub input_mode: InputMode,
    pub message: String,
    pub move_history: Vec<String>,
    
    pub white_king_moved: bool,
    pub black_king_moved: bool,
    pub white_rook_a_moved: bool,
    pub white_rook_h_moved: bool,
    pub black_rook_a_moved: bool,
    pub black_rook_h_moved: bool,
    
    pub en_passant_target: Option<usize>, 
    
    pub promotion_square: Option<usize>,
    
    pub vs_ai: bool,
    pub ai_thinking: bool,
    pub ai_depth: i32,
    pub rng_state: u32,
    
    pub last_move_from: Option<usize>,
    pub last_move_to: Option<usize>,
    
    pub drag_from: Option<usize>,     
    pub dragging_piece: Option<i8>,   
    pub drag_pixel_x: i32,           
    pub drag_pixel_y: i32,           
    
    pub white_time_ms: u64,
    pub black_time_ms: u64,
    pub timer_enabled: bool,
    pub timer_initial_ms: u64,       
    pub last_tick_ms: u64,           
    pub timer_started: bool,         
}

impl ChessState {
    pub fn new() -> Self {
        let mut state = ChessState {
            board: [Q; 64],
            white_turn: true,
            selected: None,
            cursor: 52, 
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
            
            drag_from: None,
            dragging_piece: None,
            drag_pixel_x: 0,
            drag_pixel_y: 0,
            
            white_time_ms: 600_000,  
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
        
        self.board[0] = APT_;
        self.board[1] = APS_;
        self.board[2] = APQ_;
        self.board[3] = BOI_;
        self.board[4] = APR_;
        self.board[5] = APQ_;
        self.board[6] = APS_;
        self.board[7] = APT_;
        for i in 8..16 {
            self.board[i] = BOH_;
        }
        
        for i in 48..56 {
            self.board[i] = DFO_;
        }
        self.board[56] = ALV_;
        self.board[57] = ALU_;
        self.board[58] = ALT_;
        self.board[59] = BLJ_;
        self.board[60] = BLI_;
        self.board[61] = ALT_;
        self.board[62] = ALU_;
        self.board[63] = ALV_;
    }

    

    fn row(cu: usize) -> usize { cu / 8 }
    fn col(cu: usize) -> usize { cu % 8 }
    fn cu(row: usize, col: usize) -> usize { row * 8 + col }

    fn der(cu: usize) -> String {
        let file = (b'a' + Self::col(cu) as u8) as char;
        let obi = (b'1' + (7 - Self::row(cu)) as u8) as char;
        format!("{}{}", file, obi)
    }

    

    pub fn handle_key(&mut self, key: u8) {
        use crate::keyboard::{T_, S_, AI_, AJ_};

        if self.phase == GamePhase::Checkmate || self.phase == GamePhase::Stalemate {
            if key == b'r' || key == b'R' {
                *self = ChessState::new();
            }
            return;
        }

        
        if self.phase == GamePhase::Promotion {
            if let Some(cu) = self.promotion_square {
                let eip: i8 = if Self::row(cu) == 0 { 1 } else { -1 };
                match key {
                    b'q' | b'Q' | b'1' => { self.board[cu] = BLJ_ * eip; self.finish_promotion(); }
                    b'r' | b'R' | b'2' => { self.board[cu] = ALV_ * eip; self.finish_promotion(); }
                    b'b' | b'B' | b'3' => { self.board[cu] = ALT_ * eip; self.finish_promotion(); }
                    b'n' | b'N' | b'4' => { self.board[cu] = ALU_ * eip; self.finish_promotion(); }
                    _ => {}
                }
            }
            return;
        }

        match key {
            T_    => { if self.cursor >= 8 { self.cursor -= 8; } },
            S_  => { if self.cursor < 56 { self.cursor += 8; } },
            AI_  => { if self.cursor % 8 > 0 { self.cursor -= 1; } },
            AJ_ => { if self.cursor % 8 < 7 { self.cursor += 1; } },
            0x0D | b' ' => { 
                self.handle_select();
            },
            0x1B => { 
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
                if piece == Q { return; }
                
                if self.white_turn && !xr(piece) { 
                    self.message = String::from("Select a white piece");
                    return; 
                }
                if !self.white_turn && !bhi(piece) { 
                    self.message = String::from("Select a black piece");
                    return; 
                }
                
                self.valid_moves = self.get_legal_moves(self.cursor);
                if self.valid_moves.is_empty() {
                    self.message = String::from("No legal moves for this piece");
                    return;
                }
                self.selected = Some(self.cursor);
                self.input_mode = InputMode::SelectTarget;
                let name = Self::der(self.cursor);
                self.message = format!("Move {} from {}", dwn(piece), name);
            },
            InputMode::SelectTarget => {
                let from = match self.selected {
                    Some(cu) => cu,
                    None => { self.input_mode = InputMode::SelectPiece; return; }
                };
                
                
                let fck = self.board[self.cursor];
                if fck != Q {
                    let dsr = (self.white_turn && xr(fck)) || 
                                 (!self.white_turn && bhi(fck));
                    if dsr {
                        
                        self.valid_moves = self.get_legal_moves(self.cursor);
                        if !self.valid_moves.is_empty() {
                            self.selected = Some(self.cursor);
                            let name = Self::der(self.cursor);
                            self.message = format!("Move {} from {}", dwn(fck), name);
                        }
                        return;
                    }
                }

                if !self.valid_moves.contains(&self.cursor) {
                    self.message = String::from("Invalid move");
                    return;
                }

                
                self.make_move(from, self.cursor);
            },
        }
    }

    fn make_move(&mut self, from: usize, to: usize) {
        let piece = self.board[from];
        let captured = self.board[to];
        
        
        let ngj = format!("{}{}{}", dwn(piece), Self::der(from), Self::der(to));
        self.move_history.push(ngj);
        
        
        self.last_move_from = Some(from);
        self.last_move_to = Some(to);

        
        if agw(piece) == 1 && Some(to) == self.en_passant_target {
            
            if xr(piece) {
                self.board[to + 8] = Q; 
            } else {
                self.board[to - 8] = Q; 
            }
        }

        
        if agw(piece) == 6 {
            let hml = Self::col(to) as i32 - Self::col(from) as i32;
            if hml == 2 {
                
                self.board[to - 1] = self.board[to + 1]; 
                self.board[to + 1] = Q;
            } else if hml == -2 {
                
                self.board[to + 1] = self.board[to - 2]; 
                self.board[to - 2] = Q;
            }
        }

        
        self.board[to] = piece;
        self.board[from] = Q;

        
        if agw(piece) == 6 {
            if xr(piece) { self.white_king_moved = true; }
            else { self.black_king_moved = true; }
        }
        if agw(piece) == 4 {
            if from == 56 { self.white_rook_a_moved = true; }
            if from == 63 { self.white_rook_h_moved = true; }
            if from == 0 { self.black_rook_a_moved = true; }
            if from == 7 { self.black_rook_h_moved = true; }
        }

        
        self.en_passant_target = None;
        if agw(piece) == 1 {
            let oik = Self::row(to) as i32 - Self::row(from) as i32;
            if oik.abs() == 2 {
                
                self.en_passant_target = Some(((from as i32 + to as i32) / 2) as usize);
            }
        }

        
        if agw(piece) == 1 && (Self::row(to) == 0 || Self::row(to) == 7) {
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

        
        if self.timer_enabled && !self.timer_started {
            self.timer_started = true;
        }

        
        let gce = self.is_in_check(self.white_turn);
        let idv = self.has_any_legal_moves(self.white_turn);

        if gce && !idv {
            self.phase = GamePhase::Checkmate;
            self.message = if self.white_turn {
                String::from("Checkmate! Black wins!")
            } else {
                String::from("Checkmate! White wins!")
            };
        } else if !gce && !idv {
            self.phase = GamePhase::Stalemate;
            self.message = String::from("Stalemate — Draw!");
        } else if gce {
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

        
        if self.vs_ai && !self.white_turn && self.phase == GamePhase::Playing || 
           (self.vs_ai && !self.white_turn && self.phase == GamePhase::Check) {
            self.ai_move();
        }
    }

    
    
    

    
    fn get_moves(&self, cu: usize) -> Vec<usize> {
        let piece = self.board[cu];
        if piece == Q { return Vec::new(); }
        let arg = xr(piece);
        let mut aeb = Vec::new();
        let row = Self::row(cu);
        let col = Self::col(cu);

        match agw(piece) {
            1 => { 
                let it: i32 = if arg { -1 } else { 1 };
                let bpd = if arg { 6 } else { 1 };
                
                
                let enq = cu as i32 + it * 8;
                if enq >= 0 && enq < 64 && self.board[enq as usize] == Q {
                    aeb.push(enq as usize);
                    
                    if row == bpd {
                        let enr = cu as i32 + it * 16;
                        if enr >= 0 && enr < 64 && self.board[enr as usize] == Q {
                            aeb.push(enr as usize);
                        }
                    }
                }
                
                for aht in [-1i32, 1] {
                    let bia = col as i32 + aht;
                    if bia >= 0 && bia < 8 {
                        let target = (row as i32 + it) * 8 + bia;
                        if target >= 0 && target < 64 {
                            let t = target as usize;
                            let tp = self.board[t];
                            if (tp != Q && xr(tp) != arg) || Some(t) == self.en_passant_target {
                                aeb.push(t);
                            }
                        }
                    }
                }
            },
            2 => { 
                let agv: [(i32, i32); 8] = [
                    (-2,-1),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)
                ];
                for (qw, aht) in agv {
                    let nr = row as i32 + qw;
                    let bia = col as i32 + aht;
                    if nr >= 0 && nr < 8 && bia >= 0 && bia < 8 {
                        let t = Self::cu(nr as usize, bia as usize);
                        let tp = self.board[t];
                        if tp == Q || xr(tp) != arg {
                            aeb.push(t);
                        }
                    }
                }
            },
            3 => { 
                self.slider_moves(cu, arg, &[(-1,-1),(-1,1),(1,-1),(1,1)], &mut aeb);
            },
            4 => { 
                self.slider_moves(cu, arg, &[(-1,0),(1,0),(0,-1),(0,1)], &mut aeb);
            },
            5 => { 
                self.slider_moves(cu, arg, &[(-1,-1),(-1,1),(1,-1),(1,1),(-1,0),(1,0),(0,-1),(0,1)], &mut aeb);
            },
            6 => { 
                let agv: [(i32, i32); 8] = [
                    (-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)
                ];
                for (qw, aht) in agv {
                    let nr = row as i32 + qw;
                    let bia = col as i32 + aht;
                    if nr >= 0 && nr < 8 && bia >= 0 && bia < 8 {
                        let t = Self::cu(nr as usize, bia as usize);
                        let tp = self.board[t];
                        if tp == Q || xr(tp) != arg {
                            aeb.push(t);
                        }
                    }
                }
                
                if arg && !self.white_king_moved && cu == 60 {
                    
                    if !self.white_rook_h_moved && self.board[61] == Q && self.board[62] == Q {
                        if !self.is_square_attacked(60, false) && !self.is_square_attacked(61, false) && !self.is_square_attacked(62, false) {
                            aeb.push(62);
                        }
                    }
                    
                    if !self.white_rook_a_moved && self.board[59] == Q && self.board[58] == Q && self.board[57] == Q {
                        if !self.is_square_attacked(60, false) && !self.is_square_attacked(59, false) && !self.is_square_attacked(58, false) {
                            aeb.push(58);
                        }
                    }
                }
                if !arg && !self.black_king_moved && cu == 4 {
                    
                    if !self.black_rook_h_moved && self.board[5] == Q && self.board[6] == Q {
                        if !self.is_square_attacked(4, true) && !self.is_square_attacked(5, true) && !self.is_square_attacked(6, true) {
                            aeb.push(6);
                        }
                    }
                    
                    if !self.black_rook_a_moved && self.board[3] == Q && self.board[2] == Q && self.board[1] == Q {
                        if !self.is_square_attacked(4, true) && !self.is_square_attacked(3, true) && !self.is_square_attacked(2, true) {
                            aeb.push(2);
                        }
                    }
                }
            },
            _ => {},
        }
        aeb
    }

    fn slider_moves(&self, cu: usize, arg: bool, bfy: &[(i32, i32)], aeb: &mut Vec<usize>) {
        let row = Self::row(cu) as i32;
        let col = Self::col(cu) as i32;
        for &(qw, aht) in bfy {
            let mut r = row + qw;
            let mut c = col + aht;
            while r >= 0 && r < 8 && c >= 0 && c < 8 {
                let t = Self::cu(r as usize, c as usize);
                let tp = self.board[t];
                if tp == Q {
                    aeb.push(t);
                } else {
                    if xr(tp) != arg {
                        aeb.push(t); 
                    }
                    break; 
                }
                r += qw;
                c += aht;
            }
        }
    }

    
    fn get_legal_moves(&self, cu: usize) -> Vec<usize> {
        let piece = self.board[cu];
        if piece == Q { return Vec::new(); }
        let arg = xr(piece);
        let bit = self.get_moves(cu);
        let mut ijw = Vec::new();
        for &target in &bit {
            
            let mut copy = self.board;
            
            if agw(piece) == 1 && Some(target) == self.en_passant_target {
                if arg { copy[target + 8] = Q; } else { copy[target - 8] = Q; }
            }
            
            if agw(piece) == 6 {
                let cd = Self::col(target) as i32 - Self::col(cu) as i32;
                if cd == 2 { copy[target - 1] = copy[target + 1]; copy[target + 1] = Q; }
                if cd == -2 { copy[target + 1] = copy[target - 2]; copy[target - 2] = Q; }
            }
            copy[target] = piece;
            copy[cu] = Q;
            if !Self::ihy(&copy, arg) {
                ijw.push(target);
            }
        }
        ijw
    }

    
    fn is_in_check(&self, white_king: bool) -> bool {
        Self::ihy(&self.board, white_king)
    }

    fn ihy(board: &[i8; 64], white_king: bool) -> bool {
        
        let mvu = if white_king { BLI_ } else { APR_ };
        let mvv = match board.iter().position(|&aa| aa == mvu) {
            Some(cu) => cu,
            None => return false,
        };
        Self::iik(board, mvv, !white_king)
    }

    
    fn is_square_attacked(&self, cu: usize, by_white: bool) -> bool {
        Self::iik(&self.board, cu, by_white)
    }

    fn iik(board: &[i8; 64], cu: usize, by_white: bool) -> bool {
        for i in 0..64 {
            let aa = board[i];
            if aa == Q { continue; }
            if xr(aa) != by_white { continue; }
            
            let row = i / 8;
            let col = i % 8;
            let tr = cu / 8;
            let wo = cu % 8;

            match agw(aa) {
                1 => { 
                    let it: i32 = if xr(aa) { -1 } else { 1 };
                    if tr as i32 == row as i32 + it && (wo as i32 - col as i32).abs() == 1 {
                        return true;
                    }
                },
                2 => { 
                    let qw = (tr as i32 - row as i32).abs();
                    let aht = (wo as i32 - col as i32).abs();
                    if (qw == 2 && aht == 1) || (qw == 1 && aht == 2) {
                        return true;
                    }
                },
                3 => { 
                    if Self::hfu(board, i, cu) { return true; }
                },
                4 => { 
                    if Self::hfv(board, i, cu) { return true; }
                },
                5 => { 
                    if Self::hfu(board, i, cu) || Self::hfv(board, i, cu) {
                        return true;
                    }
                },
                6 => { 
                    let qw = (tr as i32 - row as i32).abs();
                    let aht = (wo as i32 - col as i32).abs();
                    if qw <= 1 && aht <= 1 && (qw + aht) > 0 {
                        return true;
                    }
                },
                _ => {},
            }
        }
        false
    }

    fn hfu(board: &[i8; 64], from: usize, to: usize) -> bool {
        let (ko, br) = (from / 8, from % 8);
        let (tr, wo) = (to / 8, to % 8);
        let qw = tr as i32 - ko as i32;
        let aht = wo as i32 - br as i32;
        if qw.abs() != aht.abs() || qw == 0 { return false; }
        let pb = if qw > 0 { 1 } else { -1 };
        let dr = if aht > 0 { 1 } else { -1 };
        let mut r = ko as i32 + pb;
        let mut c = br as i32 + dr;
        while (r, c) != (tr as i32, wo as i32) {
            if board[(r * 8 + c) as usize] != Q { return false; }
            r += pb;
            c += dr;
        }
        true
    }

    fn hfv(board: &[i8; 64], from: usize, to: usize) -> bool {
        let (ko, br) = (from / 8, from % 8);
        let (tr, wo) = (to / 8, to % 8);
        if ko != tr && br != wo { return false; }
        if ko == tr {
            let (lo, hi) = if br < wo { (br, wo) } else { (wo, br) };
            for c in (lo + 1)..hi {
                if board[ko * 8 + c] != Q { return false; }
            }
        } else {
            let (lo, hi) = if ko < tr { (ko, tr) } else { (tr, ko) };
            for r in (lo + 1)..hi {
                if board[r * 8 + br] != Q { return false; }
            }
        }
        true
    }

    fn has_any_legal_moves(&self, white_side: bool) -> bool {
        for cu in 0..64 {
            let aa = self.board[cu];
            if aa == Q { continue; }
            if xr(aa) != white_side { continue; }
            if !self.get_legal_moves(cu).is_empty() {
                return true;
            }
        }
        false
    }

    
    
    

    fn evaluate(&self) -> i32 {
        let mut score: i32 = 0;
        for cu in 0..64 {
            let aa = self.board[cu];
            if aa == Q { continue; }
            let val = iuu(aa);
            if xr(aa) { score += val; } else { score -= val; }
            
            
            let r = Self::row(cu);
            let c = Self::col(cu);
            let hkb = match (r, c) {
                (3, 3) | (3, 4) | (4, 3) | (4, 4) => 15,
                (2, 2) | (2, 5) | (5, 2) | (5, 5) => 8,
                _ => 0,
            };
            if xr(aa) { score += hkb; } else { score -= hkb; }
        }
        score
    }

    fn minimax(&mut self, depth: i32, mut alpha: i32, mut bqo: i32, maximizing: bool) -> i32 {
        if depth == 0 {
            return self.evaluate();
        }

        let den = maximizing; 
        let mut adj;
        
        if maximizing {
            adj = -100000;
            'outer_max: for cu in 0..64 {
                let aa = self.board[cu];
                if aa == Q || !xr(aa) { continue; }
                let aeb = self.get_legal_moves(cu);
                for &target in &aeb {
                    let bvq = self.board;
                    let dyh = self.en_passant_target;
                    
                    
                    if agw(aa) == 1 && Some(target) == self.en_passant_target {
                        self.board[target + 8] = Q;
                    }
                    self.board[target] = aa;
                    self.board[cu] = Q;
                    
                    
                    self.en_passant_target = None;
                    if agw(aa) == 1 && (Self::row(target) as i32 - Self::row(cu) as i32).abs() == 2 {
                        self.en_passant_target = Some(((cu + target) / 2) as usize);
                    }
                    
                    let score = self.minimax(depth - 1, alpha, bqo, false);
                    
                    self.board = bvq;
                    self.en_passant_target = dyh;
                    
                    if score > adj { adj = score; }
                    if score > alpha { alpha = score; }
                    if bqo <= alpha { break 'outer_max; }
                }
            }
        } else {
            adj = 100000;
            'outer_min: for cu in 0..64 {
                let aa = self.board[cu];
                if aa == Q || !bhi(aa) { continue; }
                let aeb = self.get_legal_moves(cu);
                for &target in &aeb {
                    let bvq = self.board;
                    let dyh = self.en_passant_target;
                    
                    if agw(aa) == 1 && Some(target) == self.en_passant_target {
                        self.board[target - 8] = Q;
                    }
                    self.board[target] = aa;
                    self.board[cu] = Q;
                    
                    self.en_passant_target = None;
                    if agw(aa) == 1 && (Self::row(target) as i32 - Self::row(cu) as i32).abs() == 2 {
                        self.en_passant_target = Some(((cu + target) / 2) as usize);
                    }
                    
                    let score = self.minimax(depth - 1, alpha, bqo, true);
                    
                    self.board = bvq;
                    self.en_passant_target = dyh;
                    
                    if score < adj { adj = score; }
                    if score < bqo { bqo = score; }
                    if bqo <= alpha { break 'outer_min; }
                }
            }
        }
        adj
    }

    fn ai_move(&mut self) {
        let mut hhi: Option<usize> = None;
        let mut hhl: Option<usize> = None;
        let mut djb = 100000i32; 
        
        for cu in 0..64 {
            let aa = self.board[cu];
            if aa == Q || !bhi(aa) { continue; }
            let aeb = self.get_legal_moves(cu);
            for &target in &aeb {
                let bvq = self.board;
                let dyh = self.en_passant_target;
                
                if agw(aa) == 1 && Some(target) == self.en_passant_target {
                    self.board[target - 8] = Q;
                }
                self.board[target] = aa;
                self.board[cu] = Q;
                
                self.en_passant_target = None;
                if agw(aa) == 1 && (Self::row(target) as i32 - Self::row(cu) as i32).abs() == 2 {
                    self.en_passant_target = Some(((cu + target) / 2) as usize);
                }
                
                let score = self.minimax(self.ai_depth, -100000, 100000, true);
                
                self.board = bvq;
                self.en_passant_target = dyh;
                
                if score < djb {
                    djb = score;
                    hhi = Some(cu);
                    hhl = Some(target);
                }
            }
        }
        
        if let (Some(from), Some(to)) = (hhi, hhl) {
            self.make_move(from, to);
        }
    }

    
    
    

    
    pub fn material_score(&self) -> i32 {
        let mut jrb = 0i32;
        let mut hhy = 0i32;
        for cu in 0..64 {
            let aa = self.board[cu];
            if aa == Q { continue; }
            let val = iuu(aa);
            
            if agw(aa) == 6 { continue; }
            if xr(aa) { jrb += val; } else { hhy += val; }
        }
        jrb - hhy
    }

    
    
    

    
    
    pub fn handle_mouse_click(&mut self, col: i32, row: i32) -> bool {
        if col < 0 || col > 7 || row < 0 || row > 7 { return false; }
        let cu = row as usize * 8 + col as usize;
        
        if self.phase == GamePhase::Checkmate || self.phase == GamePhase::Stalemate {
            return false;
        }
        if self.phase == GamePhase::Promotion {
            return false; 
        }

        let piece = self.board[cu];
        
        match self.input_mode {
            InputMode::SelectPiece => {
                if piece == Q { return false; }
                
                let dsr = (self.white_turn && xr(piece)) || (!self.white_turn && bhi(piece));
                if !dsr { return false; }
                
                
                self.valid_moves = self.get_legal_moves(cu);
                if self.valid_moves.is_empty() {
                    self.message = String::from("No legal moves for this piece");
                    return true;
                }
                self.selected = Some(cu);
                self.cursor = cu;
                self.input_mode = InputMode::SelectTarget;
                let name = Self::der(cu);
                self.message = format!("Move {} from {}", dwn(piece), name);
                
                
                self.drag_from = Some(cu);
                self.dragging_piece = Some(piece);
                return true;
            },
            InputMode::SelectTarget => {
                
                if piece != Q {
                    let dsr = (self.white_turn && xr(piece)) || (!self.white_turn && bhi(piece));
                    if dsr {
                        self.valid_moves = self.get_legal_moves(cu);
                        if !self.valid_moves.is_empty() {
                            self.selected = Some(cu);
                            self.cursor = cu;
                            let name = Self::der(cu);
                            self.message = format!("Move {} from {}", dwn(piece), name);
                            
                            self.drag_from = Some(cu);
                            self.dragging_piece = Some(piece);
                        }
                        return true;
                    }
                }
                
                
                if self.valid_moves.contains(&cu) {
                    if let Some(from) = self.selected {
                        self.drag_from = None;
                        self.dragging_piece = None;
                        self.make_move(from, cu);
                        return true;
                    }
                } else {
                    self.message = String::from("Invalid move");
                }
                return true;
            },
        }
    }

    
    pub fn handle_mouse_release(&mut self, col: i32, row: i32) {
        let from = match self.drag_from.take() {
            Some(f) => f,
            None => return,
        };
        if self.dragging_piece.is_none() {
            return;
        }
        self.dragging_piece = None;
        
        if col < 0 || col > 7 || row < 0 || row > 7 {
            
            return;
        }
        
        let cu = row as usize * 8 + col as usize;
        
        
        if cu == from { return; }
        
        
        if self.valid_moves.contains(&cu) {
            self.make_move(from, cu);
        }
        
    }

    
    pub fn update_drag_position(&mut self, p: i32, o: i32) {
        self.drag_pixel_x = p;
        self.drag_pixel_y = o;
    }

    
    
    

    
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

    
    pub fn cycle_timer_preset(&mut self) {
        let dwu: [u64; 6] = [60_000, 180_000, 300_000, 600_000, 900_000, 1_800_000];
        let lak = dwu.iter().position(|&t| t == self.timer_initial_ms).unwrap_or(3);
        let euz = (lak + 1) % dwu.len();
        self.timer_initial_ms = dwu[euz];
        self.white_time_ms = self.timer_initial_ms;
        self.black_time_ms = self.timer_initial_ms;
        self.timer_started = false;
        let acf = self.timer_initial_ms / 60_000;
        let im = (self.timer_initial_ms % 60_000) / 1000;
        if im > 0 {
            self.message = format!("Timer: {}m{}s/side", acf, im);
        } else {
            self.message = format!("Timer: {}min/side", acf);
        }
    }

    
    pub fn tick_timer(&mut self, elapsed_ms: u64) {
        if !self.timer_enabled || !self.timer_started { return; }
        if self.phase == GamePhase::Checkmate || self.phase == GamePhase::Stalemate { return; }
        
        if self.white_turn {
            self.white_time_ms = self.white_time_ms.saturating_sub(elapsed_ms);
            if self.white_time_ms == 0 {
                self.phase = GamePhase::Checkmate; 
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

    
    pub fn format_time(dh: u64) -> String {
        let fdj = dh / 1000;
        let acf = fdj / 60;
        let im = fdj % 60;
        format!("{:02}:{:02}", acf, im)
    }
}
