#!/usr/bin/env python3
"""Original bounded water-allocation reference semantics; no real-society claims."""
from __future__ import annotations
import argparse
import itertools
import json
from dataclasses import asdict, dataclass
from pathlib import Path

@dataclass(frozen=True)
class World:
    stock: int
    needs: tuple[int, ...]
    critical: tuple[int, ...]

    def __post_init__(self) -> None:
        if len(self.needs) != len(self.critical) or not self.needs:
            raise ValueError('needs and critical must have the same nonzero length')
        if self.stock < 0 or any(n < 0 for n in self.needs):
            raise ValueError('stock and needs must be nonnegative')
        if any(c not in (0, 1) for c in self.critical):
            raise ValueError('critical flags must be 0 or 1')

@dataclass(frozen=True)
class Policy:
    priority: str
    reserve: int = 0

    def __post_init__(self) -> None:
        if self.priority not in {
            'index', 'equal', 'needs_desc', 'needs_asc',
            'critical_index', 'critical_equal',
            'critical_needs_desc', 'critical_needs_asc'
        }:
            raise ValueError('unknown priority operator')
        if self.reserve < 0:
            raise ValueError('negative reserve')

def allocation(world: World, policy: Policy) -> tuple[int, ...]:
    given = [0] * len(world.needs)
    remaining = max(0, world.stock - policy.reserve)
    def key(i: int) -> tuple[int, ...]:
        p = policy.priority
        if p == 'index':
            return (i,)
        if p == 'equal':
            return (given[i], i)
        if p == 'needs_desc':
            return (-world.needs[i], given[i], i)
        if p == 'needs_asc':
            return (world.needs[i], given[i], i)
        if p == 'critical_index':
            return (-world.critical[i], i)
        if p == 'critical_equal':
            return (-world.critical[i], given[i], i)
        if p == 'critical_needs_desc':
            return (-world.critical[i], -world.needs[i], given[i], i)
        return (-world.critical[i], world.needs[i], given[i], i)
    while remaining:
        eligible = [i for i, need in enumerate(world.needs) if given[i] < need]
        if not eligible:
            break
        chosen = min(eligible, key=key)
        given[chosen] += 1
        remaining -= 1
    return tuple(given)

def violation(world: World, given: tuple[int, ...]) -> str | None:
    if len(given) != len(world.needs) or any(x < 0 for x in given):
        return 'nonnegative_allocation'
    if any(a > need for a, need in zip(given, world.needs)):
        return 'need_cap'
    if sum(given) > world.stock:
        return 'conservation'
    if sum(given) != min(world.stock, sum(world.needs)):
        return 'nonwaste'
    if any(world.critical[i] and given[i] < world.needs[i] for i in range(len(given))):
        if any(not world.critical[j] and given[j] > 0 for j in range(len(given))):
            return 'critical_first'
    for i in range(len(given)):
        for j in range(i + 1, len(given)):
            if world.needs[i] == world.needs[j] and world.critical[i] == world.critical[j]:
                if abs(given[i] - given[j]) > 1:
                    return 'bounded_peer_disparity'
    return None

def states() -> list[World]:
    out = [World(stock, needs, flags)
           for stock in range(6)
           for needs in itertools.product(range(3), repeat=3)
           for flags in itertools.product((0, 1), repeat=3)]
    return sorted(out, key=lambda x: (sum(x.needs) + x.stock + sum(x.critical),
                                     sum(x.needs), x.stock, x.needs, x.critical))

def grammar() -> list[Policy]:
    operators = ('index', 'equal', 'needs_desc', 'needs_asc',
                 'critical_index', 'critical_equal',
                 'critical_needs_desc', 'critical_needs_asc')
    return [Policy(p, reserve) for reserve in (0, 1) for p in operators]

