




use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use super::disasm::Bj;




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XrefType {
    
    Call,
    
    Jump,
    
    ConditionalJump,
    
    DataRef,
}


#[derive(Debug, Clone)]
pub struct Er {
    
    pub from: u64,
    
    pub to: u64,
    
    pub xref_type: XrefType,
}




#[derive(Debug, Clone)]
pub struct Hp {
    
    pub entry: u64,
    
    pub end: u64,
    
    pub name: String,
    
    pub instruction_count: usize,
    
    pub calls_to: Vec<u64>,
    
    pub called_from: Vec<u64>,
    
    pub basic_blocks: usize,
}




#[derive(Debug)]
pub struct XrefDatabase {
    
    pub xrefs: Vec<Er>,
    
    pub refs_from: BTreeMap<u64, Vec<Er>>,
    
    pub refs_to: BTreeMap<u64, Vec<Er>>,
    
    pub functions: Vec<Hp>,
    
    pub function_entries: BTreeMap<u64, usize>,
}

impl XrefDatabase {
    
    pub fn ker(
        instructions: &[Bj],
        addr_to_symbol: &BTreeMap<u64, String>,
    ) -> Self {
        let mut xrefs = Vec::new();
        let mut refs_from: BTreeMap<u64, Vec<Er>> = BTreeMap::new();
        let mut refs_to: BTreeMap<u64, Vec<Er>> = BTreeMap::new();

        
        for inst in instructions {
            if let Some(target) = inst.branch_target {
                let pvx = if inst.is_call {
                    XrefType::Call
                } else if inst.is_cond_jump {
                    XrefType::ConditionalJump
                } else if inst.is_jump {
                    XrefType::Jump
                } else {
                    XrefType::DataRef
                };

                let aks = Er {
                    from: inst.address,
                    to: target,
                    xref_type: pvx,
                };

                xrefs.push(aks.clone());
                refs_from.entry(inst.address).or_insert_with(Vec::new).push(aks.clone());
                refs_to.entry(target).or_insert_with(Vec::new).push(aks);
            }

            
            if inst.mnemonic == "lea" && inst.operands_str.contains("rip") {
                
                if let Some(addr) = lts(&inst.operands_str, inst.address, inst.bytes.len() as u64) {
                    let aks = Er {
                        from: inst.address,
                        to: addr,
                        xref_type: XrefType::DataRef,
                    };
                    xrefs.push(aks.clone());
                    refs_from.entry(inst.address).or_insert_with(Vec::new).push(aks.clone());
                    refs_to.entry(addr).or_insert_with(Vec::new).push(aks);
                }
            }
        }

        
        let functions = ldw(instructions, &refs_to, addr_to_symbol);

        
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

    
    pub fn xrefs_to(&self, addr: u64) -> &[Er] {
        self.refs_to.get(&addr).map(|v| v.as_slice()).unwrap_or(&[])
    }

    
    pub fn xrefs_from(&self, addr: u64) -> &[Er] {
        self.refs_from.get(&addr).map(|v| v.as_slice()).unwrap_or(&[])
    }

    
    pub fn function_at(&self, addr: u64) -> Option<&Hp> {
        
        for func in self.functions.iter().rev() {
            if addr >= func.entry && addr < func.end {
                return Some(func);
            }
        }
        None
    }

    
    pub fn is_function_entry(&self, addr: u64) -> bool {
        self.function_entries.contains_key(&addr)
    }

    
    pub fn qhw(&self, entry: u64) -> Option<&Hp> {
        self.function_entries.get(&entry)
            .and_then(|&idx| self.functions.get(idx))
    }

    
    pub fn rat(&self) -> usize {
        self.xrefs.len()
    }

    
    pub fn pzd(&self) -> usize {
        self.xrefs.iter().filter(|x| x.xref_type == XrefType::Call).count()
    }

    
    pub fn summary(&self) -> String {
        let fkp = self.xrefs.iter().filter(|x| x.xref_type == XrefType::Call).count();
        let mvi = self.xrefs.iter().filter(|x| x.xref_type == XrefType::Jump || x.xref_type == XrefType::ConditionalJump).count();
        let data = self.xrefs.iter().filter(|x| x.xref_type == XrefType::DataRef).count();

        format!(
            "Xrefs: {} total ({} calls, {} jumps, {} data) | {} functions detected",
            self.xrefs.len(), fkp, mvi, data, self.functions.len()
        )
    }
}



fn ldw(
    instructions: &[Bj],
    refs_to: &BTreeMap<u64, Vec<Er>>,
    addr_to_symbol: &BTreeMap<u64, String>,
) -> Vec<Hp> {
    if instructions.is_empty() {
        return Vec::new();
    }

    
    let mut entries: BTreeMap<u64, String> = BTreeMap::new();

    
    for (addr, xrefs) in refs_to.iter() {
        for aks in xrefs {
            if aks.xref_type == XrefType::Call {
                let name = addr_to_symbol.get(addr)
                    .cloned()
                    .unwrap_or_else(|| format!("sub_{:x}", addr));
                entries.insert(*addr, name);
                break;
            }
        }
    }

    
    for (addr, name) in addr_to_symbol {
        entries.entry(*addr).or_insert_with(|| name.clone());
    }

    
    let hzb = instructions[0].address;
    entries.entry(hzb).or_insert_with(|| {
        addr_to_symbol.get(&hzb)
            .cloned()
            .unwrap_or_else(|| String::from("_start"))
    });

    
    let gvp: Vec<(u64, String)> = entries.into_iter().collect();

    
    let hea: BTreeMap<u64, usize> = instructions.iter()
        .enumerate()
        .map(|(i, inst)| (inst.address, i))
        .collect();

    
    let mut functions = Vec::new();
    for (i, (entry, name)) in gvp.iter().enumerate() {
        
        let bjj = match hea.get(entry) {
            Some(&idx) => idx,
            None => continue, 
        };

        
        let fus = if i + 1 < gvp.len() {
            let nka = gvp[i + 1].0;
            hea.get(&nka).copied().unwrap_or(instructions.len())
        } else {
            instructions.len()
        };

        if bjj >= fus { continue; }

        let eno = &instructions[bjj..fus];
        let lqa = eno.last()
            .map(|inst| inst.address + inst.bytes.len() as u64)
            .unwrap_or(*entry);

        
        let calls_to: Vec<u64> = eno.iter()
            .filter(|inst| inst.is_call)
            .filter_map(|inst| inst.branch_target)
            .collect();

        
        let called_from: Vec<u64> = refs_to.get(entry)
            .map(|xrefs| xrefs.iter()
                .filter(|x| x.xref_type == XrefType::Call)
                .map(|x| x.from)
                .collect())
            .unwrap_or_default();

        
        let basic_blocks = 1 + eno.iter()
            .filter(|inst| inst.is_jump || inst.is_cond_jump || inst.is_ret)
            .count();

        functions.push(Hp {
            entry: *entry,
            end: lqa,
            name: name.clone(),
            instruction_count: eno.len(),
            calls_to,
            called_from,
            basic_blocks,
        });
    }

    functions.sort_by_key(|f| f.entry);
    functions
}




fn lts(operands: &str, bhf: u64, inst_len: u64) -> Option<u64> {
    
    let grr = operands.find("rip")?;
    let hej = &operands[grr + 3..];

    let dve = bhf + inst_len; 

    if let Some(ef) = hej.strip_prefix('+') {
        
        let end = ef.find(']').unwrap_or(ef.len());
        let ga = ef[..end].trim();
        let offset = if ga.starts_with("0x") {
            u64::from_str_radix(&ga[2..], 16).ok()?
        } else {
            ga.parse::<u64>().ok()?
        };
        Some(dve + offset)
    } else if let Some(ef) = hej.strip_prefix('-') {
        
        let end = ef.find(']').unwrap_or(ef.len());
        let ga = ef[..end].trim();
        let offset = if ga.starts_with("0x") {
            u64::from_str_radix(&ga[2..], 16).ok()?
        } else {
            ga.parse::<u64>().ok()?
        };
        Some(dve.wrapping_sub(offset))
    } else {
        
        Some(dve)
    }
}
