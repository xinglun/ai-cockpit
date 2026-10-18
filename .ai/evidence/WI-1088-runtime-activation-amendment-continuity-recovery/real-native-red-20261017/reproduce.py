"""Real native temporary-repository RED; never touches an original Work Item.
Every governance record is produced by Runtime. The isolated fixture review is
explicitly synthetic test input and is not Ray's decision or historical proof.
"""
from pathlib import Path
import base64, datetime, hashlib, json, os, subprocess, sys, time
RUNTIME=Path('/workspace/scratch/wi1087-rebuild-20261017/runtime-target/release/ai-cockpit')
ROOT=Path('/workspace/scratch/wi1088-runtime-continuity/real-native-red-20261017-attempt2')
WI='WI-TEST-ACTIVATION-GAP'
REPO=ROOT/'repo'
COMMANDS=ROOT/'commands'
ROOT.mkdir(exist_ok=False);REPO.mkdir();COMMANDS.mkdir()
ENV=os.environ.copy()
ENV.update({'PATH':'/workspace/.cloud-tools/cargo/bin:'+ENV['PATH'],'CARGO_HOME':'/workspace/.cloud-tools/cargo','RUSTUP_HOME':'/workspace/.cloud-tools/rustup','CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1','CARGO_TARGET_DIR':str(ROOT/'unused-isolated-target')})
for k in list(ENV):
    if k.startswith('GIT_TRACE') or k in ('GIT_CURL_VERBOSE','GH_DEBUG'):ENV.pop(k,None)
index=0

