


























use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::collections::VecDeque;






const KB_: u32 = 0xFF050606;

const DPI_: u32 = 0xFF00FF66;

const CQ_: u32 = 0xFF00CC55;

const DPJ_: u32 = 0xFF00AA44;

const DPL_: u32 = 0xFF008844;

const DPK_: u32 = 0xFF003B1A;


const ARU_: u32 = 0xFF00FF88;

const EJD_: u32 = 0xFF1A3A2A;


const MN_: [u32; 16] = [
    0xFF050606, 
    0xFF882222, 
    0xFF00CC55, 
    0xFF888822, 
    0xFF4466AA, 
    0xFF884488, 
    0xFF448888, 
    0xFFAAAAAA, 
    0xFF666666, 
    0xFFFF5555, 
    0xFF00FF66, 
    0xFFFFFF00, 
    0xFF6688CC, 
    0xFFCC66CC, 
    0xFF66CCCC, 
    0xFFE0E8E4, 
];






#[derive(Debug, Clone, Copy)]
pub struct Cell {
    
    pub ch: char,
    
    pub fg: u32,
    
    pub bg: u32,
    
    pub attr: CellAttr,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: CQ_,
            bg: KB_,
            attr: CellAttr::default(),
        }
    }
}


#[derive(Debug, Clone, Copy, Default)]
pub struct CellAttr {
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
}






#[derive(Debug, Clone, Copy, PartialEq)]
enum ParseState {
    
    Normal,
    
    Escape,
    
    Csi,
    
    Osc,
    
    Dcs,
}


#[derive(Debug, Clone)]
pub enum Asu {
    
    Print(char),
    
    CursorUp(u16),
    
    CursorDown(u16),
    
    CursorRight(u16),
    
    CursorLeft(u16),
    
    CursorPosition(u16, u16),
    
    SaveCursor,
    
    RestoreCursor,
    
    EraseToEnd,
    
    EraseToStart,
    
    EraseScreen,
    
    EraseLineToEnd,
    
    EraseLineToStart,
    
    EraseLine,
    
    Sgr(Vec<u16>),
    
    ScrollUp(u16),
    
    ScrollDown(u16),
    
    SetTitle(String),
    
    Bell,
    
    Backspace,
    
    Tab,
    
    Newline,
    
    CarriageReturn,
    
    ShowCursor,
    
    HideCursor,
    
    Unknown,
}






pub struct GraphicsTerminal {
    
    pub cols: u16,
    pub rows: u16,
    
    
    pub cell_width: u16,
    pub cell_height: u16,
    
    
    pub cursor_x: u16,
    pub cursor_y: u16,
    
    
    saved_cursor_x: u16,
    saved_cursor_y: u16,
    
    
    pub cursor_visible: bool,
    
    
    cursor_blink: bool,
    blink_counter: u32,
    
    
    grid: Vec<Cell>,
    
    
    scrollback: VecDeque<Vec<Cell>>,
    scrollback_max: usize,
    
    
    scroll_offset: usize,
    
    
    current_attr: CellAttr,
    current_fg: u32,
    current_bg: u32,
    
    
    parse_state: ParseState,
    csi_params: Vec<u16>,
    csi_buffer: String,
    osc_buffer: String,
    
    
    pub title: String,
    
    
    pub avh: Option<u32>,
    
    
    dirty: bool,
    
    
    pub glow_enabled: bool,
    
    
    pub scanline_intensity: u8,
}

impl GraphicsTerminal {
    
