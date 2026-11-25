//! Consume shader write marks without treating the overflow sentinel as a page.

/// Takes marks from the allocated part of `bits`, in order, clearing consumed
/// words. Only the bitmap's final bit is reserved for out-of-range writes;
/// the other 31 bits of its final word can belong to live copies.
pub(crate) fn take_marked(bits: &mut [u32], used_bits: usize) -> Vec<u32> {
    let Some(last) = bits.last_mut() else {
        return Vec::new();
    };
    *last &= !(1 << 31);
    let used = used_bits.div_ceil(32).min(bits.len());
    let mut marked = Vec::new();
    for (w, word) in bits[..used].iter_mut().enumerate() {
        let mut x = std::mem::take(word);
        while x != 0 {
            marked.push((w * 32) as u32 + x.trailing_zeros());
            x &= x - 1;
        }
    }
    marked
}

#[cfg(test)]
mod tests {
    use super::take_marked;

    #[test]
    fn the_last_word_keeps_all_31_allocatable_page_marks() {
        let mut bits = [1 << 31, u32::MAX];
        assert_eq!(take_marked(&mut bits, 63), (31..63).collect::<Vec<_>>());
        assert_eq!(bits, [0, 0]);
    }

    #[test]
    fn the_overflow_sentinel_is_not_a_page_or_a_future_mark() {
        let mut bits = [0, 1 << 31];
        assert!(take_marked(&mut bits, 63).is_empty());
        assert_eq!(bits, [0, 0]);
        assert!(take_marked(&mut bits, 63).is_empty());
    }

    #[test]
    fn sparse_marks_are_ordered_and_consumed_once() {
        let mut bits = [0x8000_0001, 0x4000_0002];
        assert_eq!(take_marked(&mut bits, 63), [0, 31, 33, 62]);
        assert!(take_marked(&mut bits, 63).is_empty());
    }
}
