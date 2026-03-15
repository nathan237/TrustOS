//! Game Boy CPU — Sharp LR35902 (Z80-like)
//! All 245 regular opcodes + 256 CB-prefixed bit operations
#![allow(dead_code)]

pub const FLAG_Z: u8 = 0x80;
pub const FLAG_N: u8 = 0x40;
pub const FLAG_H: u8 = 0x20;
pub const FLAG_C: u8 = 0x10;

pub trait GbBus {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

pub struct Cpu {
    pub a: u8, pub f: u8,
    pub b: u8, pub c: u8,
    pub d: u8, pub e: u8,
    pub h: u8, pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub ime: bool,
    pub ime_next: bool,
    pub halted: bool,
    pub cycles: u64,
}

impl Cpu {
    pub fn new() -> Self {
        // Post-boot-ROM state (DMG)
        Self {
            a: 0x01, f: 0xB0,
            b: 0x00, c: 0x13,
            d: 0x00, e: 0xD8,
            h: 0x01, l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
            ime: true,
            ime_next: false,
            halted: false,
            cycles: 0,
        }
    }

    // 16-bit register pairs
    pub fn af(&self) -> u16 { (self.a as u16) << 8 | self.f as u16 }
    pub fn bc(&self) -> u16 { (self.b as u16) << 8 | self.c as u16 }
    pub fn de(&self) -> u16 { (self.d as u16) << 8 | self.e as u16 }
    pub fn hl(&self) -> u16 { (self.h as u16) << 8 | self.l as u16 }
    fn set_af(&mut self, v: u16) { self.a = (v >> 8) as u8; self.f = (v & 0xF0) as u8; }
    fn set_bc(&mut self, v: u16) { self.b = (v >> 8) as u8; self.c = (v & 0xFF) as u8; }
    fn set_de(&mut self, v: u16) { self.d = (v >> 8) as u8; self.e = (v & 0xFF) as u8; }
    fn set_hl(&mut self, v: u16) { self.h = (v >> 8) as u8; self.l = (v & 0xFF) as u8; }

    // Flag helpers
    fn zf(&self) -> bool { self.f & FLAG_Z != 0 }
    fn nf(&self) -> bool { self.f & FLAG_N != 0 }
    fn hf(&self) -> bool { self.f & FLAG_H != 0 }
    fn cf(&self) -> bool { self.f & FLAG_C != 0 }
    fn set_flags(&mut self, z: bool, n: bool, h: bool, c: bool) {
        self.f = (if z { FLAG_Z } else { 0 })
               | (if n { FLAG_N } else { 0 })
               | (if h { FLAG_H } else { 0 })
               | (if c { FLAG_C } else { 0 });
    }

    // Fetch
    fn fetch8(&mut self, bus: &mut impl GbBus) -> u8 {
        let v = bus.read(self.pc); self.pc = self.pc.wrapping_add(1); v
    }
    fn fetch16(&mut self, bus: &mut impl GbBus) -> u16 {
        let lo = bus.read(self.pc) as u16;
        let hi = bus.read(self.pc.wrapping_add(1)) as u16;
        self.pc = self.pc.wrapping_add(2);
        lo | (hi << 8)
    }

    // Stack
    fn push16(&mut self, bus: &mut impl GbBus, val: u16) {
        self.sp = self.sp.wrapping_sub(1); bus.write(self.sp, (val >> 8) as u8);
        self.sp = self.sp.wrapping_sub(1); bus.write(self.sp, val as u8);
    }
    fn pop16(&mut self, bus: &mut impl GbBus) -> u16 {
        let lo = bus.read(self.sp) as u16; self.sp = self.sp.wrapping_add(1);
        let hi = bus.read(self.sp) as u16; self.sp = self.sp.wrapping_add(1);
        lo | (hi << 8)
    }

    // Register index decode (bits 0-2 or 3-5 of opcode)
    fn get_reg(&self, r: u8, bus: &mut impl GbBus) -> u8 {
        match r & 7 {
            0 => self.b, 1 => self.c, 2 => self.d, 3 => self.e,
            4 => self.h, 5 => self.l, 6 => bus.read(self.hl()), 7 => self.a,
            _ => 0,
        }
    }
    fn set_reg(&mut self, r: u8, v: u8, bus: &mut impl GbBus) {
        match r & 7 {
            0 => self.b = v, 1 => self.c = v, 2 => self.d = v, 3 => self.e = v,
            4 => self.h = v, 5 => self.l = v, 6 => bus.write(self.hl(), v), 7 => self.a = v,
            _ => {}
        }
    }

