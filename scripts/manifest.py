#!/usr/bin/env python3
"""Compute real SHA-256 digests of versioned research source files; no hardcoded hashes."""
import hashlib
import json
import pathlib
import subprocess
import sys
root=pathlib.Path(__file__).resolve().parent.parent
proc=subprocess.run(["git","ls-files"],cwd=root,check=True,capture_output=True,text=True)
files=[]
for path in sorted(proc.stdout.splitlines()):
    p=root/path
    if p.is_file() and ".git" not in p.parts:
        files.append({"path":path,"sha256":hashlib.sha256(p.read_bytes()).hexdigest(),"bytes":p.stat().st_size})
result={"schema":"nomos/file-manifest/v0.1","commit":subprocess.run(["git","rev-parse","HEAD"],cwd=root,check=True,capture_output=True,text=True).stdout.strip(),"files":files}
json.dump(result,sys.stdout,indent=2)
print()
