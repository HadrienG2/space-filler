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
/// `num-traits` that can be used in const fn) are not yet available.
///
pub type CurveIdx = u16;

/// Coordinate of a point on a space-filling curve
///
/// Ideally, this crate would be generic over this type, but `const fn`
/// currently cannot handle this as const traits (and thus a version of
/// `num-traits` that can be used in const fn) are not yet available.
///
pub type Coordinate = u8;

/// Coordinates of a 2D point on a space-filling curve (in x, y order)
///
/// Ideally, this type would be generic to N dimensions, but this requires both
/// const generics, which aren't stable yet, and algorithmic adaptations which I
/// have not carried out yet as I haven't needed them so far.
///
pub type Coordinates2D = [u8; 2];

// ---

/// Count the number of bits of an integer
const fn num_bits<T>() -> NumBits {
    // TODO: Once assert in const is allowed, sanity check input
    // assert!(core::mem::size_of<T>() <= u32::MAX as usize);
    (core::mem::size_of::<T>() * 8) as _
}

/// Generate a mask that selects a certain number of low-order bits: 0000...0011
///
/// FIXME: Current algorithm only supports power-of-two lengths.
///
const fn low_order_mask(length: NumBits) -> CurveIdx {
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

/// Generate a mask with an alternating "striped" bit pattern: 00110011...0011
///
/// FIXME: Current algorithm only supports power-of-two lengths.
///
const fn striped_mask(stripe_length: NumBits) -> CurveIdx {
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

/// Compute the left-to-right inclusive XOR scan of an integer's bits
///
/// Given an integer with bits [ x1 x2 x3 ... ], this produces another integer
/// with bits [ x1  x1+x2  x1+x2+x3 ... ].
///
const fn bitwise_xor_ltr_inclusive_scan(mut bits: Coordinate) -> Coordinate {
    // This is a bitwise implementation of the Hillis/Steele parallel inclusive
    // scan algorithm. It can be trivially generalized to right-to-left scans or
    // other bitwise operations if there is demand.
    let mut stride = 1;
    while stride < num_bits::<Coordinate>() {
        // Iteration 0: [ x1     x2        x3           x4           x5 ... ]
        // Iteration 1: [ x1  x1+x2     x2+x3        x3+x4        x4+x5 ... ]
        // Iteration 2: [ x1  x1+x2  x1+x2+x3  x1+x2+x3+x4  x2+x3+x4+x5 ... ]
        bits ^= bits >> stride;
        stride *= 2;
    }
    bits
}

/// Compute the left-to-right exclusive XOR scan of an integer's bits
///
/// Given an integer with bits [ x1 x2 x3 ... ], this produces another integer
/// with bits [ 0 x1 x1+x2 ... ].
///
const fn bitwise_xor_ltr_exclusive_scan(bits: Coordinate) -> Coordinate {
    bitwise_xor_ltr_inclusive_scan(bits >> 1)
}

/// Conditionally swap two integers' bits according to a mask
///
/// Given an integer A with bits [ a1 a2 ... aN ], an integer B with bits
/// [ b1 b2 ... bN ], and a mask with bits [ m1 m2 ... mN ], this function
/// produces two integers:
///
/// - One whose bits are equal to ai where the corresponding mask bit is false
///   and to bi where the corresponding mask bit is true.
/// - One whose bits are conversely equal to bi where the corresponding mask
///   bit is false and to ai where the corresponding mask bit is true.
///
const fn bitwise_swaps(
    swap_mask: Coordinate,
    src1: Coordinate,
    src2: Coordinate,
) -> [Coordinate; 2] {
    let same_mask = !swap_mask;
    let res1 = (src1 & same_mask) | (src2 & swap_mask);
    let res2 = (src2 & same_mask) | (src1 & swap_mask);
    [res1, res2]
}

// ---

/// Decode an 2-dimensional Morton code into its two inner indices
///
/// A Morton code combines two integers with bit patterns [ x1 x2 ... xN ] and
/// [ y1 y2 ... yN ] into the interleaved bit pattern [ y1 x1 y2 x2 ... yN xN ].
///
/// Decoding the set of Morton codes produces a fractal space-filling curve with
/// a recuring Z-shaped pattern that has reasonable spatial locality properties,
/// though it does brutally jump from one area of 2D space to another at times.
///
pub const fn decode_morton_2d(code: CurveIdx) -> Coordinates2D {
    // TODO: Once assert in const is allowed, sanity check types
    // debug_assert!(num_bits::<Coordinates2D>() >= num_bits::<CurveIdx>() / 2);

    // Align the low-order bits of the two input sub-codes:
    // [ XX x1 XX x2 XX x3 XX x4 ... xN-1   XX xN ]
    // [ XX y1 XX y2 XX y3 XX y4 ... yN-1   XX yN ]
    let mut sub_codes = [code, code >> 1];
    let mut sub_code_idx = 0;
    while sub_code_idx < 2 {
        // We start with a coordinate's bits interleaved with irrelevant junk:
        // [ XX a1 XX a2 XX a3 XX a4 ... XX aN-1 XX aN ]
        // Let's clean that up by zeroing out the junk:
        // [  0 a1  0 a2  0 a3  0 a4 ...  0 aN-1  0 aN ]
        let mut sub_code = sub_codes[sub_code_idx] & striped_mask(1);
        // We will then pack the coordinate's bits together by recursively
        // grouping them in pairs, groups of 4, and so on.
        // Initially, bits are isolated, so we have groups of one.
        // We're done once we have grouped half of the input bits together,
        // since the other bits will be zero.
        let mut group_size = 1;
        while group_size < num_bits::<CurveIdx>() / 2 {
            // Duplicate the current bit pattern into neighboring zeroes on the
            // right in order to group pairs of subcode bits together
            // Iteration 1: [  0 a1 a1 a2 a2 a3 a3 a4 ... aN-2 aN-1 aN-1 aN ]
            // Iteration 2: [  0  0 a1 a2 a1 a2 a3 a4 ... aN-3 aN-2 aN-1 aN ]
            sub_code |= sub_code >> group_size;
            group_size *= 2;
            // Only keep the paired bit groups, zeroing out the rest
            // Iteration 1: [  0  0 a1 a2  0  0 a3 a4 ...    0    0 aN-1 aN ]
            // Iteration 2: [  0  0  0  0 a1 a2 a3 a4 ... aN-3 aN-2 aN-1 aN ]
            sub_code &= striped_mask(group_size);
        }
        // Record the decoded coordinate and move to the next one
        sub_codes[sub_code_idx] = sub_code;
        sub_code_idx += 1;
    }
    [sub_codes[0] as _, sub_codes[1] as _]
}

// TODO: Study if there's a faster way to iterate over the 2D morton curve than
//       by repeatedly decoding increasing Morton curve indices

// ---

/// Compute the coordinate of the i-th point of a ]-shaped Hilbert curve
///
/// Compared to the Morton curve, the Hilbert curve never jumps across space, it
/// always moves from one point of space to one of its direct neighbors. The
/// price to pay for this superior spatial locality is that it follows a more
/// complex geometrical pattern (that, transitively, requires more complex
/// computations), based on recursively rotated and flipped C-like shapes.
///
/// There are technically 4 C-like shapes that one could start from. Here we use
/// a vertically flipped C shape (]) instead of the U shape that is more
/// commonly seen in literature, because an extension to N dimensions and
/// non-square domains has been performed in the "Compact Hilbert Indices"
/// paper by Chris Hamilton (ref: CS-2006-07) for this particular variation of
/// the Hilbert curve, and a ] is just a coordinate transpose away from a U.
///
/// The first 256 iterations of this Hilbert curve look like this:
///
/// ┬┌─┐┌─┐┌─┐┌─┐┌─┐
/// └┘┌┘└┐└┘┌┘└┐└┘┌┘
/// ┌┐└┐┌┘┌┐│┌┐│┌┐└┐
/// │└─┘└─┘│└┘└┘│└─┘
/// └┐┌──┐┌┘┌┐┌┐│┌─┐
/// ┌┘└┐┌┘└┐│└┘│└┘┌┘
/// │┌┐││┌┐│└┐┌┘┌┐└┐
/// └┘└┘└┘└┘┌┘└─┘└─┘
/// ┌┐┌┐┌┐┌┐└┐┌─┐┌─┐
/// │└┘││└┘│┌┘└┐└┘┌┘
/// └┐┌┘└┐┌┘│┌┐│┌┐└┐
/// ┌┘└──┘└┐└┘└┘│└─┘
/// │┌─┐┌─┐│┌┐┌┐│┌─┐
/// └┘┌┘└┐└┘│└┘│└┘┌┘
/// ┌┐└┐┌┘┌┐└┐┌┘┌┐└┐
/// v└─┘└─┘└─┘└─┘└─┘
///
pub const fn decode_hilbert_2d(code: CurveIdx) -> Coordinates2D {
    // TODO: This warrants some explanation
    let [low_order, high_order] = decode_morton_2d(code);

    let and_bits = low_order & high_order;
    let xor_bits = low_order ^ high_order;
    let not_xor_bits = !(xor_bits);

    let not_control_bits = bitwise_xor_ltr_exclusive_scan(and_bits);
    let swap_control_bits = bitwise_xor_ltr_exclusive_scan(not_xor_bits);

    let [coord1, coord2] = bitwise_swaps(swap_control_bits, xor_bits, high_order);
    [coord1 ^ not_control_bits, coord2 ^ not_control_bits]
}

// TODO: Add tests

// TODO: Study if there's a faster way to iterate over the 2D Hilbert curve than
//       by repeatedly decoding increasing Hilbert curve indices

// TODO: Restructure this into submodules

#[cfg(test)]
mod tests {
    use super::*;
    use core::ops::ShrAssign;
    use num_traits::{PrimInt, Unsigned};
    use quickcheck::quickcheck;

    fn push_bit<I: PrimInt + Unsigned>(target: &mut I, bit: I) {
        assert!(bit & I::one() == bit);
        *target = (*target << 1) | bit
    }

    fn peek_bit<I: PrimInt + Unsigned>(target: I) -> I {
        target & I::one()
    }

    fn pop_bit<I: PrimInt + Unsigned + ShrAssign<NumBits>>(target: &mut I) -> I {
        let res = peek_bit(*target);
        *target >>= 1;
        res
    }

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

    #[test]
    fn bitwise_xor_ltr_inclusive_scan() {
        let num_bits = super::num_bits::<Coordinate>();
        for input in 0..=Coordinate::MAX {
            let mut input_buf = input.reverse_bits();
            let mut result = 0;
            for _bit_idx in 0..num_bits {
                let input_bit = pop_bit(&mut input_buf);
                let new_bit = peek_bit(result) ^ input_bit;
                push_bit(&mut result, new_bit);
            }
            assert_eq!(
                super::bitwise_xor_ltr_inclusive_scan(input),
                result,
                "Unexpected inclusive scan result for input {}",
                input
            );
        }
    }

    #[test]
    fn bitwise_xor_ltr_exclusive_scan() {
        for input in 0..=Coordinate::MAX {
            assert_eq!(
                super::bitwise_xor_ltr_exclusive_scan(input),
                super::bitwise_xor_ltr_inclusive_scan(input) >> 1,
                "Unexpected exclusive scan result for input {}",
                input
            );
        }
    }

    mod bitwise_swaps {
        use super::*;

        // The iteration space of this exhaustive test is a bit large, so it's not a
        // good idea to run it in debug mode...
        #[test]
        #[ignore]
        fn exhaustive() {
            for mask in 0..=Coordinate::MAX {
                for src1 in 0..=Coordinate::MAX {
                    for src2 in 0..=Coordinate::MAX {
                        test(mask, src1, src2);
                    }
                }
            }
        }

        // ...instead, random testing should be good enough for most purposes
        quickcheck! {
            fn quick(mask: Coordinate, src1: Coordinate, src2: Coordinate) -> bool {
                test(mask, src1, src2);
                true
            }
        }

        // Whichever way you probe the parameter space, for each set of
        // parameters, we perform the following check:
        fn test(mask: Coordinate, src1: Coordinate, src2: Coordinate) {
            let [mut mask_buf, mut src1_buf, mut src2_buf] = [mask, src1, src2];
            let mut results = [0; 2];
            for _bit_idx in 0..(super::super::num_bits::<Coordinate>()) {
                match pop_bit(&mut mask_buf) {
                    0 => {
                        push_bit(&mut results[0], pop_bit(&mut src1_buf));
                        push_bit(&mut results[1], pop_bit(&mut src2_buf));
                    }
                    1 => {
                        push_bit(&mut results[0], pop_bit(&mut src2_buf));
                        push_bit(&mut results[1], pop_bit(&mut src1_buf));
                    }
                    _ => unreachable!(),
                }
            }
            for result in &mut results {
                *result = result.reverse_bits();
            }
            assert_eq!(
                super::super::bitwise_swaps(mask, src1, src2),
                results,
                "Unexpected bitwise swap result for src1={}, src2={}, mask={}",
                src1,
                src2,
                mask
            );
        }
    }

    #[test]
    fn decode_morton_2d() {
        for input in 0..=CurveIdx::MAX {
            let mut input_buf = input;
            let mut results = [0; 2];
            for _bit_idx in 0..(super::num_bits::<Coordinate>()) {
                for result in &mut results {
                    push_bit(result, pop_bit(&mut input_buf) as Coordinate);
                }
            }
            for result in &mut results {
                *result = result.reverse_bits();
            }
            assert_eq!(
                super::decode_morton_2d(input),
                results,
                "Unexpected 2D Morton code decoding result for input {}",
                input
            );
        }
    }
}