    // ======== ALU operations ========
    fn alu_add(&mut self, v: u8) {
        let a = self.a; let r = a.wrapping_add(v);
        self.set_flags(r == 0, false, (a & 0xF) + (v & 0xF) > 0xF, (a as u16 + v as u16) > 0xFF);
        self.a = r;
    }
    fn alu_adc(&mut self, v: u8) {
        let a = self.a; let c = if self.cf() { 1u8 } else { 0 };
        let r = a.wrapping_add(v).wrapping_add(c);
        self.set_flags(r == 0, false, (a & 0xF) + (v & 0xF) + c > 0xF, a as u16 + v as u16 + c as u16 > 0xFF);
        self.a = r;
    }
    fn alu_sub(&mut self, v: u8) {
        let a = self.a; let r = a.wrapping_sub(v);
        self.set_flags(r == 0, true, (a & 0xF) < (v & 0xF), a < v);
        self.a = r;
    }
    fn alu_sbc(&mut self, v: u8) {
        let a = self.a; let c = if self.cf() { 1u8 } else { 0 };
        let r = a.wrapping_sub(v).wrapping_sub(c);
        self.set_flags(r == 0, true, (a & 0xF) < (v & 0xF).wrapping_add(c), (a as u16) < v as u16 + c as u16);
        self.a = r;
    }
    fn alu_and(&mut self, v: u8) { self.a &= v; self.set_flags(self.a == 0, false, true, false); }
    fn alu_xor(&mut self, v: u8) { self.a ^= v; self.set_flags(self.a == 0, false, false, false); }
    fn alu_or(&mut self, v: u8)  { self.a |= v; self.set_flags(self.a == 0, false, false, false); }
    fn alu_cp(&mut self, v: u8) {
        let a = self.a; let r = a.wrapping_sub(v);
        self.set_flags(r == 0, true, (a & 0xF) < (v & 0xF), a < v);
    }

    // ======== CB prefix helpers ========
    fn cb_rlc(&mut self, v: u8) -> u8 { let c = v >> 7; let r = (v << 1) | c; self.set_flags(r == 0, false, false, c != 0); r }
    fn cb_rrc(&mut self, v: u8) -> u8 { let c = v & 1; let r = (v >> 1) | (c << 7); self.set_flags(r == 0, false, false, c != 0); r }
    fn cb_rl(&mut self, v: u8) -> u8 { let oc = if self.cf() { 1u8 } else { 0 }; let c = v >> 7; let r = (v << 1) | oc; self.set_flags(r == 0, false, false, c != 0); r }
    fn cb_rr(&mut self, v: u8) -> u8 { let oc = if self.cf() { 1u8 } else { 0 }; let c = v & 1; let r = (v >> 1) | (oc << 7); self.set_flags(r == 0, false, false, c != 0); r }
    fn cb_sla(&mut self, v: u8) -> u8 { let c = v >> 7; let r = v << 1; self.set_flags(r == 0, false, false, c != 0); r }
    fn cb_sra(&mut self, v: u8) -> u8 { let c = v & 1; let r = (v >> 1) | (v & 0x80); self.set_flags(r == 0, false, false, c != 0); r }
    fn cb_swap(&mut self, v: u8) -> u8 { let r = (v >> 4) | (v << 4); self.set_flags(r == 0, false, false, false); r }
    fn cb_srl(&mut self, v: u8) -> u8 { let c = v & 1; let r = v >> 1; self.set_flags(r == 0, false, false, c != 0); r }

