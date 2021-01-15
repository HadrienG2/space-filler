//! Utilities related to the Morton space-filling curve

use crate::{bits, Coordinates2D, CurveIdx};

/// Decode an 2-dimensional Morton code into its two inner indices
///
/// A Morton code combines two integers with bit patterns [ x1 x2 ... xN ] and
/// [ y1 y2 ... yN ] into the interleaved bit pattern [ y1 x1 y2 x2 ... yN xN ].
///
/// Decoding the set of Morton codes produces a fractal space-filling curve with
/// a recuring Z-shaped pattern that has reasonable spatial locality properties,
/// though it does brutally jump from one area of 2D space to another at times.
///
#[inline]
pub const fn decode_2d(code: CurveIdx) -> Coordinates2D {
    // TODO: Once assert in const is allowed, sanity check types
    // debug_assert!(num_bits::<Coordinate>() >= num_bits::<CurveIdx>() / 2);

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
        let mut sub_code = sub_codes[sub_code_idx] & bits::striped_mask(1);
        // We will then pack the coordinate's bits together by recursively
        // grouping them in pairs, groups of 4, and so on.
        // Initially, bits are isolated, so we have groups of one.
        // We're done once we have grouped half of the input bits together,
        // since the other bits will be zero.
        let mut group_size = 1;
        while group_size < bits::num_bits::<CurveIdx>() / 2 {
            // Duplicate the current bit pattern into neighboring zeroes on the
            // right in order to group pairs of subcode bits together
            // Iteration 1: [  0 a1 a1 a2 a2 a3 a3 a4 ... aN-2 aN-1 aN-1 aN ]
            // Iteration 2: [  0  0 a1 a2 a1 a2 a3 a4 ... aN-3 aN-2 aN-1 aN ]
            sub_code |= sub_code >> group_size;
            group_size *= 2;
            // Only keep the paired bit groups, zeroing out the rest
            // Iteration 1: [  0  0 a1 a2  0  0 a3 a4 ...    0    0 aN-1 aN ]
            // Iteration 2: [  0  0  0  0 a1 a2 a3 a4 ... aN-3 aN-2 aN-1 aN ]
            sub_code &= bits::striped_mask(group_size);
        }
        // Record the decoded coordinate and move to the next one
        sub_codes[sub_code_idx] = sub_code;
        sub_code_idx += 1;
    }
    [sub_codes[0] as _, sub_codes[1] as _]
}

// TODO: Study if there's a faster way to iterate over the 2D morton curve than
//       by repeatedly decoding increasing Morton curve indices

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Coordinate;
    use bits::test_utils::*;

    #[test]
    fn decode_2d() {
        for input in 0..=CurveIdx::MAX {
            let mut input_buf = input;
            let mut results = [0 as Coordinate; 2];
            for _bit_idx in 0..(bits::num_bits::<Coordinate>()) {
                for result in &mut results {
                    push_bit(result, pop_bit(&mut input_buf));
                }
            }
            for result in &mut results {
                *result = result.reverse_bits();
            }
            assert_eq!(
                super::decode_2d(input),
                results,
                "Unexpected 2D Morton code decoding result for input {:08b}",
                input
            );
        }
    }
}
