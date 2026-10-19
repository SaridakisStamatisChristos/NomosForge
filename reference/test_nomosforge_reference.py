"""Independent executable invariants for the bounded NomosForge Python reference."""
import itertools
from nomosforge_reference import World,Policy,allocation,violation,states,grammar,synthesize,impossibility_certificate

def test_world_count_is_exhaustive():
    assert len(states()) == 6 * 3**3 * 2**3 == 1296

def test_conservation_and_needs_for_entire_grammar():
    for w in states():
        for p in grammar():
            got = allocation(w, p)
            assert sum(got) <= w.stock
            assert all(0 <= a <= n for a,n in zip(got, w.needs))

def test_synthesis_succeeds_with_full_bounded_coverage():
    worlds=states();r=synthesize(worlds,grammar())
    assert r['status']=='BOUNDED_VERIFIED_IN_FINITE_GRAMMAR'
    assert r['world_count']==1296
    assert len(r['transcript'])>=2
    policy=Policy(**r['policy'])
    assert all(violation(w,allocation(w,policy)) is None for w in worlds)

def test_critical_and_peer_fairness_examples():
    w=World(2,(2,2,2),(1,0,0))
    assert violation(w,allocation(w,Policy('equal'))) == 'critical_first'
    assert violation(World(2,(2,2,0),(1,1,0)),(2,0,0)) == 'bounded_peer_disparity'
    assert violation(w,allocation(w,Policy('critical_equal'))) is None

def test_impossibility_search_is_bounded_and_sound():
    r=impossibility_certificate()
    assert len(r['possible_allocations']) == 4
    assert r['satisfying_allocations'] == 0
    assert r['minimal_conflict_cores'] == [('full_utilization','deterministic_anonymity')]
    assert any(row['full_utilization'] for row in r['possible_allocations'])
    assert any(row['deterministic_anonymity'] for row in r['possible_allocations'])

def test_no_grammar_solution_does_not_claim_global_impossibility():
    r=synthesize(states(),[Policy('index')])
    assert r['status']=='UNREALIZABLE_IN_FINITE_GRAMMAR'

def test_run_is_deterministic():
    worlds=states()
    assert synthesize(worlds,grammar()) == synthesize(worlds,grammar())

def test_metamorphic_capacity_and_extended_stock():
    for needs in itertools.product(range(3),repeat=3):
        for flags in itertools.product((0,1),repeat=3):
            previous=-1
            for stock in range(6):
                w=World(stock,needs,flags)
                total=sum(allocation(w,Policy('critical_equal')))
                assert total>=previous
                previous=total