    // ======== Other helpers ========
    fn inc8(&mut self, v: u8) -> u8 {
        let r = v.wrapping_add(1);
        self.f = (if r == 0 { FLAG_Z } else { 0 }) | (if (v & 0xF) + 1 > 0xF { FLAG_H } else { 0 }) | (self.f & FLAG_C);
        r
    }
    fn dec8(&mut self, v: u8) -> u8 {
        let r = v.wrapping_sub(1);
        self.f = (if r == 0 { FLAG_Z } else { 0 }) | FLAG_N | (if v & 0xF == 0 { FLAG_H } else { 0 }) | (self.f & FLAG_C);
        r
    }
    fn add_hl(&mut self, v: u16) {
        let hl = self.hl(); let r = hl.wrapping_add(v);
        self.f = (self.f & FLAG_Z) | (if (hl & 0xFFF) + (v & 0xFFF) > 0xFFF { FLAG_H } else { 0 }) | (if hl as u32 + v as u32 > 0xFFFF { FLAG_C } else { 0 });
        self.set_hl(r);
    }
    fn daa(&mut self) {
        let mut a = self.a as i32;
        if self.nf() {
            if self.hf() { a = (a.wrapping_sub(6)) & 0xFF; }
            if self.cf() { a = a.wrapping_sub(0x60); }
        } else {
            if self.hf() || (a & 0xF) > 9 { a = a.wrapping_add(6); }
            if self.cf() || a > 0x9F { a = a.wrapping_add(0x60); }
        }
        let c = self.cf() || a > 0xFF;
        self.a = a as u8;
        self.f = (if self.a == 0 { FLAG_Z } else { 0 }) | (self.f & FLAG_N) | (if c { FLAG_C } else { 0 });
    }

    // ========================================================
    // Main step — returns M-cycles consumed (1 M-cycle = 4 T-states)
    // ========================================================
    pub fn step(&mut self, bus: &mut impl GbBus) -> u32 {
        // Pending EI
        if self.ime_next { self.ime = true; self.ime_next = false; }

        // Interrupts
        if self.ime || self.halted {
            let ie = bus.read(0xFFFF);
            let iflag = bus.read(0xFF0F);
            let pending = ie & iflag & 0x1F;
            if pending != 0 {
                self.halted = false;
                if self.ime {
                    self.ime = false;
                    for bit in 0u16..5 {
                        if pending & (1 << bit) != 0 {
                            bus.write(0xFF0F, iflag & !(1 << bit as u8));
                            self.push16(bus, self.pc);
                            self.pc = 0x0040 + bit * 8;
                            self.cycles += 5;
                            return 5;
                        }
                    }
                }
            }
        }
        if self.halted { self.cycles += 1; return 1; }

        let op = self.fetch8(bus);
        let m = self.execute(op, bus);
        self.cycles += m as u64;
        m
    }

