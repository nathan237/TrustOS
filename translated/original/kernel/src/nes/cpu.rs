//! NES CPU — MOS 6502 emulation (all 151 official opcodes + common unofficial)
#![allow(dead_code)]

// Status register flags
const FLAG_C: u8 = 0x01; // Carry
const FLAG_Z: u8 = 0x02; // Zero
const FLAG_I: u8 = 0x04; // Interrupt disable
const FLAG_D: u8 = 0x08; // Decimal (unused on NES)
const FLAG_B: u8 = 0x10; // Break
const FLAG_U: u8 = 0x20; // Unused (always 1)
const FLAG_V: u8 = 0x40; // Overflow
const FLAG_N: u8 = 0x80; // Negative

pub trait CpuBus {
    fn cpu_read(&mut self, addr: u16) -> u8;
    fn cpu_write(&mut self, addr: u16, val: u8);
}

#[derive(Clone)]
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub status: u8,
    pub cycles: u64,
    pub nmi_pending: bool,
    pub irq_pending: bool,
    pub stall: u32,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0, x: 0, y: 0,
            sp: 0xFD,
            pc: 0,
            status: FLAG_U | FLAG_I,
            cycles: 0,
            nmi_pending: false,
            irq_pending: false,
            stall: 0,
        }
    }

    pub fn reset(&mut self, bus: &mut impl CpuBus) {
        let lo = bus.cpu_read(0xFFFC) as u16;
        let hi = bus.cpu_read(0xFFFD) as u16;
        self.pc = (hi << 8) | lo;
        self.sp = 0xFD;
        self.status = FLAG_U | FLAG_I;
    }

    // ======================== Helpers ========================

    fn flag(&self, f: u8) -> bool { self.status & f != 0 }
    fn set_flag(&mut self, f: u8, on: bool) { if on { self.status |= f; } else { self.status &= !f; } }
    fn set_zn(&mut self, v: u8) { self.set_flag(FLAG_Z, v == 0); self.set_flag(FLAG_N, v & 0x80 != 0); }

    fn push8(&mut self, bus: &mut impl CpuBus, val: u8) {
        bus.cpu_write(0x0100 | self.sp as u16, val);
        self.sp = self.sp.wrapping_sub(1);
    }
    fn push16(&mut self, bus: &mut impl CpuBus, val: u16) {
        self.push8(bus, (val >> 8) as u8);
        self.push8(bus, val as u8);
    }
    fn pull8(&mut self, bus: &mut impl CpuBus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        bus.cpu_read(0x0100 | self.sp as u16)
    }
    fn pull16(&mut self, bus: &mut impl CpuBus) -> u16 {
        let lo = self.pull8(bus) as u16;
        let hi = self.pull8(bus) as u16;
        (hi << 8) | lo
    }

    // ======================== Addressing Modes ========================

    fn read8(&mut self, bus: &mut impl CpuBus) -> u8 {
        let v = bus.cpu_read(self.pc); self.pc = self.pc.wrapping_add(1); v
    }
    fn read16(&mut self, bus: &mut impl CpuBus) -> u16 {
        let lo = bus.cpu_read(self.pc) as u16;
        let hi = bus.cpu_read(self.pc.wrapping_add(1)) as u16;
        self.pc = self.pc.wrapping_add(2);
        (hi << 8) | lo
    }
    fn read16_bug(&self, bus: &mut impl CpuBus, addr: u16) -> u16 {
        // 6502 indirect JMP bug: wraps within page
        let lo = bus.cpu_read(addr) as u16;
        let hi_addr = (addr & 0xFF00) | ((addr + 1) & 0x00FF);
        let hi = bus.cpu_read(hi_addr) as u16;
        (hi << 8) | lo
    }

    // Addressing mode: return value
    fn imm(&mut self, bus: &mut impl CpuBus) -> u8 { self.read8(bus) }
    fn zp_r(&mut self, bus: &mut impl CpuBus) -> u8 { let a = self.read8(bus) as u16; bus.cpu_read(a) }
    fn zpx_r(&mut self, bus: &mut impl CpuBus) -> u8 { let a = self.read8(bus).wrapping_add(self.x) as u16; bus.cpu_read(a) }
    fn zpy_r(&mut self, bus: &mut impl CpuBus) -> u8 { let a = self.read8(bus).wrapping_add(self.y) as u16; bus.cpu_read(a) }
    fn abs_r(&mut self, bus: &mut impl CpuBus) -> u8 { let a = self.read16(bus); bus.cpu_read(a) }
    fn abx_r(&mut self, bus: &mut impl CpuBus) -> (u8, u32) {
        let base = self.read16(bus); let a = base.wrapping_add(self.x as u16);
        let p = if (base & 0xFF00) != (a & 0xFF00) { 1 } else { 0 };
        (bus.cpu_read(a), p)
    }
    fn aby_r(&mut self, bus: &mut impl CpuBus) -> (u8, u32) {
        let base = self.read16(bus); let a = base.wrapping_add(self.y as u16);
        let p = if (base & 0xFF00) != (a & 0xFF00) { 1 } else { 0 };
        (bus.cpu_read(a), p)
    }
    fn izx_r(&mut self, bus: &mut impl CpuBus) -> u8 {
        let z = self.read8(bus).wrapping_add(self.x);
        let lo = bus.cpu_read(z as u16) as u16;
        let hi = bus.cpu_read(z.wrapping_add(1) as u16) as u16;
        bus.cpu_read((hi << 8) | lo)
    }
    fn izy_r(&mut self, bus: &mut impl CpuBus) -> (u8, u32) {
        let z = self.read8(bus);
        let lo = bus.cpu_read(z as u16) as u16;
        let hi = bus.cpu_read(z.wrapping_add(1) as u16) as u16;
        let base = (hi << 8) | lo;
        let a = base.wrapping_add(self.y as u16);
        let p = if (base & 0xFF00) != (a & 0xFF00) { 1 } else { 0 };
        (bus.cpu_read(a), p)
    }

    // Addressing mode: return address
    fn zp_a(&mut self, bus: &mut impl CpuBus) -> u16 { self.read8(bus) as u16 }
    fn zpx_a(&mut self, bus: &mut impl CpuBus) -> u16 { self.read8(bus).wrapping_add(self.x) as u16 }
    fn zpy_a(&mut self, bus: &mut impl CpuBus) -> u16 { self.read8(bus).wrapping_add(self.y) as u16 }
    fn abs_a(&mut self, bus: &mut impl CpuBus) -> u16 { self.read16(bus) }
    fn abx_a(&mut self, bus: &mut impl CpuBus) -> u16 { let b = self.read16(bus); b.wrapping_add(self.x as u16) }
    fn aby_a(&mut self, bus: &mut impl CpuBus) -> u16 { let b = self.read16(bus); b.wrapping_add(self.y as u16) }
    fn izx_a(&mut self, bus: &mut impl CpuBus) -> u16 {
        let z = self.read8(bus).wrapping_add(self.x);
        let lo = bus.cpu_read(z as u16) as u16;
        let hi = bus.cpu_read(z.wrapping_add(1) as u16) as u16;
        (hi << 8) | lo
    }
    fn izy_a(&mut self, bus: &mut impl CpuBus) -> u16 {
        let z = self.read8(bus);
        let lo = bus.cpu_read(z as u16) as u16;
        let hi = bus.cpu_read(z.wrapping_add(1) as u16) as u16;
        let base = (hi << 8) | lo;
        base.wrapping_add(self.y as u16)
    }

    // ======================== Instructions ========================

    fn adc(&mut self, v: u8) {
        let a = self.a as u16;
        let m = v as u16;
        let c = if self.flag(FLAG_C) { 1u16 } else { 0 };
        let sum = a + m + c;
        self.set_flag(FLAG_C, sum > 0xFF);
        let result = sum as u8;
        self.set_flag(FLAG_V, (!(self.a ^ v) & (self.a ^ result)) & 0x80 != 0);
        self.a = result;
        self.set_zn(self.a);
    }

    fn sbc(&mut self, v: u8) { self.adc(!v); }

    fn cmp_reg(&mut self, reg: u8, v: u8) {
        let r = reg.wrapping_sub(v);
        self.set_flag(FLAG_C, reg >= v);
        self.set_zn(r);
    }

    fn branch(&mut self, bus: &mut impl CpuBus, cond: bool) -> u32 {
        let offset = self.read8(bus) as i8;
        if cond {
            let new_pc = self.pc.wrapping_add(offset as u16);
            let penalty = if (self.pc & 0xFF00) != (new_pc & 0xFF00) { 1 } else { 0 };
            self.pc = new_pc;
            3 + penalty
        } else { 2 }
    }

    fn asl_val(&mut self, v: u8) -> u8 {
        self.set_flag(FLAG_C, v & 0x80 != 0);
        let r = v << 1; self.set_zn(r); r
    }
    fn lsr_val(&mut self, v: u8) -> u8 {
        self.set_flag(FLAG_C, v & 0x01 != 0);
        let r = v >> 1; self.set_zn(r); r
    }
    fn rol_val(&mut self, v: u8) -> u8 {
        let c = if self.flag(FLAG_C) { 1u8 } else { 0 };
        self.set_flag(FLAG_C, v & 0x80 != 0);
        let r = (v << 1) | c; self.set_zn(r); r
    }
    fn ror_val(&mut self, v: u8) -> u8 {
        let c = if self.flag(FLAG_C) { 0x80u8 } else { 0 };
        self.set_flag(FLAG_C, v & 0x01 != 0);
        let r = (v >> 1) | c; self.set_zn(r); r
    }

    // ======================== Step ========================

    pub fn step(&mut self, bus: &mut impl CpuBus) -> u32 {
        if self.stall > 0 { self.stall -= 1; return 1; }

        // Handle NMI
        if self.nmi_pending {
            self.nmi_pending = false;
            self.push16(bus, self.pc);
            self.push8(bus, (self.status | FLAG_U) & !FLAG_B);
            self.set_flag(FLAG_I, true);
            let lo = bus.cpu_read(0xFFFA) as u16;
            let hi = bus.cpu_read(0xFFFB) as u16;
            self.pc = (hi << 8) | lo;
            self.cycles += 7;
            return 7;
        }

        // Handle IRQ
        if self.irq_pending && !self.flag(FLAG_I) {
            self.irq_pending = false;
            self.push16(bus, self.pc);
            self.push8(bus, (self.status | FLAG_U) & !FLAG_B);
            self.set_flag(FLAG_I, true);
            let lo = bus.cpu_read(0xFFFE) as u16;
            let hi = bus.cpu_read(0xFFFF) as u16;
            self.pc = (hi << 8) | lo;
            self.cycles += 7;
            return 7;
        }

        let opcode = bus.cpu_read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        let cycles = match opcode {
            // ===== LDA =====
            0xA9 => { let v = self.imm(bus); self.a = v; self.set_zn(v); 2 }
            0xA5 => { let v = self.zp_r(bus); self.a = v; self.set_zn(v); 3 }
            0xB5 => { let v = self.zpx_r(bus); self.a = v; self.set_zn(v); 4 }
            0xAD => { let v = self.abs_r(bus); self.a = v; self.set_zn(v); 4 }
            0xBD => { let (v, p) = self.abx_r(bus); self.a = v; self.set_zn(v); 4 + p }
            0xB9 => { let (v, p) = self.aby_r(bus); self.a = v; self.set_zn(v); 4 + p }
            0xA1 => { let v = self.izx_r(bus); self.a = v; self.set_zn(v); 6 }
            0xB1 => { let (v, p) = self.izy_r(bus); self.a = v; self.set_zn(v); 5 + p }

            // ===== LDX =====
            0xA2 => { let v = self.imm(bus); self.x = v; self.set_zn(v); 2 }
            0xA6 => { let v = self.zp_r(bus); self.x = v; self.set_zn(v); 3 }
            0xB6 => { let v = self.zpy_r(bus); self.x = v; self.set_zn(v); 4 }
            0xAE => { let v = self.abs_r(bus); self.x = v; self.set_zn(v); 4 }
            0xBE => { let (v, p) = self.aby_r(bus); self.x = v; self.set_zn(v); 4 + p }

            // ===== LDY =====
            0xA0 => { let v = self.imm(bus); self.y = v; self.set_zn(v); 2 }
            0xA4 => { let v = self.zp_r(bus); self.y = v; self.set_zn(v); 3 }
            0xB4 => { let v = self.zpx_r(bus); self.y = v; self.set_zn(v); 4 }
            0xAC => { let v = self.abs_r(bus); self.y = v; self.set_zn(v); 4 }
            0xBC => { let (v, p) = self.abx_r(bus); self.y = v; self.set_zn(v); 4 + p }

            // ===== STA =====
            0x85 => { let a = self.zp_a(bus); bus.cpu_write(a, self.a); 3 }
            0x95 => { let a = self.zpx_a(bus); bus.cpu_write(a, self.a); 4 }
            0x8D => { let a = self.abs_a(bus); bus.cpu_write(a, self.a); 4 }
            0x9D => { let a = self.abx_a(bus); bus.cpu_write(a, self.a); 5 }
            0x99 => { let a = self.aby_a(bus); bus.cpu_write(a, self.a); 5 }
            0x81 => { let a = self.izx_a(bus); bus.cpu_write(a, self.a); 6 }
            0x91 => { let a = self.izy_a(bus); bus.cpu_write(a, self.a); 6 }

            // ===== STX =====
            0x86 => { let a = self.zp_a(bus); bus.cpu_write(a, self.x); 3 }
            0x96 => { let a = self.zpy_a(bus); bus.cpu_write(a, self.x); 4 }
            0x8E => { let a = self.abs_a(bus); bus.cpu_write(a, self.x); 4 }

            // ===== STY =====
            0x84 => { let a = self.zp_a(bus); bus.cpu_write(a, self.y); 3 }
            0x94 => { let a = self.zpx_a(bus); bus.cpu_write(a, self.y); 4 }
            0x8C => { let a = self.abs_a(bus); bus.cpu_write(a, self.y); 4 }

            // ===== ADC =====
            0x69 => { let v = self.imm(bus); self.adc(v); 2 }
            0x65 => { let v = self.zp_r(bus); self.adc(v); 3 }
            0x75 => { let v = self.zpx_r(bus); self.adc(v); 4 }
            0x6D => { let v = self.abs_r(bus); self.adc(v); 4 }
            0x7D => { let (v, p) = self.abx_r(bus); self.adc(v); 4 + p }
            0x79 => { let (v, p) = self.aby_r(bus); self.adc(v); 4 + p }
            0x61 => { let v = self.izx_r(bus); self.adc(v); 6 }
            0x71 => { let (v, p) = self.izy_r(bus); self.adc(v); 5 + p }

            // ===== SBC =====
            0xE9 | 0xEB => { let v = self.imm(bus); self.sbc(v); 2 }
            0xE5 => { let v = self.zp_r(bus); self.sbc(v); 3 }
            0xF5 => { let v = self.zpx_r(bus); self.sbc(v); 4 }
            0xED => { let v = self.abs_r(bus); self.sbc(v); 4 }
            0xFD => { let (v, p) = self.abx_r(bus); self.sbc(v); 4 + p }
            0xF9 => { let (v, p) = self.aby_r(bus); self.sbc(v); 4 + p }
            0xE1 => { let v = self.izx_r(bus); self.sbc(v); 6 }
            0xF1 => { let (v, p) = self.izy_r(bus); self.sbc(v); 5 + p }

            // ===== AND =====
            0x29 => { let v = self.imm(bus); self.a &= v; self.set_zn(self.a); 2 }
            0x25 => { let v = self.zp_r(bus); self.a &= v; self.set_zn(self.a); 3 }
            0x35 => { let v = self.zpx_r(bus); self.a &= v; self.set_zn(self.a); 4 }
            0x2D => { let v = self.abs_r(bus); self.a &= v; self.set_zn(self.a); 4 }
            0x3D => { let (v, p) = self.abx_r(bus); self.a &= v; self.set_zn(self.a); 4 + p }
            0x39 => { let (v, p) = self.aby_r(bus); self.a &= v; self.set_zn(self.a); 4 + p }
            0x21 => { let v = self.izx_r(bus); self.a &= v; self.set_zn(self.a); 6 }
            0x31 => { let (v, p) = self.izy_r(bus); self.a &= v; self.set_zn(self.a); 5 + p }

            // ===== ORA =====
            0x09 => { let v = self.imm(bus); self.a |= v; self.set_zn(self.a); 2 }
            0x05 => { let v = self.zp_r(bus); self.a |= v; self.set_zn(self.a); 3 }
            0x15 => { let v = self.zpx_r(bus); self.a |= v; self.set_zn(self.a); 4 }
            0x0D => { let v = self.abs_r(bus); self.a |= v; self.set_zn(self.a); 4 }
            0x1D => { let (v, p) = self.abx_r(bus); self.a |= v; self.set_zn(self.a); 4 + p }
            0x19 => { let (v, p) = self.aby_r(bus); self.a |= v; self.set_zn(self.a); 4 + p }
            0x01 => { let v = self.izx_r(bus); self.a |= v; self.set_zn(self.a); 6 }
            0x11 => { let (v, p) = self.izy_r(bus); self.a |= v; self.set_zn(self.a); 5 + p }

            // ===== EOR =====
            0x49 => { let v = self.imm(bus); self.a ^= v; self.set_zn(self.a); 2 }
            0x45 => { let v = self.zp_r(bus); self.a ^= v; self.set_zn(self.a); 3 }
            0x55 => { let v = self.zpx_r(bus); self.a ^= v; self.set_zn(self.a); 4 }
            0x4D => { let v = self.abs_r(bus); self.a ^= v; self.set_zn(self.a); 4 }
            0x5D => { let (v, p) = self.abx_r(bus); self.a ^= v; self.set_zn(self.a); 4 + p }
            0x59 => { let (v, p) = self.aby_r(bus); self.a ^= v; self.set_zn(self.a); 4 + p }
            0x41 => { let v = self.izx_r(bus); self.a ^= v; self.set_zn(self.a); 6 }
            0x51 => { let (v, p) = self.izy_r(bus); self.a ^= v; self.set_zn(self.a); 5 + p }

            // ===== CMP =====
            0xC9 => { let v = self.imm(bus); self.cmp_reg(self.a, v); 2 }
            0xC5 => { let v = self.zp_r(bus); self.cmp_reg(self.a, v); 3 }
            0xD5 => { let v = self.zpx_r(bus); self.cmp_reg(self.a, v); 4 }
            0xCD => { let v = self.abs_r(bus); self.cmp_reg(self.a, v); 4 }
            0xDD => { let (v, p) = self.abx_r(bus); self.cmp_reg(self.a, v); 4 + p }
            0xD9 => { let (v, p) = self.aby_r(bus); self.cmp_reg(self.a, v); 4 + p }
            0xC1 => { let v = self.izx_r(bus); self.cmp_reg(self.a, v); 6 }
            0xD1 => { let (v, p) = self.izy_r(bus); self.cmp_reg(self.a, v); 5 + p }

            // ===== CPX =====
            0xE0 => { let v = self.imm(bus); self.cmp_reg(self.x, v); 2 }
            0xE4 => { let v = self.zp_r(bus); self.cmp_reg(self.x, v); 3 }
            0xEC => { let v = self.abs_r(bus); self.cmp_reg(self.x, v); 4 }

            // ===== CPY =====
            0xC0 => { let v = self.imm(bus); self.cmp_reg(self.y, v); 2 }
            0xC4 => { let v = self.zp_r(bus); self.cmp_reg(self.y, v); 3 }
            0xCC => { let v = self.abs_r(bus); self.cmp_reg(self.y, v); 4 }

            // ===== BIT =====
            0x24 => { let v = self.zp_r(bus); self.set_flag(FLAG_Z, self.a & v == 0); self.set_flag(FLAG_V, v & 0x40 != 0); self.set_flag(FLAG_N, v & 0x80 != 0); 3 }
            0x2C => { let v = self.abs_r(bus); self.set_flag(FLAG_Z, self.a & v == 0); self.set_flag(FLAG_V, v & 0x40 != 0); self.set_flag(FLAG_N, v & 0x80 != 0); 4 }

            // ===== ASL =====
            0x0A => { self.a = self.asl_val(self.a); 2 }
            0x06 => { let a = self.zp_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); 5 }
            0x16 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); 6 }
            0x0E => { let a = self.abs_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); 6 }
            0x1E => { let a = self.abx_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); 7 }

            // ===== LSR =====
            0x4A => { self.a = self.lsr_val(self.a); 2 }
            0x46 => { let a = self.zp_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); 5 }
            0x56 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); 6 }
            0x4E => { let a = self.abs_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); 6 }
            0x5E => { let a = self.abx_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); 7 }

            // ===== ROL =====
            0x2A => { self.a = self.rol_val(self.a); 2 }
            0x26 => { let a = self.zp_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); 5 }
            0x36 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); 6 }
            0x2E => { let a = self.abs_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); 6 }
            0x3E => { let a = self.abx_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); 7 }

            // ===== ROR =====
            0x6A => { self.a = self.ror_val(self.a); 2 }
            0x66 => { let a = self.zp_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); 5 }
            0x76 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); 6 }
            0x6E => { let a = self.abs_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); 6 }
            0x7E => { let a = self.abx_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); 7 }

            // ===== INC =====
            0xE6 => { let a = self.zp_a(bus); let v = bus.cpu_read(a).wrapping_add(1); self.set_zn(v); bus.cpu_write(a, v); 5 }
            0xF6 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a).wrapping_add(1); self.set_zn(v); bus.cpu_write(a, v); 6 }
            0xEE => { let a = self.abs_a(bus); let v = bus.cpu_read(a).wrapping_add(1); self.set_zn(v); bus.cpu_write(a, v); 6 }
            0xFE => { let a = self.abx_a(bus); let v = bus.cpu_read(a).wrapping_add(1); self.set_zn(v); bus.cpu_write(a, v); 7 }

            // ===== DEC =====
            0xC6 => { let a = self.zp_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); self.set_zn(v); bus.cpu_write(a, v); 5 }
            0xD6 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); self.set_zn(v); bus.cpu_write(a, v); 6 }
            0xCE => { let a = self.abs_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); self.set_zn(v); bus.cpu_write(a, v); 6 }
            0xDE => { let a = self.abx_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); self.set_zn(v); bus.cpu_write(a, v); 7 }

            // ===== INX/INY/DEX/DEY =====
            0xE8 => { self.x = self.x.wrapping_add(1); self.set_zn(self.x); 2 }
            0xC8 => { self.y = self.y.wrapping_add(1); self.set_zn(self.y); 2 }
            0xCA => { self.x = self.x.wrapping_sub(1); self.set_zn(self.x); 2 }
            0x88 => { self.y = self.y.wrapping_sub(1); self.set_zn(self.y); 2 }

            // ===== Transfers =====
            0xAA => { self.x = self.a; self.set_zn(self.x); 2 }  // TAX
            0xA8 => { self.y = self.a; self.set_zn(self.y); 2 }  // TAY
            0x8A => { self.a = self.x; self.set_zn(self.a); 2 }  // TXA
            0x98 => { self.a = self.y; self.set_zn(self.a); 2 }  // TYA
            0xBA => { self.x = self.sp; self.set_zn(self.x); 2 }  // TSX
            0x9A => { self.sp = self.x; 2 }                        // TXS

            // ===== Branches =====
            0x90 => self.branch(bus, !self.flag(FLAG_C)),  // BCC
            0xB0 => self.branch(bus, self.flag(FLAG_C)),   // BCS
            0xF0 => self.branch(bus, self.flag(FLAG_Z)),   // BEQ
            0xD0 => self.branch(bus, !self.flag(FLAG_Z)),  // BNE
            0x30 => self.branch(bus, self.flag(FLAG_N)),   // BMI
            0x10 => self.branch(bus, !self.flag(FLAG_N)),  // BPL
            0x50 => self.branch(bus, !self.flag(FLAG_V)),  // BVC
            0x70 => self.branch(bus, self.flag(FLAG_V)),   // BVS

            // ===== JMP =====
            0x4C => { self.pc = self.read16(bus); 3 }
            0x6C => { let a = self.read16(bus); self.pc = self.read16_bug(bus, a); 5 }

            // ===== JSR/RTS/RTI =====
            0x20 => { let a = self.read16(bus); self.push16(bus, self.pc.wrapping_sub(1)); self.pc = a; 6 }
            0x60 => { self.pc = self.pull16(bus).wrapping_add(1); 6 }
            0x40 => {
                self.status = (self.pull8(bus) & !FLAG_B) | FLAG_U;
                self.pc = self.pull16(bus);
                4
            }

            // ===== Stack =====
            0x48 => { self.push8(bus, self.a); 3 }           // PHA
            0x08 => { self.push8(bus, self.status | FLAG_B | FLAG_U); 3 } // PHP
            0x68 => { self.a = self.pull8(bus); self.set_zn(self.a); 4 }  // PLA
            0x28 => { self.status = (self.pull8(bus) & !FLAG_B) | FLAG_U; 4 } // PLP

            // ===== Flags =====
            0x18 => { self.set_flag(FLAG_C, false); 2 } // CLC
            0x38 => { self.set_flag(FLAG_C, true); 2 }  // SEC
            0x58 => { self.set_flag(FLAG_I, false); 2 } // CLI
            0x78 => { self.set_flag(FLAG_I, true); 2 }  // SEI
            0xD8 => { self.set_flag(FLAG_D, false); 2 } // CLD
            0xF8 => { self.set_flag(FLAG_D, true); 2 }  // SED
            0xB8 => { self.set_flag(FLAG_V, false); 2 } // CLV

            // ===== BRK =====
            0x00 => {
                self.pc = self.pc.wrapping_add(1);
                self.push16(bus, self.pc);
                self.push8(bus, self.status | FLAG_B | FLAG_U);
                self.set_flag(FLAG_I, true);
                let lo = bus.cpu_read(0xFFFE) as u16;
                let hi = bus.cpu_read(0xFFFF) as u16;
                self.pc = (hi << 8) | lo;
                7
            }

            // ===== NOP =====
            0xEA => 2,

            // ===== Unofficial NOPs (various sizes) =====
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => 2,
            0x04 | 0x44 | 0x64 => { self.pc = self.pc.wrapping_add(1); 3 }
            0x0C => { self.pc = self.pc.wrapping_add(2); 4 }
            0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => { self.pc = self.pc.wrapping_add(1); 4 }
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                let (_, p) = self.abx_r(bus); 4 + p
            }
            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => { self.pc = self.pc.wrapping_add(1); 2 }

            // ===== Unofficial: LAX (LDA + LDX) =====
            0xA7 => { let v = self.zp_r(bus); self.a = v; self.x = v; self.set_zn(v); 3 }
            0xB7 => { let v = self.zpy_r(bus); self.a = v; self.x = v; self.set_zn(v); 4 }
            0xAF => { let v = self.abs_r(bus); self.a = v; self.x = v; self.set_zn(v); 4 }
            0xBF => { let (v, p) = self.aby_r(bus); self.a = v; self.x = v; self.set_zn(v); 4 + p }
            0xA3 => { let v = self.izx_r(bus); self.a = v; self.x = v; self.set_zn(v); 6 }
            0xB3 => { let (v, p) = self.izy_r(bus); self.a = v; self.x = v; self.set_zn(v); 5 + p }

            // ===== Unofficial: SAX (store A & X) =====
            0x87 => { let a = self.zp_a(bus); bus.cpu_write(a, self.a & self.x); 3 }
            0x97 => { let a = self.zpy_a(bus); bus.cpu_write(a, self.a & self.x); 4 }
            0x8F => { let a = self.abs_a(bus); bus.cpu_write(a, self.a & self.x); 4 }
            0x83 => { let a = self.izx_a(bus); bus.cpu_write(a, self.a & self.x); 6 }

            // ===== Unofficial: DCP (DEC + CMP) =====
            0xC7 => { let a = self.zp_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); bus.cpu_write(a, v); self.cmp_reg(self.a, v); 5 }
            0xD7 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); bus.cpu_write(a, v); self.cmp_reg(self.a, v); 6 }
            0xCF => { let a = self.abs_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); bus.cpu_write(a, v); self.cmp_reg(self.a, v); 6 }
            0xDF => { let a = self.abx_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); bus.cpu_write(a, v); self.cmp_reg(self.a, v); 7 }
            0xDB => { let a = self.aby_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); bus.cpu_write(a, v); self.cmp_reg(self.a, v); 7 }
            0xC3 => { let a = self.izx_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); bus.cpu_write(a, v); self.cmp_reg(self.a, v); 8 }
            0xD3 => { let a = self.izy_a(bus); let v = bus.cpu_read(a).wrapping_sub(1); bus.cpu_write(a, v); self.cmp_reg(self.a, v); 8 }

            // ===== Unofficial: ISB/ISC (INC + SBC) =====
            0xE7 => { let a = self.zp_a(bus); let v = bus.cpu_read(a).wrapping_add(1); bus.cpu_write(a, v); self.sbc(v); 5 }
            0xF7 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a).wrapping_add(1); bus.cpu_write(a, v); self.sbc(v); 6 }
            0xEF => { let a = self.abs_a(bus); let v = bus.cpu_read(a).wrapping_add(1); bus.cpu_write(a, v); self.sbc(v); 6 }
            0xFF => { let a = self.abx_a(bus); let v = bus.cpu_read(a).wrapping_add(1); bus.cpu_write(a, v); self.sbc(v); 7 }
            0xFB => { let a = self.aby_a(bus); let v = bus.cpu_read(a).wrapping_add(1); bus.cpu_write(a, v); self.sbc(v); 7 }
            0xE3 => { let a = self.izx_a(bus); let v = bus.cpu_read(a).wrapping_add(1); bus.cpu_write(a, v); self.sbc(v); 8 }
            0xF3 => { let a = self.izy_a(bus); let v = bus.cpu_read(a).wrapping_add(1); bus.cpu_write(a, v); self.sbc(v); 8 }

            // ===== Unofficial: SLO (ASL + ORA) =====
            0x07 => { let a = self.zp_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); self.a |= r; self.set_zn(self.a); 5 }
            0x17 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); self.a |= r; self.set_zn(self.a); 6 }
            0x0F => { let a = self.abs_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); self.a |= r; self.set_zn(self.a); 6 }
            0x1F => { let a = self.abx_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); self.a |= r; self.set_zn(self.a); 7 }
            0x1B => { let a = self.aby_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); self.a |= r; self.set_zn(self.a); 7 }
            0x03 => { let a = self.izx_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); self.a |= r; self.set_zn(self.a); 8 }
            0x13 => { let a = self.izy_a(bus); let v = bus.cpu_read(a); let r = self.asl_val(v); bus.cpu_write(a, r); self.a |= r; self.set_zn(self.a); 8 }

            // ===== Unofficial: RLA (ROL + AND) =====
            0x27 => { let a = self.zp_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); self.a &= r; self.set_zn(self.a); 5 }
            0x37 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); self.a &= r; self.set_zn(self.a); 6 }
            0x2F => { let a = self.abs_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); self.a &= r; self.set_zn(self.a); 6 }
            0x3F => { let a = self.abx_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); self.a &= r; self.set_zn(self.a); 7 }
            0x3B => { let a = self.aby_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); self.a &= r; self.set_zn(self.a); 7 }
            0x23 => { let a = self.izx_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); self.a &= r; self.set_zn(self.a); 8 }
            0x33 => { let a = self.izy_a(bus); let v = bus.cpu_read(a); let r = self.rol_val(v); bus.cpu_write(a, r); self.a &= r; self.set_zn(self.a); 8 }

            // ===== Unofficial: SRE (LSR + EOR) =====
            0x47 => { let a = self.zp_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); self.a ^= r; self.set_zn(self.a); 5 }
            0x57 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); self.a ^= r; self.set_zn(self.a); 6 }
            0x4F => { let a = self.abs_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); self.a ^= r; self.set_zn(self.a); 6 }
            0x5F => { let a = self.abx_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); self.a ^= r; self.set_zn(self.a); 7 }
            0x5B => { let a = self.aby_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); self.a ^= r; self.set_zn(self.a); 7 }
            0x43 => { let a = self.izx_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); self.a ^= r; self.set_zn(self.a); 8 }
            0x53 => { let a = self.izy_a(bus); let v = bus.cpu_read(a); let r = self.lsr_val(v); bus.cpu_write(a, r); self.a ^= r; self.set_zn(self.a); 8 }

            // ===== Unofficial: RRA (ROR + ADC) =====
            0x67 => { let a = self.zp_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); self.adc(r); 5 }
            0x77 => { let a = self.zpx_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); self.adc(r); 6 }
            0x6F => { let a = self.abs_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); self.adc(r); 6 }
            0x7F => { let a = self.abx_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); self.adc(r); 7 }
            0x7B => { let a = self.aby_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); self.adc(r); 7 }
            0x63 => { let a = self.izx_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); self.adc(r); 8 }
            0x73 => { let a = self.izy_a(bus); let v = bus.cpu_read(a); let r = self.ror_val(v); bus.cpu_write(a, r); self.adc(r); 8 }

            // ===== Unknown / unimplemented — treat as NOP =====
            _ => {
                // KIL/JAM opcodes and truly unknown: skip based on addressing mode pattern
                1
            }
        };

        self.cycles += cycles as u64;
        cycles
    }
}
