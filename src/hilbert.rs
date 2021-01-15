//! Utilities related to the Hilbert space-filling curve

use crate::{bits, morton, Coordinates2D, CurveIdx};

/// Compute the coordinate of the i-th point of a ]-shaped Hilbert curve
///
/// Compared to the Morton curve, the Hilbert curve never jumps across space, it
/// always moves from one point of space to one of its direct neighbors. The
/// price to pay for this superior spatial locality is that it follows a more
/// complex geometrical pattern (that, logically, requires more complex
/// computations), based on recursively flipped C-like shapes.
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
#[inline]
pub const fn decode_2d(code: CurveIdx) -> Coordinates2D {
    // TODO: Once assert in const is allowed, sanity check types
    // debug_assert!(num_bits::<Coordinate>() >= num_bits::<CurveIdx>() / 2);

    // Here's the mathematical derivation of this algorithm.
    //
    // ---
    //
    // Remember that we took this shape as our basic pattern:
    //
    // ├┐
    // <┘
    //
    // This means that...
    //
    // * On iteration 0, (x, y) is (0, 0)
    // * On iteration 1, (x, y) is (1, 0)
    // * On iteration 2, (x, y) is (1, 1)
    // * On iteration 3, (x, y) is (0, 1)
    //
    // So if we denote ij the binary digits of the iteration number, we have...
    //
    // * x = i XOR j
    // * y = i
    //
    // ...which happens to be the Gray code associated with the 2D Morton code
    // yx. This is totally not a coincidence, it simplifies extension to higher
    // dimensions, where both the Gray and Morton code are defined...
    //
    // Now, if we were to turn this basic shape into a fractal without extra
    // precautions, we would still get jumps from some sub-patterns to the next:
    //
    // 0┐1┐
    // <┘┌┘
    // 3┐2┐
    // <┘<┘
    //
    // To avoid this, we need to transform the sub-patterns, which we can do
    // without changing the curve's endpoints by swapping the coordinates of
    // sub-pattern 0...
    //
    // 0┌1┐
    // └┘┌┘
    // 3┐2┐
    // <┘<┘
    //
    // ...and swapping and inverting the coordinates of sub-pattern 3:
    //
    // 0┌1┐
    // └┘┌┘
    // ┌┐2┐
    // v3─┘
    //
    // Let's translate those transformations into binary arithmetic:
    //
    // - If (i XOR j) is 0, we need to swap the coordinates of our sub-pattern
    // - If (i AND j) is 1, we need to invert the coordinates of our sub-pattern
    //   * In binary, this can be done by NOT-ing x and y when (i AND j) is 1...
    //   * ...which we can do without testing the value of i and j by XORing x
    //     and y with (i AND j).
    //
    // With that, we get the first layer of fractal recursion, but then we must
    // recursively apply the same recursion rules to our transformed patterns at
    // the next level of recursion:
    //
    // 0┐┌─1┌─┐
    // ┌┘└┐└┘┌┘
    // │┌┐│┌┐└┐
    // └┘└┘│└─┘
    // ┌┐┌┐2┌─┐
    // │└┘│└┘┌┘
    // └┐┌┘┌┐└┐
    // <┘└3┘└─┘
    //
    // It so happens, however, that the transforms applied above are their own
    // inverse: swapping coordinates twice gives back the original coordinates,
    // and inverting coordinates twice gives back the original coordinates.
    //
    // Therefore, if for every level of recursion, we can compute a bit b that
    // controls whether a certain transform is applied when going to the next
    // level of recursion, the truth that we need to apply that transform at a
    // given recursion depth is given by the XOR of those control bits at all
    // previous recursion depths.
    //
    // ---
    //
    // Now, with that in mind, let's make the observation that when written out
    // in binary, the index of a point on the curve is [ i1 j1 i2 j2 ... iN jN ]
    // where (ix, jx) controls how the pattern is followed at recursion depth x.
    //
    // This looks very much like a 2D Morton code, and we can use a 2D Morton
    // code decoder to separate that index into two integers with bits
    // [ j1 j2 ... jN ] and [ i1 i2 ... iN ].
    //
    let [low_order, high_order] = morton::decode_2d(code);

    // From that, we can compute the binary combinations of i-s and j-s that we
    // need at every depth in order to move through the curve's basic ]-shaped
    // pattern and recurse to the next depth.
    //
    let and_bits = low_order & high_order; // Controls coordinate inversion
    let xor_bits = low_order ^ high_order; // Basic pattern's x coordinate
    let not_xor_bits = !(xor_bits); // Controls coordinate swapping

    // Then we can compute whether coordinates should be swapped or inverted
    // at every depth by computing the XOR of the recursive swapping/conversion
    // bits at every previous depth. This is most efficiently done by using a
    // bitwise version of the parallel scan algorithm.
    //
    let coord_swap_bits = bits::bitwise_xor_ltr_exclusive_scan(not_xor_bits);
    let coord_not_bits = bits::bitwise_xor_ltr_exclusive_scan(and_bits);

    // Finally, we start from the top-level Gray code coordinates, transform
    // every bit through coordinate swapping and inversion as appropriate, and
    // we get integer words whose bits are the coordinate on the Hilbert curve
    // at increasing recursion depths, which is what we want.
    //
    let [coord1, coord2] = bits::bitwise_swaps(coord_swap_bits, xor_bits, high_order);
    [coord1 ^ coord_not_bits, coord2 ^ coord_not_bits]
}

// TODO: Study if there's a faster way to iterate over the 2D Hilbert curve than
//       by repeatedly decoding increasing Hilbert curve indices

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Coordinate;
    use bits::test_utils::*;

    #[test]
    fn decode_2d() {
        for input in 0..=CurveIdx::MAX {
            let mut input_buf = input.reverse_bits();
            let mut results = [0; 2];
            let mut swap = false;
            let mut invert = false;
            for _bit_idx in 0..(bits::num_bits::<Coordinate>()) {
                let high_order_bit = pop_bit(&mut input_buf);
                let low_order_bit = pop_bit(&mut input_buf);
                let mut x_bit = high_order_bit ^ low_order_bit;
                let mut y_bit = high_order_bit;
                if swap {
                    core::mem::swap(&mut x_bit, &mut y_bit);
                }
                push_bit(&mut results[0], x_bit ^ invert);
                push_bit(&mut results[1], y_bit ^ invert);
                swap ^= !(high_order_bit ^ low_order_bit);
                invert ^= high_order_bit & low_order_bit;
            }
            assert_eq!(
                super::decode_2d(input),
                results,
                "Unexpected 2D Hilbert code decoding result for input {:08b}",
                input
            );
        }
    }
}