    pub fn new(width: u32, height: u32) -> Self {
        
        let cell_width = 8u16;
        let cell_height = 16u16;
        let cols = (width / cell_width as u32) as u16;
        let rows = (height / cell_height as u32) as u16;
        
        let eon = (cols as usize) * (rows as usize);
        let grid = vec![Cell::default(); eon];
        
        let mut wp = Self {
            cols,
            rows,
            cell_width,
            cell_height,
            cursor_x: 0,
            cursor_y: 0,
            saved_cursor_x: 0,
            saved_cursor_y: 0,
            cursor_visible: true,
            cursor_blink: true,
            blink_counter: 0,
            grid,
            scrollback: VecDeque::new(),
            scrollback_max: 1000,
            scroll_offset: 0,
            current_attr: CellAttr::default(),
            current_fg: CQ_,
            current_bg: KB_,
            parse_state: ParseState::Normal,
            csi_params: Vec::new(),
            csi_buffer: String::new(),
            osc_buffer: String::new(),
            title: String::from("TrustOS Terminal"),
            avh: None,
            dirty: true,
            glow_enabled: true,
            scanline_intensity: 20,
        };
        
        
        wp.write_str("\x1b[1;32m"); 
        wp.write_str("╔══════════════════════════════════════════════════════════╗\r\n");
        wp.write_str("║  \x1b[1;97mTrustOS\x1b[1;32m Graphical Terminal                             ║\r\n");
        wp.write_str("║  Matrix Edition v1.0                                     ║\r\n");
        wp.write_str("╚══════════════════════════════════════════════════════════╝\r\n");
        wp.write_str("\x1b[0;32m"); 
        wp.write_str("\r\n");
        
        wp
    }
    
    
    pub fn write_str(&mut self, j: &str) {
        for c in j.chars() {
            self.write_char(c);
        }
    }
    
    
    pub fn write_char(&mut self, c: char) {
        match self.parse_state {
            ParseState::Normal => self.handle_normal(c),
            ParseState::Escape => self.handle_escape(c),
            ParseState::Csi => self.handle_csi(c),
            ParseState::Osc => self.handle_osc(c),
            ParseState::Dcs => self.handle_dcs(c),
        }
        self.dirty = true;
    }
    
    fn handle_normal(&mut self, c: char) {
        match c {
            '\x1b' => {
                self.parse_state = ParseState::Escape;
            }
            '\n' => {
                self.newline();
            }
            '\r' => {
                self.cursor_x = 0;
            }
            '\x08' => {
                
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            '\t' => {
                
                let nkg = ((self.cursor_x / 8) + 1) * 8;
                self.cursor_x = nkg.min(self.cols - 1);
            }
            '\x07' => {
                
            }
            _ if c >= ' ' => {
                self.put_char(c);
            }
            _ => {
                
            }
        }
    }
    
    fn handle_escape(&mut self, c: char) {
        match c {
            '[' => {
                self.parse_state = ParseState::Csi;
                self.csi_params.clear();
                self.csi_buffer.clear();
            }
            ']' => {
                self.parse_state = ParseState::Osc;
                self.osc_buffer.clear();
            }
            'P' => {
                self.parse_state = ParseState::Dcs;
            }
            'c' => {
                
                self.reset();
                self.parse_state = ParseState::Normal;
            }
            '7' => {
                
                self.saved_cursor_x = self.cursor_x;
                self.saved_cursor_y = self.cursor_y;
                self.parse_state = ParseState::Normal;
            }
            '8' => {
                
                self.cursor_x = self.saved_cursor_x;
                self.cursor_y = self.saved_cursor_y;
                self.parse_state = ParseState::Normal;
            }
            'D' => {
                
                self.newline();
                self.parse_state = ParseState::Normal;
            }
            'M' => {
                
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;
                }
                self.parse_state = ParseState::Normal;
            }
            'E' => {
                
                self.cursor_x = 0;
                self.newline();
                self.parse_state = ParseState::Normal;
            }
            _ => {
                
                self.parse_state = ParseState::Normal;
            }
        }
    }
    
    fn handle_csi(&mut self, c: char) {
        if c.is_ascii_digit() || c == ';' {
            self.csi_buffer.push(c);
        } else {
            
            self.csi_params.clear();
            for jn in self.csi_buffer.split(';') {
                if let Ok(ae) = jn.parse::<u16>() {
                    self.csi_params.push(ae);
                } else {
                    self.csi_params.push(0);
                }
            }
            
            
            self.execute_csi(c);
            self.parse_state = ParseState::Normal;
        }
    }
    
