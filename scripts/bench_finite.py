#!/usr/bin/env python3
"""Descriptive end-to-end CLI scaling measurements, never a core-kernel speedup claim."""
import argparse
import json
import platform
import subprocess
import tempfile
import time
from pathlib import Path
def main():
    ap=argparse.ArgumentParser()
    ap.add_argument('--binary',default='target/debug/nomos')
    ap.add_argument('--repeats',type=int,default=3)
    args=ap.parse_args()
    if args.repeats<1: raise SystemExit('repeats must be >=1')
    import yaml
    template=yaml.safe_load(Path('examples/water-allocation/world.nomos.yaml').read_text())
    rows=[]
    for stock_max in (1,3,5):
        template['verification']['stock']['max']=stock_max
        template['model']['resources'][0]['max_stock']=stock_max
        with tempfile.TemporaryDirectory() as d:
            f=Path(d)/'world.yaml'
            f.write_text(yaml.safe_dump(template))
            for trial in range(args.repeats):
                start=time.perf_counter_ns()
                p=subprocess.run([args.binary,'synth',str(f)],capture_output=True,text=True)
                ms=(time.perf_counter_ns()-start)/1e6
                if p.returncode!=0: raise RuntimeError(p.stderr or p.stdout)
                obj=json.loads(p.stdout)
                rows.append({'stock_max':stock_max,'domain_states':obj['world_count'],
                    'trial':trial,'elapsed_ms_end_to_end':ms,'host':platform.platform(),
                    'python':platform.python_version(),'status':obj['status']})
    print(json.dumps({'scope':'end-to-end CLI, includes startup/JSON; not comparable to raw solver cores',
        'measurements':rows},indent=2))
if __name__=='__main__': main()
