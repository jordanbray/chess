#[derive(Copy, Clone, PartialEq, PartialOrd)]
struct CacheTableEntry<T: Copy + Clone + PartialEq + PartialOrd > {
    hash: u64,
    entry: T
}

pub struct CacheTable<T: Copy + Clone + PartialEq + PartialOrd> {
    table: Vec<CacheTableEntry<T>>,
    mask: usize
}

impl <T: Copy + Clone + PartialEq + PartialOrd> CacheTable<T> {
    pub fn new(size: usize, default: T) -> CacheTable<T> {
        if size.count_ones() != 1 {
            panic!("You cannot create a CacheTable with a non-binary number.");
        }
        let mut res = CacheTable::<T> { table: Vec::with_capacity(size),
                                        mask: size - 1 };
        for _ in 0..65536 {
            res.table.push(CacheTableEntry { hash: 0, entry: default });
        }
        res
    }

    pub fn get(&self, hash: u64) -> Option<T> {
        let entry = unsafe { *self.table.get_unchecked((hash as usize) & self.mask) };
        if entry.hash == hash {
            Some(entry.entry)
        } else {
            None
        }
    }

    pub fn add(&mut self, hash: u64, entry: T) {
        let mut e = unsafe { self.table.get_unchecked_mut((hash as usize) & self.mask) };
        *e = CacheTableEntry { hash: hash, entry: entry };
    }
}
