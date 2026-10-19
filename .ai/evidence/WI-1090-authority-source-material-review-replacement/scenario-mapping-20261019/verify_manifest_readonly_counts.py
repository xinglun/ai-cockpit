from pathlib import Path
import hashlib,json,os,stat
base=Path('/workspace/scratch/wi1090-m6-execution-20261019')
expected=Path('/workspace/scratch/wi1090-m6-readonly-plan-20261019/prospective-non-ai-source-manifest.proposal-only.json').read_bytes()
actual=(base/'evidence/actual-materialized-source-manifest.json').read_bytes()
assert expected==actual
manifest=json.loads(expected);source=base/'source'
found={str(p.relative_to(source)) for p in source.rglob('*') if p.is_file() or p.is_symlink()}
assert found==set(manifest)
for name,item in manifest.items():
 p=source/name;data=os.fsencode(os.readlink(p)) if p.is_symlink() else p.read_bytes()
 oid=hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()
 mode='120000' if p.is_symlink() else ('100755' if p.stat().st_mode&stat.S_IXUSR else '100644')
 assert oid==item['gitBlob'] and mode==item['mode']
print(json.dumps({'bothRawSha256':hashlib.sha256(actual).hexdigest(),'byteIdentical':True,'entries':len(manifest),'missing':0,'extra':0,'blobOrModeMismatch':0}))
