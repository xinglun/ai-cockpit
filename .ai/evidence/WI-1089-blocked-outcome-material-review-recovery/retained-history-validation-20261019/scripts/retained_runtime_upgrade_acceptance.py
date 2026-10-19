#!/usr/bin/env python3
"""Actual isolated retained-history fixture; not Sentinel acceptance or human authority."""
from pathlib import Path
import shutil
prefix=(Path(__file__).parent/'retained_canonical_baseline.py').read_text().split('run("git-init"')[0]
exec(compile(prefix,'baseline-helper-definitions','exec'))
source=base/'fixture-01'
shutil.copytree(source/'repository',root,dirs_exist_ok=True,ignore=shutil.ignore_patterns('fixture-target'))
old_bytes=(root/f'.ai/evidence/{wid}.verification.json').read_bytes()
old=json.loads(old_bytes)
old_outcome=(root/f'.ai/work-items/active/{wid}.outcome.json').read_bytes()
(case/'old-canonical.input.actual.json').write_bytes(old_bytes)
(case/'old-blocked-outcome.input.actual.json').write_bytes(old_outcome)
assert old['passed'] and old['receipt']['nodesExecuted']==4 and old['receipt']['nodesReused']==0
assert json.loads(old_outcome)['state']=='blocked'
binary=Path('/workspace/scratch/wi1089-preserved-history-verify-target-20261019/debug/ai-cockpit')
new_digest='sha256:'+hashlib.sha256(binary.read_bytes()).hexdigest()
assert new_digest!=old['runtimeDigest']
relative=f'.ai/work-items/active/{wid}.contract.json'
cli('current-candidate-preflight',['preflight','--contract',relative])
review('current-candidate-synthetic-contract-review')
status=json.loads(cli('current-candidate-status',['work-item','status','--id',wid,'--json']).stdout)
assert status['sourceDigests']['repositorySnapshot']==old['repositorySnapshotDigest'],status
request,_=material('current-candidate-material')
plan=json.loads(cli('current-candidate-verify-plan',['verify','--work-item',wid,'--workers','1','--plan-only']).stdout)
assert plan['processesSpawned']==0 and plan['nodesPlanned']==4 and plan['nodesToExecute']==4 and plan['nodesReused']==0,plan
assert (root/f'.ai/evidence/{wid}.verification.json').read_bytes()==old_bytes
cli('current-candidate-four-actual-checks',['verify','--work-item',wid,'--workers','1'])
new_bytes=(root/f'.ai/evidence/{wid}.verification.json').read_bytes();new=json.loads(new_bytes)
assert new['passed'] and new['runtimeDigest']==new_digest
receipt=new['receipt'];assert receipt['nodesExecuted']==4 and receipt['nodesReused']==0 and receipt['processesSpawned']==4 and receipt['maxConcurrentProcesses']==1,receipt
assert len(receipt['executionRecords'])==4 and all(r['spawned'] and r['passed'] and r['exitCode']==0 for r in receipt['executionRecords'])
summary=document('summary');assert summary['preflightState']=='green'
(case/'current-canonical.output.actual.json').write_bytes(new_bytes)
(case/'current-summary.output.actual.json').write_text(json.dumps(summary,indent=2)+'\n')
cli('current-candidate-fixture-finish',['finish','--id',wid,'--json'])
cli('current-candidate-fixture-archive',['archive','--id',wid,'--json'])
archive=root/f'.ai/work-items/archive/{wid}.archive.json';assert archive.is_file()
(case/'archive.output.actual.json').write_bytes(archive.read_bytes())
assert (case/'old-canonical.input.actual.json').read_bytes()==old_bytes
assert (root/'.ai/evidence/fixture-preserved-old-cycle.json').read_bytes()==old_bytes
report={'kind':'actual_isolated_four_check_retained_history_runtime_upgrade_acceptance','candidateSourceHead':subprocess.check_output(['git','rev-parse','HEAD'],cwd='/workspace/ai-cockpit-wi1089-recovery',text=True).strip(),'candidateBinarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'oldProducerRuntimeDigest':old['runtimeDigest'],'currentProducerRuntimeDigest':new_digest,'oldSnapshot':old['repositorySnapshotDigest'],'currentSnapshot':new['repositorySnapshotDigest'],'materialRequestDigest':request['requestDigest'],'oldCanonicalRawSha256':hashlib.sha256(old_bytes).hexdigest(),'oldBlockedOutcomeRawSha256':hashlib.sha256(old_outcome).hexdigest(),'currentCanonicalRawSha256':hashlib.sha256(new_bytes).hexdigest(),'currentNodesExecuted':4,'currentNodesReused':0,'currentProcessesSpawned':4,'maxConcurrentProcesses':1,'freshGreenPreflight':True,'actualFixtureFinish':True,'actualFixtureArchive':True,'limitations':'Independent fixture copied from actual c960 baseline. Old producer is preserved Linux c960 1.0.1; current producer is Linux development candidate. Same source snapshot; no fabricated runtime identity. Not the real rc2/macOS capsule, not Sentinel recovery, not real WI terminal authorization.'}
(case/'acceptance.actual.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
