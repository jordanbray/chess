#[derive(Copy, Clone, PartialEq, PartialOrd)]
struct CacheTableEntry<T: Copy + Clone + PartialEq + PartialOrd> {
    hash: u64,
    entry: T,
}

/// Store a cache of entries, each with an associated hash.
pub struct CacheTable<T: Copy + Clone + PartialEq + PartialOrd> {
    table: Box<[CacheTableEntry<T>]>,
    mask: usize,
}

impl<T: Copy + Clone + PartialEq + PartialOrd> CacheTable<T> {
    /// Create a new `CacheTable` with each associated entry initialized with a hash of '0'
    /// Note: You must pass in a size where only 1 bit is set. (AKA: 2, 4, 8, 16, 1024, 65536,
    /// etc.)
    /// Panics when size is invalid.
    #[inline]
    pub fn new(size: usize, default: T) -> CacheTable<T> {
        if size.count_ones() != 1 {
            panic!("You cannot create a CacheTable with a non-binary number.");
        }
        let values = vec![
            CacheTableEntry {
                hash: 0,
                entry: default
            };
            size
        ];
        CacheTable {
            table: values.into_boxed_slice(),
            mask: size - 1,
        }
    }

    /// Get a particular entry with the hash specified
    #[inline]
    pub fn get(&self, hash: u64) -> Option<T> {
        let entry = unsafe { *self.table.get_unchecked((hash as usize) & self.mask) };
        if entry.hash == hash {
            Some(entry.entry)
        } else {
            None
        }
    }

    /// Add (or overwrite) an entry with the associated hash
    #[inline]
    pub fn add(&mut self, hash: u64, entry: T) {
        let e = unsafe { self.table.get_unchecked_mut((hash as usize) & self.mask) };
        *e = CacheTableEntry { hash, entry };
    }

    /// Replace an entry in the hash table with a user-specified replacement policy specified by
    /// `replace`. The `replace` closure is called with the previous entry occupying the hash table
    /// slot, and returns true or false to specify whether the entry should be replaced. Note that
    /// the previous entry may not have the same hash, but merely be the default initialization or
    /// a hash collision with `hash`.
    ///
    /// ```
    /// use chess::CacheTable;
    ///
    /// # fn main() {
    ///
    /// let mut table: CacheTable<char> = CacheTable::new(256, 'a');
    ///
    /// assert_eq!(table.get(5), None);
    /// // Note that 'a' is the default initialization value.
    /// table.replace_if(5, 'b', |old_entry| old_entry != 'a');
    /// assert_eq!(table.get(5), None);
    /// table.replace_if(5, 'c', |old_entry| old_entry == 'a');
    /// assert_eq!(table.get(5), Some('c'));
    /// table.replace_if(5, 'd', |old_entry| old_entry == 'c');
    /// assert_eq!(table.get(5), Some('d'));
    ///
    /// # }
    /// ```
    #[inline(always)]
    pub fn replace_if<F: Fn(T) -> bool>(&mut self, hash: u64, entry: T, replace: F) {
        let e = unsafe { self.table.get_unchecked_mut((hash as usize) & self.mask) };
        if replace(e.entry) {
            *e = CacheTableEntry { hash, entry };
        }
    }
}