    fn execute(&mut self, op: u8, bus: &mut impl GbBus) -> u32 {
        match op {
            // ===== 0x00-0x0F =====
            0x00 => 1,
            0x01 => { let v = self.fetch16(bus); self.set_bc(v); 3 }
            0x02 => { bus.write(self.bc(), self.a); 2 }
            0x03 => { let v = self.bc().wrapping_add(1); self.set_bc(v); 2 }
            0x04 => { self.b = self.inc8(self.b); 1 }
            0x05 => { self.b = self.dec8(self.b); 1 }
            0x06 => { self.b = self.fetch8(bus); 2 }
            0x07 => { let c = self.a >> 7; self.a = (self.a << 1) | c; self.set_flags(false, false, false, c != 0); 1 }
            0x08 => { let a = self.fetch16(bus); bus.write(a, self.sp as u8); bus.write(a.wrapping_add(1), (self.sp >> 8) as u8); 5 }
            0x09 => { self.add_hl(self.bc()); 2 }
            0x0A => { self.a = bus.read(self.bc()); 2 }
            0x0B => { let v = self.bc().wrapping_sub(1); self.set_bc(v); 2 }
            0x0C => { self.c = self.inc8(self.c); 1 }
            0x0D => { self.c = self.dec8(self.c); 1 }
            0x0E => { self.c = self.fetch8(bus); 2 }
            0x0F => { let c = self.a & 1; self.a = (self.a >> 1) | (c << 7); self.set_flags(false, false, false, c != 0); 1 }

            // ===== 0x10-0x1F =====
            0x10 => {
                // STOP: CGB speed switch if KEY1 bit 0 set
                let key1 = bus.read(0xFF4D);
                if key1 & 0x01 != 0 {
                    // Toggle speed (bit 7) and clear prepare flag (bit 0)
                    bus.write(0xFF4D, (key1 ^ 0x80) & !0x01);
                }
                self.pc = self.pc.wrapping_add(1);
                1
            }
            0x11 => { let v = self.fetch16(bus); self.set_de(v); 3 }
            0x12 => { bus.write(self.de(), self.a); 2 }
            0x13 => { let v = self.de().wrapping_add(1); self.set_de(v); 2 }
            0x14 => { self.d = self.inc8(self.d); 1 }
            0x15 => { self.d = self.dec8(self.d); 1 }
            0x16 => { self.d = self.fetch8(bus); 2 }
            0x17 => { let oc = if self.cf() { 1u8 } else { 0 }; let c = self.a >> 7; self.a = (self.a << 1) | oc; self.set_flags(false, false, false, c != 0); 1 }
            0x18 => { let e = self.fetch8(bus) as i8; self.pc = self.pc.wrapping_add(e as u16); 3 }
            0x19 => { self.add_hl(self.de()); 2 }
            0x1A => { self.a = bus.read(self.de()); 2 }
            0x1B => { let v = self.de().wrapping_sub(1); self.set_de(v); 2 }
            0x1C => { self.e = self.inc8(self.e); 1 }
            0x1D => { self.e = self.dec8(self.e); 1 }
            0x1E => { self.e = self.fetch8(bus); 2 }
            0x1F => { let oc = if self.cf() { 1u8 } else { 0 }; let c = self.a & 1; self.a = (self.a >> 1) | (oc << 7); self.set_flags(false, false, false, c != 0); 1 }

            // ===== 0x20-0x2F =====
            0x20 => { let e = self.fetch8(bus) as i8; if !self.zf() { self.pc = self.pc.wrapping_add(e as u16); 3 } else { 2 } }
            0x21 => { let v = self.fetch16(bus); self.set_hl(v); 3 }
            0x22 => { let hl = self.hl(); bus.write(hl, self.a); self.set_hl(hl.wrapping_add(1)); 2 }
            0x23 => { let v = self.hl().wrapping_add(1); self.set_hl(v); 2 }
            0x24 => { self.h = self.inc8(self.h); 1 }
            0x25 => { self.h = self.dec8(self.h); 1 }
            0x26 => { self.h = self.fetch8(bus); 2 }
            0x27 => { self.daa(); 1 }
            0x28 => { let e = self.fetch8(bus) as i8; if self.zf() { self.pc = self.pc.wrapping_add(e as u16); 3 } else { 2 } }
            0x29 => { let hl = self.hl(); self.add_hl(hl); 2 }
            0x2A => { let hl = self.hl(); self.a = bus.read(hl); self.set_hl(hl.wrapping_add(1)); 2 }
            0x2B => { let v = self.hl().wrapping_sub(1); self.set_hl(v); 2 }
            0x2C => { self.l = self.inc8(self.l); 1 }
            0x2D => { self.l = self.dec8(self.l); 1 }
            0x2E => { self.l = self.fetch8(bus); 2 }
            0x2F => { self.a = !self.a; self.f = (self.f & (FLAG_Z | FLAG_C)) | FLAG_N | FLAG_H; 1 }

            // ===== 0x30-0x3F =====
            0x30 => { let e = self.fetch8(bus) as i8; if !self.cf() { self.pc = self.pc.wrapping_add(e as u16); 3 } else { 2 } }
            0x31 => { self.sp = self.fetch16(bus); 3 }
            0x32 => { let hl = self.hl(); bus.write(hl, self.a); self.set_hl(hl.wrapping_sub(1)); 2 }
            0x33 => { self.sp = self.sp.wrapping_add(1); 2 }
            0x34 => { let hl = self.hl(); let v = self.inc8(bus.read(hl)); bus.write(hl, v); 3 }
            0x35 => { let hl = self.hl(); let v = self.dec8(bus.read(hl)); bus.write(hl, v); 3 }
            0x36 => { let v = self.fetch8(bus); bus.write(self.hl(), v); 3 }
            0x37 => { self.f = (self.f & FLAG_Z) | FLAG_C; 1 }
            0x38 => { let e = self.fetch8(bus) as i8; if self.cf() { self.pc = self.pc.wrapping_add(e as u16); 3 } else { 2 } }
            0x39 => { self.add_hl(self.sp); 2 }
            0x3A => { let hl = self.hl(); self.a = bus.read(hl); self.set_hl(hl.wrapping_sub(1)); 2 }
            0x3B => { self.sp = self.sp.wrapping_sub(1); 2 }
            0x3C => { self.a = self.inc8(self.a); 1 }
            0x3D => { self.a = self.dec8(self.a); 1 }
            0x3E => { self.a = self.fetch8(bus); 2 }
            0x3F => { let c = !self.cf(); self.f = (self.f & FLAG_Z) | if c { FLAG_C } else { 0 }; 1 }

            // ===== 0x40-0x7F: LD r,r (+ HALT) =====
            0x76 => { self.halted = true; 1 }
            0x40..=0x75 | 0x77..=0x7F => {
                let dst = (op >> 3) & 7;
                let src = op & 7;
                let v = self.get_reg(src, bus);
                self.set_reg(dst, v, bus);
                if src == 6 || dst == 6 { 2 } else { 1 }
            }

            // ===== 0x80-0xBF: ALU A,r =====
            0x80..=0x87 => { let v = self.get_reg(op & 7, bus); self.alu_add(v); if op & 7 == 6 { 2 } else { 1 } }
            0x88..=0x8F => { let v = self.get_reg(op & 7, bus); self.alu_adc(v); if op & 7 == 6 { 2 } else { 1 } }
            0x90..=0x97 => { let v = self.get_reg(op & 7, bus); self.alu_sub(v); if op & 7 == 6 { 2 } else { 1 } }
            0x98..=0x9F => { let v = self.get_reg(op & 7, bus); self.alu_sbc(v); if op & 7 == 6 { 2 } else { 1 } }
            0xA0..=0xA7 => { let v = self.get_reg(op & 7, bus); self.alu_and(v); if op & 7 == 6 { 2 } else { 1 } }
            0xA8..=0xAF => { let v = self.get_reg(op & 7, bus); self.alu_xor(v); if op & 7 == 6 { 2 } else { 1 } }
            0xB0..=0xB7 => { let v = self.get_reg(op & 7, bus); self.alu_or(v); if op & 7 == 6 { 2 } else { 1 } }
            0xB8..=0xBF => { let v = self.get_reg(op & 7, bus); self.alu_cp(v); if op & 7 == 6 { 2 } else { 1 } }

            // ===== 0xC0-0xFF: Control flow, stack, misc =====
            0xC0 => { if !self.zf() { self.pc = self.pop16(bus); 5 } else { 2 } }
            0xC1 => { let v = self.pop16(bus); self.set_bc(v); 3 }
            0xC2 => { let a = self.fetch16(bus); if !self.zf() { self.pc = a; 4 } else { 3 } }
            0xC3 => { self.pc = self.fetch16(bus); 4 }
            0xC4 => { let a = self.fetch16(bus); if !self.zf() { self.push16(bus, self.pc); self.pc = a; 6 } else { 3 } }
            0xC5 => { let v = self.bc(); self.push16(bus, v); 4 }
            0xC6 => { let v = self.fetch8(bus); self.alu_add(v); 2 }
            0xC7 => { self.push16(bus, self.pc); self.pc = 0x00; 4 }
            0xC8 => { if self.zf() { self.pc = self.pop16(bus); 5 } else { 2 } }
            0xC9 => { self.pc = self.pop16(bus); 4 }
            0xCA => { let a = self.fetch16(bus); if self.zf() { self.pc = a; 4 } else { 3 } }
            0xCB => { return self.execute_cb(bus); }
            0xCC => { let a = self.fetch16(bus); if self.zf() { self.push16(bus, self.pc); self.pc = a; 6 } else { 3 } }
            0xCD => { let a = self.fetch16(bus); self.push16(bus, self.pc); self.pc = a; 6 }
            0xCE => { let v = self.fetch8(bus); self.alu_adc(v); 2 }
            0xCF => { self.push16(bus, self.pc); self.pc = 0x08; 4 }

            0xD0 => { if !self.cf() { self.pc = self.pop16(bus); 5 } else { 2 } }
            0xD1 => { let v = self.pop16(bus); self.set_de(v); 3 }
            0xD2 => { let a = self.fetch16(bus); if !self.cf() { self.pc = a; 4 } else { 3 } }
            0xD4 => { let a = self.fetch16(bus); if !self.cf() { self.push16(bus, self.pc); self.pc = a; 6 } else { 3 } }
            0xD5 => { let v = self.de(); self.push16(bus, v); 4 }
            0xD6 => { let v = self.fetch8(bus); self.alu_sub(v); 2 }
            0xD7 => { self.push16(bus, self.pc); self.pc = 0x10; 4 }
            0xD8 => { if self.cf() { self.pc = self.pop16(bus); 5 } else { 2 } }
            0xD9 => { self.pc = self.pop16(bus); self.ime = true; 4 }
            0xDA => { let a = self.fetch16(bus); if self.cf() { self.pc = a; 4 } else { 3 } }
            0xDC => { let a = self.fetch16(bus); if self.cf() { self.push16(bus, self.pc); self.pc = a; 6 } else { 3 } }
            0xDE => { let v = self.fetch8(bus); self.alu_sbc(v); 2 }
            0xDF => { self.push16(bus, self.pc); self.pc = 0x18; 4 }

            0xE0 => { let n = self.fetch8(bus); bus.write(0xFF00 | n as u16, self.a); 3 }
            0xE1 => { let v = self.pop16(bus); self.set_hl(v); 3 }
            0xE2 => { bus.write(0xFF00 | self.c as u16, self.a); 2 }
            0xE5 => { let v = self.hl(); self.push16(bus, v); 4 }
            0xE6 => { let v = self.fetch8(bus); self.alu_and(v); 2 }
            0xE7 => { self.push16(bus, self.pc); self.pc = 0x20; 4 }
            0xE8 => {
                let e = self.fetch8(bus) as i8 as i16 as u16;
                let sp = self.sp;
                self.set_flags(false, false, (sp & 0xF) + (e & 0xF) > 0xF, (sp & 0xFF) + (e & 0xFF) > 0xFF);
                self.sp = sp.wrapping_add(e);
                4
            }
            0xE9 => { self.pc = self.hl(); 1 }
            0xEA => { let a = self.fetch16(bus); bus.write(a, self.a); 4 }
            0xEE => { let v = self.fetch8(bus); self.alu_xor(v); 2 }
            0xEF => { self.push16(bus, self.pc); self.pc = 0x28; 4 }

            0xF0 => { let n = self.fetch8(bus); self.a = bus.read(0xFF00 | n as u16); 3 }
            0xF1 => { let v = self.pop16(bus); self.set_af(v); 3 }
            0xF2 => { self.a = bus.read(0xFF00 | self.c as u16); 2 }
            0xF3 => { self.ime = false; 1 }
            0xF5 => { let v = self.af(); self.push16(bus, v); 4 }
            0xF6 => { let v = self.fetch8(bus); self.alu_or(v); 2 }
            0xF7 => { self.push16(bus, self.pc); self.pc = 0x30; 4 }
            0xF8 => {
                let e = self.fetch8(bus) as i8 as i16 as u16;
                let sp = self.sp;
                self.set_flags(false, false, (sp & 0xF) + (e & 0xF) > 0xF, (sp & 0xFF) + (e & 0xFF) > 0xFF);
                self.set_hl(sp.wrapping_add(e));
                3
            }
            0xF9 => { self.sp = self.hl(); 2 }
            0xFA => { let a = self.fetch16(bus); self.a = bus.read(a); 4 }
            0xFB => { self.ime_next = true; 1 }
            0xFE => { let v = self.fetch8(bus); self.alu_cp(v); 2 }
            0xFF => { self.push16(bus, self.pc); self.pc = 0x38; 4 }

            _ => 1, // Illegal → NOP
        }
    }

