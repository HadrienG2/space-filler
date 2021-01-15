pub(crate) mod bits;
pub mod hilbert;
pub mod morton;

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
