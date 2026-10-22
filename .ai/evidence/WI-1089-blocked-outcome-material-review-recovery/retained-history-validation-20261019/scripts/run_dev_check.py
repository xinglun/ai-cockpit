#!/usr/bin/env python3
"""Bounded development check recorder; never a native verification receipt."""
import json, os, signal, subprocess, sys, time
from datetime import datetime, timezone
from pathlib import Path
base=Path(__file__).parent.parent
root=Path('/workspace/ai-cockpit-wi1089-recovery')
label,*args=sys.argv[1:]
assert label and args and '/' not in label
log=base/(label+'.actual.log');meta_path=base/(label+'.actual.metadata.json')
assert not log.exists() and not meta_path.exists(), 'preserve previous attempts'
env=os.environ.copy()
env.update(PATH='/workspace/.cloud-tools/cargo/bin:'+env['PATH'], CARGO_HOME='/workspace/.cloud-tools/cargo', RUSTUP_HOME='/workspace/.cloud-tools/rustup', CARGO_TARGET_DIR='/workspace/scratch/wi1089-preserved-history-verify-target-20261019', CARGO_BUILD_JOBS='1', CARGO_INCREMENTAL='0', CARGO_NET_OFFLINE='true', RUST_TEST_THREADS='1')
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
started=time.monotonic();at=datetime.now(timezone.utc).isoformat()
meta={'kind':'actual_development_check_not_native_receipt','argv':args,'startedAt':at,'sourceHead':head,'terminalState':'running'}
meta_path.write_text(json.dumps(meta,indent=2)+'\n')
with log.open('wb') as f:
 p=subprocess.Popen(args,cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT,start_new_session=True)
 try:code=p.wait(timeout=880)
 except subprocess.TimeoutExpired:
  os.killpg(p.pid,signal.SIGTERM)
  try:p.wait(timeout=5)
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
  code=124
meta.update(endedAt=datetime.now(timezone.utc).isoformat(),exitCode=code,elapsedSeconds=time.monotonic()-started,terminalState='completed',sourceHeadAfter=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip())
meta_path.write_text(json.dumps(meta,indent=2)+'\n')
print(json.dumps(meta));print(log.read_text(errors='replace')[-10000:]);sys.exit(code)
