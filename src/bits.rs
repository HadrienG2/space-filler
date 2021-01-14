//! Binary arithmetic utilities used for space-filling curve computations

use crate::{Coordinate, CurveIdx, NumBits};

/// Count the number of bits of an integer
pub const fn num_bits<T>() -> NumBits {
    // TODO: Once assert in const is allowed, sanity check input
    // assert!(core::mem::size_of<T>() <= NumBits::MAX as usize);
    (core::mem::size_of::<T>() * 8) as _
}

/// Generate a mask that selects a certain number of low-order bits: 0000...0011
///
/// FIXME: Current algorithm only supports power-of-two lengths.
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
        // Iteration 0: 00...00001
        // Iteration 1: 00...00011
        // Iteration 2: 00...01111
        mask |= mask << curr_length;
        curr_length *= 2;
    }
    mask
}

/// Generate a mask with an alternating "striped" bit pattern: 00110011...0011
///
/// FIXME: Current algorithm only supports power-of-two lengths.
///
pub const fn striped_mask(stripe_length: NumBits) -> CurveIdx {
    // TODO: Once assert in const is allowed, sanity check input
    // assert!(length != 0 && length.is_power_of_two() && length < num_bits::<CurveIdx>());

    // Generate the stripes
    let mut stripes = low_order_mask(stripe_length);
    let mut curr_length = 2 * stripe_length;
    while curr_length < num_bits::<CurveIdx>() {
        // Iteration 0: 00...00000000000000011
        // Iteration 1: 00...00000000000110011
        // Iteration 2: 00...00011001100110011
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
pub const fn bitwise_xor_ltr_inclusive_scan(mut bits: Coordinate) -> Coordinate {
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
pub const fn bitwise_xor_ltr_exclusive_scan(bits: Coordinate) -> Coordinate {
    bitwise_xor_ltr_inclusive_scan(bits >> 1)
}

/// Conditionally swap two integers' bits according to a mask
///
/// Given an integer A with bits [ a1 a2 ... aN ], an integer B with bits
/// [ b1 b2 ... bN ], and a mask with bits [ m1 m2 ... mN ], this function
/// produces two integers:
///
/// - One whose bits are equal to ai where the corresponding mask bit mi is
///   false and to bi where mi is true.
/// - One whose bits are conversely equal to bi where the corresponding mask
///   bit mi is false and to ai where mi is true.
///
pub const fn bitwise_swaps(
    swap_mask: Coordinate,
    src1: Coordinate,
    src2: Coordinate,
) -> [Coordinate; 2] {
    let same_mask = !swap_mask;
    let res1 = (src1 & same_mask) | (src2 & swap_mask);
    let res2 = (src2 & same_mask) | (src1 & swap_mask);
    [res1, res2]
}

#[cfg(test)]
pub(crate) mod test_utils {
    use crate::NumBits;
    use core::ops::ShrAssign;
    use num_traits::{PrimInt, Unsigned};

    pub fn push_bit<I: PrimInt + Unsigned>(target: &mut I, bit: bool) {
        let bit = if bit { I::one() } else { I::zero() };
        *target = (*target << 1) | bit
    }

    pub fn peek_bit<I: PrimInt + Unsigned>(target: I) -> bool {
        (target & I::one()) == I::one()
    }

    pub fn pop_bit<I: PrimInt + Unsigned + ShrAssign<NumBits>>(target: &mut I) -> bool {
        let res = peek_bit(*target);
        *target >>= 1;
        res
    }
}

#[cfg(test)]
mod tests {
    use super::{test_utils::*, *};
    use quickcheck::quickcheck;

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
                "Unexpected inclusive scan result for input {:08b}",
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
                "Unexpected exclusive scan result for input {:08b}",
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
            let mut results = [0 as Coordinate; 2];
            for _bit_idx in 0..(super::super::num_bits::<Coordinate>()) {
                match pop_bit(&mut mask_buf) {
                    false => {
                        push_bit(&mut results[0], pop_bit(&mut src1_buf));
                        push_bit(&mut results[1], pop_bit(&mut src2_buf));
                    }
                    true => {
                        push_bit(&mut results[0], pop_bit(&mut src2_buf));
                        push_bit(&mut results[1], pop_bit(&mut src1_buf));
                    }
                }
            }
            for result in &mut results {
                *result = result.reverse_bits();
            }
            assert_eq!(
                super::super::bitwise_swaps(mask, src1, src2),
                results,
                "Unexpected bitwise swap result for src1={:08b}, src2={:08b}, mask={:08b}",
                src1,
                src2,
                mask
            );
        }
    }
}
