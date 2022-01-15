//! Hilbert curves in your terminal!
//!
//! I originally wrote this as a manual algorithm validation tool, and kept it
//! around because I think it just looks cool :)

use space_filler::{hilbert, Coordinates2D, CurveIdx};

// Display a Hilbert curve of specified order
fn print_hilbert(order: u8) {
    // Print header
    println!("--- At order {order} ---\n");

    // Compute a Hilbert curve's coordinates
    let coord_range = 2usize.pow(order as u32);
    let num_points = coord_range * coord_range;
    let coordinates = (0..num_points)
        .map(|idx| {
            // Here, we simulate a low-order curve from a higher-order one by
            // swapping coordinates when order is odd.
            let [mut x, mut y] = hilbert::decode_2d(idx as CurveIdx);
            if order % 2 == 1 {
                core::mem::swap(&mut x, &mut y);
            }
            [x, y]
        })
        .collect::<Vec<_>>();

    // Set up a 2D character-based display
    let mut display = (0..(num_points + coord_range))
        .map(|idx| {
            if idx % (coord_range + 1) == coord_range {
                '\n'
            } else {
                // This character is a placeholder that should not persist in
                // the final program output.
                '@'
            }
        })
        .collect::<Vec<_>>();
    let to_index = |coords: Coordinates2D| {
        assert!(
            coords.iter().all(|&coord| (coord as usize) < coord_range),
            "Coordinates out of range: {coords:?} for range {coord_range}"
        );
        (coords[1] as usize) * (coord_range + 1) + (coords[0] as usize)
    };
    let to_dir = |src: Coordinates2D, dst: Coordinates2D| {
        [
            dst[0] as isize - src[0] as isize,
            dst[1] as isize - src[1] as isize,
        ]
    };

    // Draw the start of the curve
    let start = coordinates[0];
    let next = coordinates[1];
    display[to_index(start)] = match to_dir(start, next) {
        [0, -1] => '┴',
        [1, 0] => '├',
        [0, 1] => '┬',
        [-1, 0] => '┤',
        _ => unreachable!("Hilbert curve moves by single-coordinate steps"),
    };

    // Draw the end of the curve
    let end = coordinates[num_points - 1];
    let prev = coordinates[num_points - 2];
    display[to_index(end)] = match to_dir(prev, end) {
        [0, -1] => '^',
        [1, 0] => '>',
        [0, 1] => 'v',
        [-1, 0] => '<',
        _ => unreachable!("Hilbert curve moves by single-coordinate steps"),
    };

    // Draw the middle of the curve
    for window in coordinates.windows(3) {
        let path = match (to_dir(window[0], window[1]), to_dir(window[1], window[2])) {
            ([-1, 0], [0, -1]) | ([0, 1], [1, 0]) => '└',
            ([-1, 0], [0, 1]) | ([0, -1], [1, 0]) => '┌',
            ([1, 0], [1, 0]) | ([-1, 0], [-1, 0]) => '─',
            ([1, 0], [0, -1]) | ([0, 1], [-1, 0]) => '┘',
            ([0, 1], [0, 1]) | ([0, -1], [0, -1]) => '│',
            ([1, 0], [0, 1]) | ([0, -1], [-1, 0]) => '┐',
            _ => unreachable!("Hilbert curve moves by single-coordinate steps and doesn't go back"),
        };
        display[to_index(window[1])] = path;
    }

    // Display the curve
    let display_string = display.into_iter().collect::<String>();
    println!("{display_string}");
}

// Display the Hilbert curve at a few orders
fn main() {
    println!();
    for order in 1..=8 {
        print_hilbert(order);
    }
}
