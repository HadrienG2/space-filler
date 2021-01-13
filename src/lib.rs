/// Integer type suitable for counting number of bits
///
/// Although 32-bit is very much overkill for this purpose, I am using this type
/// for interface compatibility with standard Rust integer methods.
///
pub type NumBits = u32;

/// Index of a point on a space-filling curve
///
/// Ideally, this crate would be generic over this type, but `const fn`
/// currently cannot handle this as const traits (and thus a version of
/// `num-traits` that can be used in const fn) is not yet available.
///
pub type CurveIdx = u16;

/// Coordinate of a point on a space-filling curve
///
/// Ideally, this crate would be generic over this type, but `const fn`
/// currently cannot handle this as const traits (and thus a version of
/// `num-traits` that can be used in const fn) is not yet available.
///
pub type Coord = u8;

/// Coordinates of a 2D point on a space-filling curve (in x, y order)
///
/// Ideally, this type would be generic to N dimensions, but this requires both
/// const generics, which aren't stable yet, and algorithmic adaptations which I
/// have not carried out yet as I haven't needed them so far.
///
pub type Coords2D = [u8; 2];

// ---

/// Count the number of bits of an integer
pub const fn num_bits<T>() -> NumBits {
    // TODO: Once assert in const is allowed, sanity check input
    // assert!(core::mem::size_of<T>() <= u32::MAX as usize);
    (core::mem::size_of::<T>() * 8) as _
}

/// Generate a mask with bit pattern 0000...0011
///
/// FIXME: Algorithm only supports power-of-two sizes at the moment
///
pub const fn low_order_mask(length: NumBits) -> CurveIdx {
    // TODO: Once assert in const is allowed, sanity check input
    // assert!(length.is_power_of_two() && length <= num_bits::<CurveIdx>());

    // Handle zero-sized mask edge case
    if length == 0 {
        return 0;
    }

    // Generate the mask
    let mut mask = 0b1;
    let mut curr_length = 1;
    while curr_length < length {
        mask |= mask << curr_length;
        curr_length *= 2;
    }
    mask
}

/// Generate a mask with bit pattern 00110011...0011
///
/// FIXME: Algorithm only supports power-of-two sizes at the moment
///
pub const fn striped_mask(stripe_length: NumBits) -> CurveIdx {
    // TODO: Once assert in const is allowed, sanity check input
    // assert!(length != 0 && length.is_power_of_two() && length < num_bits::<CurveIdx>());

    // Generate the stripes
    let mut stripes = low_order_mask(stripe_length);
    let mut curr_length = 2 * stripe_length;
    while curr_length < num_bits::<CurveIdx>() {
        stripes |= stripes << curr_length;
        curr_length *= 2;
    }
    stripes
}

// TODO: Other binary utilities used by Hilbert decoding

// ---

// TODO: Morton decoding

// ---

// TODO: Hilbert decoding

// TODO: Structure this into submodules
// TODO: Study the possibility of faster curve iteration schemes than
//       repeatedly decoding an increasing curve index.

#[cfg(test)]
mod tests {
    #[test]
    fn num_bits() {
        assert_eq!(super::num_bits::<u8>(), 8);
        assert_eq!(super::num_bits::<u16>(), 16);
        assert_eq!(super::num_bits::<u32>(), 32);
        assert_eq!(super::num_bits::<u64>(), 64);
        assert_eq!(super::num_bits::<u128>(), 128);
    }

    #[test]
    fn low_order_mask() {
        assert_eq!(super::low_order_mask(0), 0b0000000000000000);
        assert_eq!(super::low_order_mask(1), 0b0000000000000001);
        assert_eq!(super::low_order_mask(2), 0b0000000000000011);
        assert_eq!(super::low_order_mask(4), 0b0000000000001111);
        assert_eq!(super::low_order_mask(8), 0b0000000011111111);
        assert_eq!(super::low_order_mask(16), 0b1111111111111111);
    }

    #[test]
    fn striped_mask() {
        assert_eq!(super::striped_mask(1), 0b0101010101010101);
        assert_eq!(super::striped_mask(2), 0b0011001100110011);
        assert_eq!(super::striped_mask(4), 0b0000111100001111);
        assert_eq!(super::striped_mask(8), 0b0000000011111111);
    }
}
