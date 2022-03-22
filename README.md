# Some (specialized) space-filling curve implementations

This crate implements optimized algorithms for translating from an index on a 2D
Morton or Hilbert space-filling curve to the corresponding 2D coordinates.

---

The algorithms are implemented as `const fn`, which guarantees that the compiler
is technically able to perform any subset of the inner operations at compilation
time during the code optimization processes.

However, due to current limitations of `const fn`, this also means that I cannot
yet implement genericity over the integer type used for curve indexing
(currently, the algorithms are generic, but the function signatures aren't,
const traits and matching adaptations of the `num-traits` crates would be needed
for that). I will wait for this before considering publishing this crate for
general use.

Also, there is no guarantee at the moment that the compiler will actually
perform the work at compile time. This would require new compiler optimizer
directives, but failing that, once we have const generics, passing arguments as
const parameters should provide a reasonably reliable guarantee.

---

Generalization of the algorithms to N-dimensional space-filling curves should be
quite straightforward for the Morton curve and a bit more challenging but doable
for the Hilbert curve. The technical report "Compact Hilbert Indices" by Chris
Hamilton (CS-2006-07) provides a reference computation of N-dimensional Hilbert
curve properties that can be used to adapt the algorithm.

However, I don't have a need for these right now, so I didn't do the work yet.

---

As far as performance is concerned, I've researched the binary arithmetic quite
a bit and think it is reasonably optimal, but the code could probably be sped up
further by tabulating some results in strategic places. However, table lookup is
progressively becoming a hardware-specific optimization strategy these days, as
the relative cost of memory accesses with respect to integer arithmetic
operations is increasing over time and diverging across compute hardware (think
e.g. POWER vs x86, CPU vs GPU...).

Therefore, I would advise anyone investigating such an optimization strategy to
test it on a wide base of target hardware and pick the right degree of
tabulation for their hardware, which I personally don't have the luxury of doing
at the moment as I'm a bit starved for time.