    // ======== CB-prefix (256 bit/shift/rotate opcodes) ========
    fn execute_cb(&mut self, bus: &mut impl GbBus) -> u32 {
        let cb = self.fetch8(bus);
        let r = cb & 7;
        let v = self.get_reg(r, bus);
        let is_hl = r == 6;

        match cb {
            0x00..=0x07 => { let res = self.cb_rlc(v); self.set_reg(r, res, bus); }
            0x08..=0x0F => { let res = self.cb_rrc(v); self.set_reg(r, res, bus); }
            0x10..=0x17 => { let res = self.cb_rl(v); self.set_reg(r, res, bus); }
            0x18..=0x1F => { let res = self.cb_rr(v); self.set_reg(r, res, bus); }
            0x20..=0x27 => { let res = self.cb_sla(v); self.set_reg(r, res, bus); }
            0x28..=0x2F => { let res = self.cb_sra(v); self.set_reg(r, res, bus); }
            0x30..=0x37 => { let res = self.cb_swap(v); self.set_reg(r, res, bus); }
            0x38..=0x3F => { let res = self.cb_srl(v); self.set_reg(r, res, bus); }
            0x40..=0x7F => {
                let bit = (cb >> 3) & 7;
                let z = (v >> bit) & 1 == 0;
                self.f = (if z { FLAG_Z } else { 0 }) | FLAG_H | (self.f & FLAG_C);
                return if is_hl { 3 } else { 2 };
            }
            0x80..=0xBF => {
                let bit = (cb >> 3) & 7;
                self.set_reg(r, v & !(1 << bit), bus);
            }
            0xC0..=0xFF => {
                let bit = (cb >> 3) & 7;
                self.set_reg(r, v | (1 << bit), bus);
            }
        }
        if is_hl { 4 } else { 2 }
    }
}
