# Model revisions, not moral recommendations

`nomos relax examples/symmetric-single-token/world.nomos.yaml --show-all-minimal` reports three single-change model variants **within a fixed, declared catalog**:

- allow_waste: leave the token unused and withdraw full utilization;
- divisible: allow equal half-token shares and change the physical indivisibility assumption;
- lottery: use rational 1/2 - 1/2 probability over (1,0) and (0,1), preserving ex-post full utilization but only distributional anonymity.

The checker verifies the new outcome models, exact rational weights and explicit assumption deltas. The results have no hidden welfare ranking. The catalog is not complete over possible institutional designs, and `--show-all-minimal` is defined relative only to this three-transform library.
