# Exhaustive finite CEGIS

1. Order the finite domain by the reference's precommitted witness-complexity ordering.
2. Choose the first typed grammar candidate that passes all cached counterexample states.
3. Exhaustively scan every declared state and return the first violating state.
4. Store an explicit tuple (candidate, state, observed allocation, violated axiom) and repeat.
5. If no grammar candidate remains, report UNREALIZABLE_IN_FINITE_GRAMMAR, **not** MODEL_INFEASIBLE.
6. If no violating state exists, report BOUNDED_VERIFIED_IN_FINITE_GRAMMAR.
7. If a challenge budget prevents exhaustive checking, report UNKNOWN.

The "shrinker" for the reference universe is first failure in ascending, predeclared complexity order; it is NOT a globally minimum witness by an unspecified metric. Every candidate and counterexample is deterministically replayable.
