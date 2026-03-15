//! TrustView — Cross-Reference (Xref) Analysis
//!
//! Builds a cross-reference graph from disassembled instructions.
//! Tracks CALL targets, JMP targets, data references, and function boundaries.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use super::disasm::Instruction;

// ──── Xref Types ───────────────────────────────────────────────────────────

/// Type of cross-reference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XrefType {
    /// CALL instruction
    Call,
    /// Unconditional JMP
    Jump,
    /// Conditional jump (Jcc)
    ConditionalJump,
    /// Data reference (LEA, MOV with memory)
    DataRef,
}

/// A single cross-reference
#[derive(Debug, Clone)]
pub struct Xref {
    /// Source address (where the reference is made from)
    pub from: u64,
    /// Target address (where it points to)
    pub to: u64,
    /// Type of reference
    pub xref_type: XrefType,
}

// ──── Function Boundary ────────────────────────────────────────────────────

/// Detected function with its instruction range
#[derive(Debug, Clone)]
pub struct DetectedFunction {
    /// Function entry address
    pub entry: u64,
    /// End address (address after last instruction)
    pub end: u64,
    /// Function name (from symbol table or generated)
    pub name: String,
    /// Number of instructions
    pub instruction_count: usize,
    /// Addresses this function calls
    pub calls_to: Vec<u64>,
    /// Addresses that call this function
    pub called_from: Vec<u64>,
    /// Number of basic blocks (approximate)
    pub basic_blocks: usize,
}

// ──── Xref Database ────────────────────────────────────────────────────────

/// Complete cross-reference database for a binary
#[derive(Debug)]
pub struct XrefDatabase {
    /// All cross-references
    pub xrefs: Vec<Xref>,
    /// References FROM a given address → list of targets
    pub refs_from: BTreeMap<u64, Vec<Xref>>,
    /// References TO a given address → list of sources
    pub refs_to: BTreeMap<u64, Vec<Xref>>,
    /// Detected functions
    pub functions: Vec<DetectedFunction>,
    /// Function entry addresses (for quick lookup)
    pub function_entries: BTreeMap<u64, usize>,
}

impl XrefDatabase {
    /// Build xref database from disassembled instructions and symbol table
    pub fn build(
        instructions: &[Instruction],
        addr_to_symbol: &BTreeMap<u64, String>,
    ) -> Self {
        let mut xrefs = Vec::new();
        let mut refs_from: BTreeMap<u64, Vec<Xref>> = BTreeMap::new();
        let mut refs_to: BTreeMap<u64, Vec<Xref>> = BTreeMap::new();

        // ── Pass 1: Collect all xrefs from instructions ──
        for inst in instructions {
            if let Some(target) = inst.branch_target {
                let xtype = if inst.is_call {
                    XrefType::Call
                } else if inst.is_cond_jump {
                    XrefType::ConditionalJump
                } else if inst.is_jump {
                    XrefType::Jump
                } else {
                    XrefType::DataRef
                };

                let xref = Xref {
                    from: inst.address,
                    to: target,
                    xref_type: xtype,
                };

                xrefs.push(xref.clone());
                refs_from.entry(inst.address).or_insert_with(Vec::new).push(xref.clone());
                refs_to.entry(target).or_insert_with(Vec::new).push(xref);
            }

            // Check for LEA with RIP-relative addressing (data reference)
            if inst.mnemonic == "lea" && inst.operands_str.contains("rip") {
                // Try to extract the effective address from the operand string
                if let Some(addr) = extract_rip_target(&inst.operands_str, inst.address, inst.bytes.len() as u64) {
                    let xref = Xref {
                        from: inst.address,
                        to: addr,
                        xref_type: XrefType::DataRef,
                    };
                    xrefs.push(xref.clone());
                    refs_from.entry(inst.address).or_insert_with(Vec::new).push(xref.clone());
                    refs_to.entry(addr).or_insert_with(Vec::new).push(xref);
                }
            }
        }

        // ── Pass 2: Detect function boundaries ──
        let functions = detect_functions(instructions, &refs_to, addr_to_symbol);

        // Build function entry lookup
        let mut function_entries = BTreeMap::new();
        for (i, f) in functions.iter().enumerate() {
            function_entries.insert(f.entry, i);
        }

        Self {
            xrefs,
            refs_from,
            refs_to,
            functions,
            function_entries,
        }
    }

