




use alloc::collections::BTreeMap;
use spin::Mutex;

const BOM_: usize = 256; 
const H_: usize = 512;

struct Xe {
    data: [u8; H_],
    dirty: bool,
    access: u64,     
}

pub struct BlockCache {
    entries: BTreeMap<u64, Xe>,
    counter: u64,
    read_fn: fn(u64, &mut [u8; H_]) -> Result<(), ()>,
    write_fn: fn(u64, &[u8; H_]) -> Result<(), ()>,
}

impl BlockCache {
    pub fn new(
        read_fn: fn(u64, &mut [u8; H_]) -> Result<(), ()>,
        write_fn: fn(u64, &[u8; H_]) -> Result<(), ()>,
    ) -> Self {
        Self {
            entries: BTreeMap::new(),
            counter: 0,
            read_fn,
            write_fn,
        }
    }

    
    pub fn read(&mut self, dj: u64, buf: &mut [u8; H_]) -> Result<(), ()> {
        self.counter += 1;
        if let Some(entry) = self.entries.get_mut(&dj) {
            entry.access = self.counter;
            buf.copy_from_slice(&entry.data);
            return Ok(());
        }
        
        (self.read_fn)(dj, buf)?;
        self.insert(dj, *buf, false);
        Ok(())
    }

    
    pub fn write(&mut self, dj: u64, buf: &[u8; H_]) -> Result<(), ()> {
        self.counter += 1;
        if let Some(entry) = self.entries.get_mut(&dj) {
            entry.data.copy_from_slice(buf);
            entry.dirty = true;
            entry.access = self.counter;
            return Ok(());
        }
        self.insert(dj, *buf, true);
        Ok(())
    }

    
    fn insert(&mut self, dj: u64, data: [u8; H_], dirty: bool) {
        if self.entries.len() >= BOM_ {
            self.evict_lru();
        }
        self.entries.insert(dj, Xe {
            data,
            dirty,
            access: self.counter,
        });
    }

    
    fn evict_lru(&mut self) {
        let nbb = self.entries.iter()
            .min_by_key(|(_, e)| e.access)
            .map(|(&j, _)| j);
        if let Some(dj) = nbb {
            if let Some(entry) = self.entries.remove(&dj) {
                if entry.dirty {
                    let _ = (self.write_fn)(dj, &entry.data);
                }
            }
        }
    }

    
    pub fn sync(&mut self) -> Result<(), ()> {
        for (&dj, entry) in self.entries.iter_mut() {
            if entry.dirty {
                (self.write_fn)(dj, &entry.data)?;
                entry.dirty = false;
            }
        }
        Ok(())
    }

    
    pub fn qlt(&mut self, dj: u64) {
        if let Some(entry) = self.entries.remove(&dj) {
            if entry.dirty {
                let _ = (self.write_fn)(dj, &entry.data);
            }
        }
    }

    
    pub fn cached_count(&self) -> usize {
        self.entries.len()
    }

    
    pub fn dirty_count(&self) -> usize {
        self.entries.values().filter(|e| e.dirty).count()
    }
}


static MU_: Mutex<Option<BlockCache>> = Mutex::new(None);


pub fn init(
    read_fn: fn(u64, &mut [u8; H_]) -> Result<(), ()>,
    write_fn: fn(u64, &[u8; H_]) -> Result<(), ()>,
) {
    *MU_.lock() = Some(BlockCache::new(read_fn, write_fn));
}


pub fn kgv(dj: u64, buf: &mut [u8; H_]) -> Result<(), ()> {
    if let Some(adk) = MU_.lock().as_mut() {
        adk.read(dj, buf)
    } else {
        Err(()) 
    }
}


pub fn kgw(dj: u64, buf: &[u8; H_]) -> Result<(), ()> {
    if let Some(adk) = MU_.lock().as_mut() {
        adk.write(dj, buf)
    } else {
        Err(())
    }
}


pub fn sync() -> Result<(), ()> {
    if let Some(adk) = MU_.lock().as_mut() {
        adk.sync()
    } else {
        Ok(())
    }
}


pub fn stats() -> (usize, usize) {
    if let Some(adk) = MU_.lock().as_ref() {
        (adk.cached_count(), adk.dirty_count())
    } else {
        (0, 0)
    }
}