    fn execute_csi(&mut self, cmd: char) {
        let params = &self.csi_params;
        let qm = params.first().copied().unwrap_or(1).max(1);
        let gw = params.get(1).copied().unwrap_or(1).max(1);
        
        match cmd {
            'A' => {
                
                self.cursor_y = self.cursor_y.saturating_sub(qm);
            }
            'B' => {
                
                self.cursor_y = (self.cursor_y + qm).min(self.rows - 1);
            }
            'C' => {
                
                self.cursor_x = (self.cursor_x + qm).min(self.cols - 1);
            }
            'D' => {
                
                self.cursor_x = self.cursor_x.saturating_sub(qm);
            }
            'E' => {
                
                self.cursor_x = 0;
                self.cursor_y = (self.cursor_y + qm).min(self.rows - 1);
            }
            'F' => {
                
                self.cursor_x = 0;
                self.cursor_y = self.cursor_y.saturating_sub(qm);
            }
            'G' => {
                
                self.cursor_x = (qm - 1).min(self.cols - 1);
            }
            'H' | 'f' => {
                
                let row = params.first().copied().unwrap_or(1).max(1);
                let col = params.get(1).copied().unwrap_or(1).max(1);
                self.cursor_y = (row - 1).min(self.rows - 1);
                self.cursor_x = (col - 1).min(self.cols - 1);
            }
            'J' => {
                
                let mode = params.first().copied().unwrap_or(0);
                match mode {
                    0 => self.erase_to_end_of_screen(),
                    1 => self.erase_to_start_of_screen(),
                    2 | 3 => self.erase_screen(),
                    _ => {}
                }
            }
            'K' => {
                
                let mode = params.first().copied().unwrap_or(0);
                match mode {
                    0 => self.erase_to_end_of_line(),
                    1 => self.erase_to_start_of_line(),
                    2 => self.erase_line(),
                    _ => {}
                }
            }
            'S' => {
                
                for _ in 0..qm {
                    self.scroll_up();
                }
            }
            'T' => {
                
                for _ in 0..qm {
                    self.scroll_down();
                }
            }
            'm' => {
                
                self.execute_sgr();
            }
            's' => {
                
                self.saved_cursor_x = self.cursor_x;
                self.saved_cursor_y = self.cursor_y;
            }
            'u' => {
                
                self.cursor_x = self.saved_cursor_x;
                self.cursor_y = self.saved_cursor_y;
            }
            '?' if !params.is_empty() => {
                
            }
            'h' => {
                
                if self.csi_buffer.starts_with('?') {
                    let mode = params.first().copied().unwrap_or(0);
                    if mode == 25 {
                        self.cursor_visible = true;
                    }
                }
            }
            'l' => {
                
                if self.csi_buffer.starts_with('?') {
                    let mode = params.first().copied().unwrap_or(0);
                    if mode == 25 {
                        self.cursor_visible = false;
                    }
                }
            }
            _ => {
                
            }
        }
    }
    