def synthesize(worlds: list[World], programs: list[Policy]) -> dict:
    witnesses: list[World] = []
    transcript: list[dict] = []
    while True:
        candidate = next((p for p in programs if all(
            violation(w, allocation(w, p)) is None for w in witnesses)), None)
        if candidate is None:
            return {'status': 'UNREALIZABLE_IN_FINITE_GRAMMAR',
                    'grammar_size': len(programs), 'world_count': len(worlds),
                    'transcript': transcript, 'witnesses': [asdict(x) for x in witnesses]}
        bad = next(((w, allocation(w, candidate)) for w in worlds
                    if violation(w, allocation(w, candidate)) is not None), None)
        if bad is None:
            return {'status': 'BOUNDED_VERIFIED_IN_FINITE_GRAMMAR',
                    'grammar_size': len(programs), 'world_count': len(worlds),
                    'policy': asdict(candidate), 'transcript': transcript,
                    'witnesses': [asdict(x) for x in witnesses]}
        w, result = bad
        reason = violation(w, result)
        transcript.append({'candidate': asdict(candidate), 'counterexample': asdict(w),
                           'observed_allocation': result, 'violated_axiom': reason})
        witnesses.append(w)

def impossibility_certificate() -> dict:
    world = World(stock=1, needs=(1, 1), critical=(0, 0))
    all_alloc = [tuple(a) for a in itertools.product((0, 1), repeat=2)]
    rows = []
    for a in all_alloc:
        within_capacity = sum(a) <= 1
        full_use = sum(a) == 1
        strict_anonymity = a[0] == a[1]
        rows.append({'allocation': a, 'capacity': within_capacity,
                     'full_utilization': full_use, 'deterministic_anonymity': strict_anonymity,
                     'all': within_capacity and full_use and strict_anonymity})
    axioms = ('capacity', 'full_utilization', 'deterministic_anonymity')
    def unsatisfiable(subset: tuple[str, ...]) -> bool:
        return not any(all(r[axiom] for axiom in subset) for r in rows)
    minimal_cores = []
    for size in range(1, len(axioms) + 1):
        for subset in itertools.combinations(axioms, size):
            if unsatisfiable(subset) and all(not unsatisfiable(proper)
                for length in range(len(subset))
                for proper in itertools.combinations(subset, length)):
                minimal_cores.append(subset)
    return {
        'world': asdict(world),
        'minimal_conflict_cores': minimal_cores,
        'axioms': ['integer_ex_post_allocation', 'full_utilization', 'deterministic_anonymity'],
        'possible_allocations': rows,
        'satisfying_allocations': sum(r['all'] for r in rows),
        'claim_scope': 'This single input is impossible under a deterministic anonymous integer-valued allocator. It does not rule out lotteries or divisible resources.',
        'relaxations': {
            'lottery': '50%-50% selection of (1,0) vs (0,1); anonymous distribution with full ex-post utilization',
            'divisible': '(0.5,0.5) if divisibility is explicitly permitted; deterministic and anonymous',
            'allow_waste': '(0,0) if full utilization is relaxed'
        }
    }

def experiment() -> dict:
    worlds = states()
    result = synthesize(worlds, grammar())
    result['impossibility'] = impossibility_certificate()
    result['universe'] = {'agents': 3, 'needs': '0..2 per agent', 'critical': '0 or 1',
                          'stock': '0..5', 'model': 'single-step, one indivisible resource'}
    result['evidence'] = 'Exhaustive *bounded* testing; not a formal verification of real institutions'
    return result

def main() -> None:
    parser = argparse.ArgumentParser(description='NomosForge bounded program synthesis')
    parser.add_argument('--output', type=Path, default=Path('nomosforge_experiment_results.json'))
    args = parser.parse_args()
    report = experiment()
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({k: report[k] for k in ('status','grammar_size','world_count','policy')}, indent=2))
    print(f"CEGIS adversarial rounds: {len(report['transcript'])}")
    print(f"Impossibility satisfying allocations: {report['impossibility']['satisfying_allocations']}")
    print(f"JSON report: {args.output}")

if __name__ == '__main__':
    main()
