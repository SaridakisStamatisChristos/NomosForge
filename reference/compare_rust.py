#!/usr/bin/env python3
"""Independent 1,296 × 16 differential comparison; no timing claims."""
from dataclasses import asdict
import json
import sys
from pathlib import Path
from nomosforge_reference import states,grammar,allocation,violation,synthesize

def main() -> None:
    if len(sys.argv)!=3:
        raise SystemExit('usage: compare_rust.py rust-corpus.jsonl rust-synthesis.json')
    worlds=states()
    policies=grammar()
    count=0
    with Path(sys.argv[1]).open() as stream:
        for policy in policies:
            for world in worlds:
                line=stream.readline()
                if not line:
                    raise AssertionError(f'corpus truncated at {count}')
                got=json.loads(line)
                expected={'policy':asdict(policy),'world':asdict(world),
                          'allocation':list(allocation(world,policy)),
                          'violation':violation(world,allocation(world,policy))}
                # Normalize Python tuples to their JSON array representation before comparison.
                expected=json.loads(json.dumps(expected))
                if got!=expected:
                    raise AssertionError(f'Rust/Python disagreement at item {count}: {got} vs {expected}')
                count+=1
        if stream.readline():
            raise AssertionError('extra Rust corpus entries')
    assert count==20736
    report=json.loads(Path(sys.argv[2]).read_text())
    expected=synthesize(worlds,policies)
    for key in ('status','grammar_size','world_count','policy','transcript','witnesses'):
        assert report[key]==json.loads(json.dumps(expected[key])),f'synthesis mismatch: {key}'
    print(f'DIFFERENTIAL_VERIFIED: {count} allocation+axiom results; CEGIS transcript matches Python')

if __name__=='__main__':
    main()