    fn execute_sgr(&mut self) {
        let params = if self.csi_params.is_empty() {
            vec![0] 
        } else {
            self.csi_params.clone()
        };
        
        let mut i = 0;
        while i < params.len() {
            let aa = params[i];
            match aa {
                0 => {
                    
                    self.current_attr = CellAttr::default();
                    self.current_fg = CQ_;
                    self.current_bg = KB_;
                }
                1 => self.current_attr.bold = true,
                2 => self.current_attr.dim = true,
                3 => self.current_attr.italic = true,
                4 => self.current_attr.underline = true,
                5 => self.current_attr.blink = true,
                7 => self.current_attr.reverse = true,
                8 => self.current_attr.hidden = true,
                9 => self.current_attr.strikethrough = true,
                21 => self.current_attr.bold = false,
                22 => {
                    self.current_attr.bold = false;
                    self.current_attr.dim = false;
                }
                23 => self.current_attr.italic = false,
                24 => self.current_attr.underline = false,
                25 => self.current_attr.blink = false,
                27 => self.current_attr.reverse = false,
                28 => self.current_attr.hidden = false,
                29 => self.current_attr.strikethrough = false,
                
                30..=37 => {
                    let idx = (aa - 30) as usize;
                    self.current_fg = MN_[idx];
                }
                38 => {
                    
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        
                        let idx = params[i + 2] as usize;
                        self.current_fg = self.color_256(idx);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        
                        let r = params[i + 2] as u32;
                        let g = params[i + 3] as u32;
                        let b = params[i + 4] as u32;
                        self.current_fg = 0xFF000000 | (r << 16) | (g << 8) | b;
                        i += 4;
                    }
                }
                39 => self.current_fg = CQ_, 
                
                40..=47 => {
                    let idx = (aa - 40) as usize;
                    self.current_bg = MN_[idx];
                }
                48 => {
                    
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        let idx = params[i + 2] as usize;
                        self.current_bg = self.color_256(idx);
                        i += 2;
                    } else if i + 4 < params.len() && params[i + 1] == 2 {
                        let r = params[i + 2] as u32;
                        let g = params[i + 3] as u32;
                        let b = params[i + 4] as u32;
                        self.current_bg = 0xFF000000 | (r << 16) | (g << 8) | b;
                        i += 4;
                    }
                }
                49 => self.current_bg = KB_, 
                
                90..=97 => {
                    let idx = (aa - 90 + 8) as usize;
                    self.current_fg = MN_[idx];
                }
                
                100..=107 => {
                    let idx = (aa - 100 + 8) as usize;
                    self.current_bg = MN_[idx];
                }
                _ => {}
            }
            i += 1;
        }
    }
    
    
    fn color_256(&self, idx: usize) -> u32 {
        if idx < 16 {
            MN_[idx]
        } else if idx < 232 {
            
            let idx = idx - 16;
            let r = (idx / 36) % 6;
            let g = (idx / 6) % 6;
            let b = idx % 6;
            let r = if r > 0 { r * 40 + 55 } else { 0 };
            let g = if g > 0 { g * 40 + 55 } else { 0 };
            let b = if b > 0 { b * 40 + 55 } else { 0 };
            0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        } else {
            
            let fzk = (idx - 232) * 10 + 8;
            0xFF000000 | ((fzk as u32) << 16) | ((fzk as u32) << 8) | (fzk as u32)
        }
    }
    
    fn handle_osc(&mut self, c: char) {
        if c == '\x07' || c == '\x1b' {
            
            self.execute_osc();
            self.parse_state = ParseState::Normal;
        } else {
            self.osc_buffer.push(c);
        }
    }
    
    fn execute_osc(&mut self) {
        
        if let Some(idx) = self.osc_buffer.find(';') {
            let cmd = &self.osc_buffer[..idx];
            let data = &self.osc_buffer[idx + 1..];
            
            match cmd {
                "0" | "2" => {
                    
                    self.title = String::from(data);
                }
                _ => {}
            }
        }
    }
    
    fn handle_dcs(&mut self, c: char) {
        
        if c == '\x1b' || c == '\\' {
            self.parse_state = ParseState::Normal;
        }
    }
    
    
    fn put_char(&mut self, c: char) {
        if self.cursor_x >= self.cols {
            self.cursor_x = 0;
            self.newline();
        }
        
        let idx = self.cursor_y as usize * self.cols as usize + self.cursor_x as usize;
        if idx < self.grid.len() {
            
            let (fg, bg) = if self.current_attr.reverse {
                (self.current_bg, self.current_fg)
            } else {
                (self.current_fg, self.current_bg)
            };
            
            let fg = if self.current_attr.bold {
                self.brighten(fg)
            } else if self.current_attr.dim {
                self.dim_color(fg)
            } else {
                fg
            };
            
            self.grid[idx] = Cell {
                ch: c,
                fg,
                bg,
                attr: self.current_attr,
            };
        }
        
        self.cursor_x += 1;
    }
    
    
    fn brighten(&self, color: u32) -> u32 {
        let r = ((color >> 16) & 0xFF).min(255);
        let g = ((color >> 8) & 0xFF).min(255);
        let b = (color & 0xFF).min(255);
        
        let r = (r + (255 - r) / 3).min(255);
        let g = (g + (255 - g) / 3).min(255);
        let b = (b + (255 - b) / 3).min(255);
        
        0xFF000000 | (r << 16) | (g << 8) | b
    }
    
    
    fn dim_color(&self, color: u32) -> u32 {
        let r = ((color >> 16) & 0xFF) * 2 / 3;
        let g = ((color >> 8) & 0xFF) * 2 / 3;
        let b = (color & 0xFF) * 2 / 3;
        
        0xFF000000 | (r << 16) | (g << 8) | b
    }
    
    
    fn newline(&mut self) {
        if self.cursor_y >= self.rows - 1 {
            self.scroll_up();
        } else {
            self.cursor_y += 1;
        }
    }
    
    
    fn scroll_up(&mut self) {
        
        let plg: Vec<Cell> = self.grid[..self.cols as usize].to_vec();
        self.scrollback.push_back(plg);
        
        
        while self.scrollback.len() > self.scrollback_max {
            self.scrollback.pop_front();
        }
        
        
        let cols = self.cols as usize;
        for y in 0..self.rows as usize - 1 {
            let zl = (y + 1) * cols;
            let alj = y * cols;
            for x in 0..cols {
                self.grid[alj + x] = self.grid[zl + x];
            }
        }
        
        
        let mwq = (self.rows as usize - 1) * cols;
        for x in 0..cols {
            self.grid[mwq + x] = Cell::default();
        }
    }
    
    
    fn scroll_down(&mut self) {
        let cols = self.cols as usize;
        
        
        for y in (1..self.rows as usize).rev() {
            let zl = (y - 1) * cols;
            let alj = y * cols;
            for x in 0..cols {
                self.grid[alj + x] = self.grid[zl + x];
            }
        }
        
        
        for x in 0..cols {
            self.grid[x] = Cell::default();
        }
    }
    
    
    fn erase_to_end_of_screen(&mut self) {
        
        self.erase_to_end_of_line();
        
        
        let cols = self.cols as usize;
        for y in (self.cursor_y + 1) as usize..self.rows as usize {
            let fk = y * cols;
            for x in 0..cols {
                self.grid[fk + x] = Cell::default();
            }
        }
    }
    
    
    fn erase_to_start_of_screen(&mut self) {
        
        self.erase_to_start_of_line();
        
        
        let cols = self.cols as usize;
        for y in 0..self.cursor_y as usize {
            let fk = y * cols;
            for x in 0..cols {
                self.grid[fk + x] = Cell::default();
            }
        }
    }
    
    
    fn erase_screen(&mut self) {
        for cell in &mut self.grid {
            *cell = Cell::default();
        }
    }
    
    
    fn erase_to_end_of_line(&mut self) {
        let fk = self.cursor_y as usize * self.cols as usize;
        for x in self.cursor_x as usize..self.cols as usize {
            self.grid[fk + x] = Cell::default();
        }
    }
    
    
    fn erase_to_start_of_line(&mut self) {
        let fk = self.cursor_y as usize * self.cols as usize;
        for x in 0..=self.cursor_x as usize {
            if fk + x < self.grid.len() {
                self.grid[fk + x] = Cell::default();
            }
        }
    }
    
    
    fn erase_line(&mut self) {
        let fk = self.cursor_y as usize * self.cols as usize;
        for x in 0..self.cols as usize {
            self.grid[fk + x] = Cell::default();
        }
    }
    
    
    pub fn reset(&mut self) {
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.current_attr = CellAttr::default();
        self.current_fg = CQ_;
        self.current_bg = KB_;
        self.cursor_visible = true;
        self.erase_screen();
    }
    
    
    pub fn handle_key(&mut self, c: char) {
        
        
        if c == '\n' {
            self.write_str("\r\n");
        } else {
            self.write_char(c);
        }
    }
    
    
    pub fn render(&mut self) -> Vec<u32> {
        let width = self.cols as usize * self.cell_width as usize;
        let height = self.rows as usize * self.cell_height as usize;
        
        
        let mut buffer = vec![0u32; width * height];
        crate::graphics::simd::hyi(&mut buffer, KB_);
        
        
        self.blink_counter = self.blink_counter.wrapping_add(1);
        if self.blink_counter % 30 == 0 {
            self.cursor_blink = !self.cursor_blink;
        }
        
        
        for y in 0..self.rows as usize {
            for x in 0..self.cols as usize {
                let idx = y * self.cols as usize + x;
                let cell = &self.grid[idx];
                
                
                self.draw_cell_bg_fast(&mut buffer, width, x as u32, y as u32, cell.bg);
                
                
                if cell.ch != ' ' {
                    self.draw_char(&mut buffer, width as u32, x as u32, y as u32, cell.ch, cell.fg);
                }
                
                
                if cell.attr.underline {
                    self.draw_underline_fast(&mut buffer, width, x as u32, y as u32, cell.fg);
                }
            }
        }
        
        
        if self.cursor_visible && self.cursor_blink {
            self.draw_cursor_fast(&mut buffer, width);
        }
        
        
        if self.glow_enabled {
            self.apply_glow(&mut buffer, width as u32, height as u32);
        }
        
        
        if self.scanline_intensity > 0 {
            self.apply_scanlines(&mut buffer, width as u32, height as u32);
        }
        
        self.dirty = false;
        buffer
    }
    
    
    fn draw_cell_bg_fast(&self, buffer: &mut [u32], width: usize, cx: u32, u: u32, bg: u32) {
        let amk = cx as usize * self.cell_width as usize;
        let aze = u as usize * self.cell_height as usize;
        let cell_w = self.cell_width as usize;
        
        
        for ad in 0..self.cell_height as usize {
            let fk = (aze + ad) * width + amk;
            if fk + cell_w <= buffer.len() {
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    crate::graphics::simd::adq(
                        buffer.as_mut_ptr().add(fk),
                        cell_w,
                        bg
                    );
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    buffer[fk..fk + cell_w].fill(bg);
                }
            }
        }
    }

    fn qdl(&self, buffer: &mut [u32], width: u32, cx: u32, u: u32, bg: u32) {
        let amk = cx * self.cell_width as u32;
        let aze = u * self.cell_height as u32;
        
        for ad in 0..self.cell_height as u32 {
            for dx in 0..self.cell_width as u32 {
                let idx = ((aze + ad) * width + amk + dx) as usize;
                if idx < buffer.len() {
                    buffer[idx] = bg;
                }
            }
        }
    }
    
    fn draw_char(&self, buffer: &mut [u32], width: u32, cx: u32, u: u32, c: char, fg: u32) {
        let du = crate::framebuffer::font::ol(c);
        let amk = cx * self.cell_width as u32;
        let aze = u * self.cell_height as u32;
        
        for (amq, &row) in du.iter().enumerate() {
            for bf in 0..8 {
                if (row >> (7 - bf)) & 1 == 1 {
                    let x = amk + bf;
                    let y = aze + amq as u32;
                    let idx = (y * width + x) as usize;
                    if idx < buffer.len() {
                        buffer[idx] = fg;
                    }
                }
            }
        }
    }
    
    fn qed(&self, buffer: &mut [u32], width: u32, cx: u32, u: u32, fg: u32) {
        let amk = cx * self.cell_width as u32;
        let aze = u * self.cell_height as u32 + self.cell_height as u32 - 2;
        
        for dx in 0..self.cell_width as u32 {
            let idx = (aze * width + amk + dx) as usize;
            if idx < buffer.len() {
                buffer[idx] = fg;
            }
        }
    }
    
    
    fn draw_underline_fast(&self, buffer: &mut [u32], width: usize, cx: u32, u: u32, fg: u32) {
        let amk = cx as usize * self.cell_width as usize;
        let aze = u as usize * self.cell_height as usize + self.cell_height as usize - 2;
        let cell_w = self.cell_width as usize;
        
        let start = aze * width + amk;
        if start + cell_w <= buffer.len() {
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::adq(
                    buffer.as_mut_ptr().add(start),
                    cell_w,
                    fg
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                buffer[start..start + cell_w].fill(fg);
            }
        }
    }
    
    fn draw_cursor(&self, buffer: &mut [u32], width: u32) {
        let amk = self.cursor_x as u32 * self.cell_width as u32;
        let aze = self.cursor_y as u32 * self.cell_height as u32;
        
        
        for ad in 0..self.cell_height as u32 {
            for dx in 0..self.cell_width as u32 {
                let idx = ((aze + ad) * width + amk + dx) as usize;
                if idx < buffer.len() {
                    
                    let ku = buffer[idx];
                    buffer[idx] = ku ^ ARU_;
                }
            }
        }
    }
    
    
    fn draw_cursor_fast(&self, buffer: &mut [u32], width: usize) {
        let amk = self.cursor_x as usize * self.cell_width as usize;
        let aze = self.cursor_y as usize * self.cell_height as usize;
        let cell_w = self.cell_width as usize;
        
        
        for ad in 0..self.cell_height as usize {
            let fk = (aze + ad) * width + amk;
            if fk + cell_w <= buffer.len() {
                for dx in 0..cell_w {
                    let ku = buffer[fk + dx];
                    buffer[fk + dx] = ku ^ ARU_;
                }
            }
        }
    }
    
    
    fn apply_glow(&self, buffer: &mut [u32], width: u32, height: u32) {
        
        
        
        let mut ts = buffer.to_vec();
        
        for y in 0..height {
            for x in 1..(width - 1) {
                let idx = (y * width + x) as usize;
                let left = buffer[(y * width + x - 1) as usize];
                let center = buffer[idx];
                let right = buffer[(y * width + x + 1) as usize];
                
                
                let ias = (left >> 8) & 0xFF;
                let iar = (center >> 8) & 0xFF;
                let iat = (right >> 8) & 0xFF;
                
                if iar > 100 || ias > 100 || iat > 100 {
                    let glow = ((ias + iar * 2 + iat) / 4).min(255);
                    let r = (center >> 16) & 0xFF;
                    let b = center & 0xFF;
                    ts[idx] = 0xFF000000 | (r << 16) | (glow << 8) | b;
                }
            }
        }
        
        buffer.copy_from_slice(&ts);
    }
    
    
    fn apply_scanlines(&self, buffer: &mut [u32], width: u32, height: u32) {
        let intensity = 255 - self.scanline_intensity as u32;
        
        for y in (1..height).step_by(2) {
            let fk = (y * width) as usize;
            let azm = ((y + 1) * width) as usize;
            
            if azm <= buffer.len() {
                for idx in fk..azm.min(fk + width as usize) {
                    let ct = buffer[idx];
                    let r = ((ct >> 16) & 0xFF) * intensity / 255;
                    let g = ((ct >> 8) & 0xFF) * intensity / 255;
                    let b = (ct & 0xFF) * intensity / 255;
                    buffer[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
    }
    
    
    pub fn qmi(&self) -> bool {
        self.dirty
    }
    
    
    pub fn pixel_size(&self) -> (u32, u32) {
        (
            self.cols as u32 * self.cell_width as u32,
            self.rows as u32 * self.cell_height as u32,
        )
    }
}





use spin::Mutex;

static OH_: Mutex<Option<GraphicsTerminal>> = Mutex::new(None);


pub fn init(width: u32, height: u32) -> Result<(), &'static str> {
    let mut wp = OH_.lock();
    if wp.is_some() {
        return Err("Graphics terminal already initialized");
    }
    
    *wp = Some(GraphicsTerminal::new(width, height));
    crate::serial_println!("[GTERM] Graphics terminal initialized ({}x{})", width, height);
    Ok(())
}


pub fn write(j: &str) {
    if let Some(wp) = OH_.lock().as_mut() {
        wp.write_str(j);
    }
}


pub fn render() -> Option<Vec<u32>> {
    OH_.lock().as_mut().map(|wp| wp.render())
}


pub fn handle_key(c: char) {
    if let Some(wp) = OH_.lock().as_mut() {
        wp.handle_key(c);
    }
}


pub fn cyt() -> Option<(u32, u32)> {
    OH_.lock().as_ref().map(|wp| wp.pixel_size())
}
