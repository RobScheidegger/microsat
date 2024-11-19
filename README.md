# :microscope: microsat

A tiny (_microscopic_) DPLL SAT-solver written in Rust. This is not meant to be:

1. A particularly _fast_ solver
2. A particularly _extensible_ solver
3. A particularly _useful_ solver

But instead serves as a proof-of-concept for what a small, readable, and understandable [DPLL](https://en.wikipedia.org/wiki/DPLL_algorithm) SAT Solver could look like in Rust.

This project originated as a project for Brown's [CSCI2951-O Foundations of Prescriptive Analysis](https://cs.brown.edu/courses/csci2951-o/).

Authors:

- [Rob Scheidegger ](https://github.com/RobScheidegger)
- [Hammad Izhar](https://github.com/Hammad-Izhar)

## Benchmarks

Although `microsat` isn't intended to be used as a fast SAT-solver, I felt it appropriate to compare it at a basic level to the project, [`minisat`](https://github.com/niklasso/minisat) (a small [CDCL](https://en.wikipedia.org/wiki/Conflict-driven_clause_learning) SAT solver that disrupted the SAT solver scene many years back). Times were for release-compiled variants of `microsat` and `minisat` on the same computer, for all of the examples in `examples/cnf`:

|| `microsat`  | `minisat`  |
|---|---|---|
|Time to solve example suite| 44.158s  |  41.432s |
|Lines of code| 791  | 3517 |

As you can see, `microsat` does pretty remarkably well in this benchmark, despite being _much_ smaller than the already small `minisat`. Further, it is important to note that for any reasonably large instance (eg. larger than the `1040` variable, `3668` clause file in `examples/cnf`, which is the largest in this benchmark), so in a way, this benchmark is clearly cheating (but fascinating regardless).