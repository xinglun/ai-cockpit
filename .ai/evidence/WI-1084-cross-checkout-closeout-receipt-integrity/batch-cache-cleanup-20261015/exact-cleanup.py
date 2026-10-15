import os, pathlib, json, hashlib, stat, datetime, subprocess

R = pathlib.Path('/workspace/ai-cockpit-wi1084-closeout-receipt-integrity')
D = R / '.ai/evidence/WI-1084-cross-checkout-closeout-receipt-integrity'
P = D / 'batch-cache-proposal-20261015'
O = D / 'batch-cache-cleanup-20261015'
EXPECTED = '8e12a74c56ea839341532475e0fca0eeb77000c0f65552d482e9ddc9d336e539'

def now(): return datetime.datetime.now(datetime.timezone.utc).isoformat()
def save(name, x): (O/name).write_text(json.dumps(x, ensure_ascii=False, indent=2)+'\n')
def sha(p):
    h=hashlib.sha256()
    with open(p,'rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
    return h.hexdigest()
def processes():
    active=[]; zombies=0
    for p in pathlib.Path('/proc').glob('[0-9]*'):
        try:
            s=(p/'stat').read_text(); tail=s[s.rfind(')')+2:].split(); name=(p/'comm').read_text().strip()
            if tail[0]=='Z': zombies+=1
            elif name in ('cargo','rustc','ai-cockpit','clippy-driver') or name.startswith('cargo-'):
                active.append({'pid':int(p.name),'name':name})
        except (FileNotFoundError,ProcessLookupError,PermissionError): pass
    return {'activeBuildProcesses':active,'zombies':zombies,'pid1':pathlib.Path('/proc/1/comm').read_text().strip()}
def disk():
    return {p:{'device':os.stat(p).st_dev,'availableBytes':os.statvfs(p).f_bavail*os.statvfs(p).f_frsize,'freeInodes':os.statvfs(p).f_favail} for p in ('/workspace','/tmp')}
def ident(s): return {'device':s.st_dev,'inode':s.st_ino,'mode':oct(s.st_mode),'sizeBytes':s.st_size,'linkCount':s.st_nlink,'mtimeNs':s.st_mtime_ns}
def check(e):
    p=pathlib.Path(e['path']); s=p.lstat()
    assert stat.S_ISREG(s.st_mode) and not s.st_mode&0o111, str(p)
    for k,v in [('device',s.st_dev),('inode',s.st_ino),('sizeBytes',s.st_size),('linkCount',s.st_nlink),('mode',oct(s.st_mode))]: assert e[k]==v,(str(p),k,e[k],v)
    assert datetime.datetime.fromtimestamp(s.st_mtime,datetime.timezone.utc).isoformat()==e['modifiedAtUTC'],str(p)
    for q in p.parents: assert stat.S_ISDIR(q.lstat().st_mode),str(q)
    return s

raw=(P/'candidate-files.manifest.json').read_bytes()
assert hashlib.sha256(raw).hexdigest()==EXPECTED
entries=json.loads(raw)['entries']; proposal=json.loads((P/'proposal.json').read_bytes())
assert len(entries)==2112 and len({e['path'] for e in entries})==2112
assert not O.exists(), 'Do not overwrite an existing cleanup execution'
allowed=[pathlib.Path(g) for g in (
 '/workspace/.home/.cache/ai-cockpit-verify-target/ec9dc514276685dc3ff47011e8497b3987b3c90466848455758df44466684773/debug/build',
 '/workspace/.home/.cache/ai-cockpit-verify-target/ec9dc514276685dc3ff47011e8497b3987b3c90466848455758df44466684773/debug/deps',
 '/workspace/.tmp-wi1083-target-00461714-20261015/debug/build',
 '/workspace/.tmp-wi1083-target-00461714-20261015/debug/deps')]
for e in entries:
    assert any(pathlib.Path(e['path']).is_relative_to(p) for p in allowed)
    check(e)
health=processes(); assert not health['activeBuildProcesses'], health
protected=[]; hashes={}
for x in proposal['protectedRuntimeFiles']:
    p=pathlib.Path(x['path']); s=p.lstat(); key=(s.st_dev,s.st_ino)
    assert s.st_dev==x['device'] and s.st_ino==x['inode'] and s.st_size==x['sizeBytes']
    if key not in hashes: hashes[key]=sha(p)
    assert hashes[key]==x['sha256'],str(p)
    protected.append({'path':str(p),**ident(s),'sha256':hashes[key]})
selected={e['path'] for e in entries}
excluded=[]
for root in allowed:
    for p in root.rglob('*'):
        if str(p) in selected: continue
        s=p.lstat()
        if not stat.S_ISDIR(s.st_mode): excluded.append({'path':str(p),**ident(s)})
O.mkdir()
save('authority.json',{'recordedAtUTC':now(),'source':'direct current user message','approvalText':'允许按已提交清单清理 2,112 个旧缓存文件（清单 SHA-256：8e12a74c56ea839341532475e0fca0eeb77000c0f65552d482e9ddc9d336e539），保持全部排除项，清理后核验空间，再继续 WI-1084 的修复和完整验证。','manifestSha256':EXPECTED,'scope':'Only the 2112 listed stable regular non-executable cache files; retain every exclusion and containing directory.'})
save('excluded-before.json',excluded); save('protected-runtimes-before.json',protected)
before=disk(); save('before.json',{'recordedAtUTC':now(),'disk':before,'health':health,'verifiedCandidateCount':len(entries)})
for args,name in [(['df','-B1','/workspace','/tmp'],'df-before.txt'),(['df','-i','/workspace','/tmp'],'df-inodes-before.txt')]: (O/name).write_bytes(subprocess.check_output(args))
assert not processes()['activeBuildProcesses']
removed=[]
try:
    with (O/'removed.jsonl').open('x') as log:
        for e in entries:
            p=pathlib.Path(e['path']); check(e)
            fd=os.open(p.parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
            try:
                s=os.stat(p.name,dir_fd=fd,follow_symlinks=False)
                assert ident(s)==ident(p.lstat()) and s.st_ino==e['inode'] and s.st_dev==e['device']
                os.unlink(p.name,dir_fd=fd)
            finally: os.close(fd)
            removed.append(e['path']); log.write(json.dumps({'path':e['path'],'inode':e['inode'],'device':e['device']})+'\n'); log.flush()
except BaseException as ex:
    save('interrupted.json',{'recordedAtUTC':now(),'removedCount':len(removed),'error':repr(ex)})
    raise
assert all(not os.path.lexists(e['path']) for e in entries)
for x in excluded: assert ident(pathlib.Path(x['path']).lstat())=={k:v for k,v in x.items() if k!='path'}, x['path']
hashes={}; after_protected=[]
for x in protected:
    p=pathlib.Path(x['path']); s=p.lstat(); assert ident(s)=={k:v for k,v in x.items() if k not in ('path','sha256')}
    key=(s.st_dev,s.st_ino)
    if key not in hashes: hashes[key]=sha(p)
    assert hashes[key]==x['sha256'],str(p)
    after_protected.append({**x,'afterSha256':hashes[key]})
save('protected-runtimes-after.json',after_protected)
after=disk()
for args,name in [(['df','-B1','/workspace','/tmp'],'df-after.txt'),(['df','-i','/workspace','/tmp'],'df-inodes-after.txt')]: (O/name).write_bytes(subprocess.check_output(args))
result={'recordedAtUTC':now(),'manifestSha256':EXPECTED,'deletedFileCount':len(removed),'deletedDirectoryCount':0,'allCandidatesAbsent':True,'allExcludedEntriesUnchanged':True,'excludedEntryCount':len(excluded),'protectedRuntimePathsUnchanged':len(protected),'diskBefore':before,'diskAfter':after,'workspaceAvailableByteIncrease':after['/workspace']['availableBytes']-before['/workspace']['availableBytes'],'workspaceFreeInodeIncrease':after['/workspace']['freeInodes']-before['/workspace']['freeInodes'],'healthAfter':processes()}
save('result.json',result)
print(json.dumps(result,indent=2))