    /// Get all references to this address
    pub fn xrefs_to(&self, addr: u64) -> &[Xref] {
        self.refs_to.get(&addr).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get all references from this address
    pub fn xrefs_from(&self, addr: u64) -> &[Xref] {
        self.refs_from.get(&addr).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Find which function contains this address
    pub fn function_at(&self, addr: u64) -> Option<&DetectedFunction> {
        // Find the function with the largest entry ≤ addr
        for func in self.functions.iter().rev() {
            if addr >= func.entry && addr < func.end {
                return Some(func);
            }
        }
        None
    }

    /// Is this address a function entry?
    pub fn is_function_entry(&self, addr: u64) -> bool {
        self.function_entries.contains_key(&addr)
    }

    /// Get function by entry address
    pub fn get_function(&self, entry: u64) -> Option<&DetectedFunction> {
        self.function_entries.get(&entry)
            .and_then(|&idx| self.functions.get(idx))
    }

    /// Count total xrefs
    pub fn total_xrefs(&self) -> usize {
        self.xrefs.len()
    }

    /// Count call xrefs only
    pub fn call_count(&self) -> usize {
        self.xrefs.iter().filter(|x| x.xref_type == XrefType::Call).count()
    }

    /// Summary string
    pub fn summary(&self) -> String {
        let calls = self.xrefs.iter().filter(|x| x.xref_type == XrefType::Call).count();
        let jumps = self.xrefs.iter().filter(|x| x.xref_type == XrefType::Jump || x.xref_type == XrefType::ConditionalJump).count();
        let data = self.xrefs.iter().filter(|x| x.xref_type == XrefType::DataRef).count();

        format!(
            "Xrefs: {} total ({} calls, {} jumps, {} data) | {} functions detected",
            self.xrefs.len(), calls, jumps, data, self.functions.len()
        )
    }
}

// ──── Function Detection ───────────────────────────────────────────────────

fn detect_functions(
    instructions: &[Instruction],
    refs_to: &BTreeMap<u64, Vec<Xref>>,
    addr_to_symbol: &BTreeMap<u64, String>,
) -> Vec<DetectedFunction> {
    if instructions.is_empty() {
        return Vec::new();
    }

    // Collect function entry points
    let mut entries: BTreeMap<u64, String> = BTreeMap::new();

    // 1. All CALL targets are function entries
    for (addr, xrefs) in refs_to.iter() {
        for xref in xrefs {
            if xref.xref_type == XrefType::Call {
                let name = addr_to_symbol.get(addr)
                    .cloned()
                    .unwrap_or_else(|| format!("sub_{:x}", addr));
                entries.insert(*addr, name);
                break;
            }
        }
    }

    // 2. All symbols of type FUNC are entries
    for (addr, name) in addr_to_symbol {
        entries.entry(*addr).or_insert_with(|| name.clone());
    }

    // 3. First instruction is an entry (if no symbol covers it)
    let first_addr = instructions[0].address;
    entries.entry(first_addr).or_insert_with(|| {
        addr_to_symbol.get(&first_addr)
            .cloned()
            .unwrap_or_else(|| String::from("_start"))
    });

    // Sort entries by address
    let sorted_entries: Vec<(u64, String)> = entries.into_iter().collect();

    // Build instruction address map for quick lookup
    let addr_to_idx: BTreeMap<u64, usize> = instructions.iter()
        .enumerate()
        .map(|(i, inst)| (inst.address, i))
        .collect();

    // Build functions
    let mut functions = Vec::new();
    for (i, (entry, name)) in sorted_entries.iter().enumerate() {
        // Find instruction index for this entry
        let start_idx = match addr_to_idx.get(entry) {
            Some(&idx) => idx,
            None => continue, // Entry address not in our instructions
        };

        // End is either the next function entry or end of instructions
        let end_idx = if i + 1 < sorted_entries.len() {
            let next_entry = sorted_entries[i + 1].0;
            addr_to_idx.get(&next_entry).copied().unwrap_or(instructions.len())
        } else {
            instructions.len()
        };

        if start_idx >= end_idx { continue; }

        let func_instructions = &instructions[start_idx..end_idx];
        let end_addr = func_instructions.last()
            .map(|inst| inst.address + inst.bytes.len() as u64)
            .unwrap_or(*entry);

        // Count calls from this function
        let calls_to: Vec<u64> = func_instructions.iter()
            .filter(|inst| inst.is_call)
            .filter_map(|inst| inst.branch_target)
            .collect();

        // Count callers (from refs_to)
        let called_from: Vec<u64> = refs_to.get(entry)
            .map(|xrefs| xrefs.iter()
                .filter(|x| x.xref_type == XrefType::Call)
                .map(|x| x.from)
                .collect())
            .unwrap_or_default();

        // Count basic blocks (rough: each Jcc/JMP/RET starts a new one)
        let basic_blocks = 1 + func_instructions.iter()
            .filter(|inst| inst.is_jump || inst.is_cond_jump || inst.is_ret)
            .count();

        functions.push(DetectedFunction {
            entry: *entry,
            end: end_addr,
            name: name.clone(),
            instruction_count: func_instructions.len(),
            calls_to,
            called_from,
            basic_blocks,
        });
    }

    functions.sort_by_key(|f| f.entry);
    functions
}

// ──── Helpers ──────────────────────────────────────────────────────────────

/// Try to extract an absolute address from a RIP-relative operand
fn extract_rip_target(operands: &str, inst_addr: u64, inst_len: u64) -> Option<u64> {
    // Format: "reg, qword [rip+0x123]" or "reg, qword [rip-0x123]"
    let rip_pos = operands.find("rip")?;
    let after_rip = &operands[rip_pos + 3..];

    let next_addr = inst_addr + inst_len; // RIP points to next instruction

    if let Some(rest) = after_rip.strip_prefix('+') {
        // Positive offset
        let end = rest.find(']').unwrap_or(rest.len());
        let hex = rest[..end].trim();
        let offset = if hex.starts_with("0x") {
            u64::from_str_radix(&hex[2..], 16).ok()?
        } else {
            hex.parse::<u64>().ok()?
        };
        Some(next_addr + offset)
    } else if let Some(rest) = after_rip.strip_prefix('-') {
        // Negative offset
        let end = rest.find(']').unwrap_or(rest.len());
        let hex = rest[..end].trim();
        let offset = if hex.starts_with("0x") {
            u64::from_str_radix(&hex[2..], 16).ok()?
        } else {
            hex.parse::<u64>().ok()?
        };
        Some(next_addr.wrapping_sub(offset))
    } else {
        // [rip] with no offset
        Some(next_addr)
    }
}