def run(label,args,native=False,expect=True):
    global index
    index+=1
    command=([str(RUNTIME),*args,'--repo',str(REPO)] if native else args)
    started=datetime.datetime.now(datetime.timezone.utc).isoformat();tick=time.monotonic()
    p=subprocess.run(command,cwd=REPO,env=ENV,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=60)
    out=COMMANDS/f'{index:02d}-{label}'
    out.with_suffix('.stdout').write_bytes(p.stdout);out.with_suffix('.stderr').write_bytes(p.stderr)
    record={'kind':'actual_native_fixture_observation' if native else 'actual_fixture_git_observation','label':label,'command':command,'cwd':str(REPO),'startedAt':started,'endedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'elapsedSeconds':round(time.monotonic()-tick,3),'exitCode':p.returncode,'runtimeRawSha256':hashlib.sha256(RUNTIME.read_bytes()).hexdigest() if native else None}
    for name,data in [('stdout',p.stdout),('stderr',p.stderr)]:record[name]={'bytes':len(data),'rawSha256':hashlib.sha256(data).hexdigest(),'encoding':'base64','data':base64.b64encode(data).decode()}
    out.with_suffix('.json').write_text(json.dumps(record,indent=2)+'\n')
    print(json.dumps({'step':index,'label':label,'exitCode':p.returncode}),flush=True)
    if expect and p.returncode:raise AssertionError(f'Setup failed at {label}: {p.stderr.decode(errors="replace")[:2500]}')
    return p

def native(label,args,expect=True):return run(label,args,True,expect)
def read(path):return json.loads((REPO/path).read_text())
def write(name,value):
    p=ROOT/name;p.write_text(json.dumps(value,indent=2)+'\n');return str(p)
def digest(value):return 'sha256:'+hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
def amendment(change_id,changes):
    c=read(f'.ai/work-items/active/{WI}.contract.json')
    req={'schemaVersion':1,'changeId':change_id,'expectedContractDigest':digest(c),'reason':'Isolated WI-1088 regression fixture only; no real Work Item or human authority.','changes':[{'path':'/'+k,'operation':'replace' if isinstance(v,(list,dict)) else 'set','value':v} for k,v in changes.items()]}
    return write(change_id+'.json',req)

run('init',['git','init','-q','--initial-branch=main'])
run('git-email',['git','config','user.email','fixture@example.invalid'])
run('git-name',['git','config','user.name','WI1088 fixture'])
(REPO/'README.md').write_text('isolated activation continuity fixture\n')
run('git-add',['git','add','README.md']);run('git-initial',['git','commit','-qm','initial fixture'])
run('bare-origin',['git','init','-q','--bare','--initial-branch=main',str(ROOT/'origin.git')])
run('remote',['git','remote','add','origin',str(ROOT/'origin.git')])
run('push-fixture-main',['git','push','-q','-u','origin','main']);run('default-head',['git','remote','set-head','origin','-a'])
run('fixture-branch',['git','checkout','-q','-b','fixture/activation-gap'])
native('attach',['attach'])
native('new-not-ready',['work-item','new','--id',WI,'--mode','implementation'])
intent='Exercise the native activation continuity boundary in an isolated fixture'
goal='Keep native journal transitions reviewable across activation'
acceptance='A post-checkpoint amendment must remain inspectable without a discontinuous Contract chain'
verification='git diff --check'
pre=amendment('before-activation',{'intent':intent,'goal':goal,'scope':['README.md'],'outOfScope':['No external providers grants or real Work Items'],'risk':'normal','authority':'authorized','acceptanceCriteria':[acceptance],'verification':[verification]})
native('amend-before-activation',['work-item','amend','--id',WI,'--request',pre])
before=read(f'.ai/work-items/active/{WI}.contract.json')
start=native('activate-prepare',['start','--id',WI,'--intent',intent,'--goal',goal,'--scope','README.md','--out-of-scope','No external providers grants or real Work Items','--risk','normal','--authority','authorized','--acceptance',acceptance,'--verification',verification,'--prepare'])
after=read(f'.ai/work-items/active/{WI}.contract.json');summary=read(f'.ai/work-items/active/{WI}.summary.json')
assert before['state']=='not_ready' and after['state']=='implementation_active'
review={'decisionEvidence':{'schemaVersion':1,'decisionId':'contract-preflight-review','decision':'confirm_review','workItemId':WI,'repositoryId':after['repositoryId'],'contractDigest':digest(after),'preflightDecisionDigest':summary['preflightDecisionDigest'],'repositorySnapshotDigest':summary['preflightRepositorySnapshotDigest'],'recordedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'recordedBy':'fixture:WI1088-isolated-test-only','reason':'Synthetic fixture input only, not a real human response or WI-1087 historical proof.'}}
native('record-fixture-review',['work-item','controls','--id',WI,'--input',write('fixture-review.json',review)])
pf=native('reviewed-preflight',['preflight','--contract',str(REPO/f'.ai/work-items/active/{WI}.contract.json')]);pf=json.loads(pf.stdout);assert pf['state']=='green',pf
native('fixture-checkpoint',['checkpoint','--id',WI])
post=amendment('after-checkpoint',{'goal':'Keep the post-checkpoint amended plan natively inspectable'})
failed=native('amend-after-checkpoint',['work-item','amend','--id',WI,'--request',post],False)
status=native('status-after-post-amend',['work-item','status','--id',WI,'--json'],False)
history=native('history-after-post-amend',['work-item','amendments','--id',WI])
receipts=json.loads(history.stdout)
files=sorted((REPO/f'.ai/evidence/{WI}.contract-amendments').glob('*.json'))
prepared=[p for p in files if p.name.endswith('.prepared.json')];committed=[p for p in files if p.name.endswith('.committed.json')]
error='amendment Contract transition chain is discontinuous'
result={'kind':'actual_pre_fix_native_temporary_repository_RED_not_product_verification','runtimeRawSha256':hashlib.sha256(RUNTIME.read_bytes()).hexdigest(),'runtimeSourceRevision':'a12ee2ff4413ac261560339c9ed7af59dde0f701','repository':str(REPO),'fixtureWorkItemId':WI,'activationBeforeContractDigest':digest(before),'activationAfterContractDigest':digest(after),'postAmendExitCode':failed.returncode,'statusExitCode':status.returncode,'postAmendContainsExpectedContinuityError':error in failed.stderr.decode(),'statusContainsExpectedContinuityError':error in status.stderr.decode(),'preparedCount':len(prepared),'committedCount':len(committed),'nativeHistoryCount':len(receipts),'checkpointCount':read(f'.ai/work-items/active/{WI}.summary.json')['checkpointCount'],'journalLinkIntact':len(receipts)==2 and receipts[1]['previousJournalDigest']==receipts[0]['journalDigest'],'contractBoundaryDiscontinuous':len(receipts)==2 and receipts[1]['previousContractDigest']!=receipts[0]['newContractDigest'],'historyRawHashes':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in files},'testAuthority':'Synthetic isolated fixture only; no real human review or historical transition authentication claimed.'}
(ROOT/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result),flush=True)
assert len(prepared)==2 and len(committed)==2 and result['journalLinkIntact'] and result['contractBoundaryDiscontinuous']
assert result['postAmendContainsExpectedContinuityError'] and result['statusContainsExpectedContinuityError']
assert failed.returncode==0,'RED: a real native post-checkpoint amendment committed both records, then its admission failed on the activation Contract boundary'
