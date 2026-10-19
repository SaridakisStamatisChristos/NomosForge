# Explicit finite multi-resource trajectories

All resources in the first transition model use the same declared `token` unit so optional cross-resource coupling is dimensionally meaningful. Each resource has its own inventory, allocation policy, needs and critical flags. Every time step declares replenishment and adversarial loss explicitly. Executing a trajectory checks conservation `opening+replenishment = shock_loss+allocation+closing`, stock bounds, the single-step axioms and optional per-agent cross-resource caps.

`BOUNDED_TRACE_CHECKED` means the **supplied trace** passed; it is not a universally quantified safety theorem over all shock sequences. `COUNTEREXAMPLE` means a supplied executable trace violates one of the modeled constraints. No strategic population behavior is assumed.
