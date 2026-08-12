use watermelon::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TranspositionBound {
    #[allow(
        unused,
        reason = "this is implicitly constructed by `TranspositionEntry::packed`"
    )]
    None,
    Exact,
    Lower,
    Upper,
}

#[derive(Clone)]
pub struct TranspositionEntry {
    /// The upper 16 bits of the (Zobrist) hash of this node.
    pub hash16: u16,
    /// Remaining depth searched from this node.
    pub depth: u8,
    /// Evaluation score of this node.
    pub score: i16,
    /// Packs two different things in 8 bits:
    ///  - **{0..2}** Alpha-beta pruning modifier outlined in [`TranspositionBound`],
    ///               where [`TranspositionBound::None`] means an empty slot.
    ///  - **{2..8}** A generation counter different between consecutive moves.
    pub packed: u8,
    /// Best move found.
    pub best_move: Move,
}

impl TranspositionEntry {
    const EMPTY: Self = TranspositionEntry {
        hash16: 0,
        depth: 0,
        score: 0,
        packed: 0,
        best_move: Move::NULL,
    };

    pub fn bound(&self) -> TranspositionBound {
        // SAFETY: The bitwise operation guarantees that the input is 0 to 3, which are
        // valid values of this enumeration.
        unsafe { std::mem::transmute(self.packed & 0x03) }
    }
}

/// Holds a hashtable of transpositions that can be queried quickly.
pub struct TranspositionTable {
    entries: Vec<TranspositionEntry>,
    mask: usize,
    generation: u8,
}

impl TranspositionTable {
    pub fn with_megabytes(megabytes: usize) -> Self {
        let max_capacity = (megabytes * 1048576) / std::mem::size_of::<TranspositionEntry>();
        let capacity = 1 << max_capacity.max(1).ilog2();

        let entries = vec![TranspositionEntry::EMPTY; capacity];
        Self {
            entries,
            mask: capacity - 1,
            generation: 0,
        }
    }

    pub fn advance_generation(&mut self) {
        self.generation = self.generation.wrapping_add(4);
    }

    pub fn reset(&mut self) {
        self.generation = 0;
        self.entries.fill(TranspositionEntry::EMPTY);
    }

    pub fn store(
        &mut self,
        hash: u64,
        depth: u8,
        score: i16,
        bound: TranspositionBound,
        best_move: Move,
    ) {
        let generation = self.generation;
        let hash16 = (hash >> 48) as u16;
        let slot = self.slot_mut(hash);

        // Obtain the existing slot and determine if this should be replaced with the given entry.
        if bound == TranspositionBound::Exact
            || hash16 != slot.hash16
            || depth >= slot.depth
            || (slot.packed & 0xFC) != generation
        {
            slot.hash16 = hash16;
            slot.depth = depth;
            slot.score = score;
            slot.packed = bound as u8 | generation;
            slot.best_move = best_move;
        }
    }

    pub fn probe(&mut self, hash: u64) -> Option<&TranspositionEntry> {
        let generation = self.generation;
        let hash16 = (hash >> 48) as u16;
        let slot = self.slot_mut(hash);

        if slot.hash16 == hash16 {
            slot.packed = generation | (slot.packed & 0x03);
            Some(slot)
        } else {
            None
        }
    }

    fn slot_mut(&mut self, hash: u64) -> &mut TranspositionEntry {
        let index = (hash as usize) & self.mask;
        // SAFETY: The mask is one less than the length, which is a nonzero power of two by
        // construction. Therefore, the bitwise-and has the same result as modulo length.
        unsafe { self.entries.get_unchecked_mut(index) }
    }
}
