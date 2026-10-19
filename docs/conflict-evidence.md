# Conflict proof: single-token symmetry

State: two identical agents each demand one indivisible integer token; stock=1.

Full utilization demands a+b=1. Deterministic anonymity under swapping identical inputs demands a=b. Therefore 2a=1 contradicts integer-valued allocation. Enumerating all 4 integer outcomes confirms no witness simultaneously satisfies full utilization and deterministic anonymity.

The `nomos conflict` producer records all four outcome rows and the subset-minimal core. `nomos explain certificate.json` independently reconstructs the rows, verifies completeness, checks every field and tests the minimal core. A failure is not a certificate.

A subset-minimal conflict need not be minimum-cardinality in a general MUS problem. The current two-person instance is exhaustive and exact. This is MODEL_INFEASIBLE **only for the declared state and its outcome universe**; not a generalized social-choice theorem.
