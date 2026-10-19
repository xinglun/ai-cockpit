#!/usr/bin/env python3
"""Negative mutations of an actual native old receipt; never fabricated success evidence."""
from pathlib import Path
import subprocess,hashlib,json,shutil
base=Path(__file__).parent;source=base/'fixture-05-native-workspace-manifest';out=base/'manifest-removal.before-fix.actual';out.mkdir(exist_ok=False)
binary=Path('/workspace/scratch/wi1089-preserved-history-verify-target-20261019/debug/ai-cockpit');wid='WI-RETAINED-CANONICAL-FIXTURE';results=[]
for case in ['remove_manifest','wrong_command_with_manifest','wrong_command_then_remove_manifest']:
 directory=out/case;root=directory/'repository';shutil.copytree(source/'repository',root,ignore=shutil.ignore_patterns('fixture-target'))
 def call(label,args):
  r=subprocess.run([str(binary),*args,'--repo',str(root)],cwd=root,capture_output=True,timeout=60);(directory/(label+'.stdout.actual')).write_bytes(r.stdout);(directory/(label+'.stderr.actual')).write_bytes(r.stderr);return r
 status=call('baseline-status',['work-item','status','--id',wid,'--json']);s=json.loads(status.stdout);assert 'record_material_review_decision' in s['safeActions'],s
 p=root/f'.ai/evidence/{wid}.verification.json';e=json.loads(p.read_text());manifest=e['receipt']['planReceipt']['coverageManifest'];assert manifest['nodeIds']
 if case!='remove_manifest':
  node=manifest['nodeIds'][0];next(r for r in e['receipt']['executionRecords'] if r['nodeId']==node)['commandDigest']='sha256:'+hashlib.sha256(b'wrong command valid SHA').hexdigest()
 if case!='wrong_command_with_manifest':del e['receipt']['planReceipt']['coverageManifest']
 e['receiptDigest']='sha256:'+hashlib.sha256(json.dumps(e['receipt'],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest();p.write_text(json.dumps(e,indent=2))
 plan=call('material-plan',['work-item','material-review','plan','--id',wid]);request=json.loads(plan.stdout);inp=json.loads((source/'current-material-recovery.input.json').read_text());inp['requestDigest']=request['requestDigest'];ip=directory/'input.probe.json';ip.write_text(json.dumps(inp))
 record=call('material-record',['work-item','material-review','record','--id',wid,'--input',str(ip)]);result={'case':case,'nativeManifestPresentBeforeMutation':True,'initialPositiveAdmission':True,'exitCode':record.returncode,'admitted':record.returncode==0,'expectedAdmitted':False};results.append(result);print(json.dumps(result),flush=True)
report={'kind':'actual_negative_manifest_mutation_probe','sourceHead':'a2483ecc8c43f134f1a1d39661930110eba0033b','candidateBinarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'results':results,'isGreen':all(not x['admitted'] for x in results)};(out/'result.actual.json').write_text(json.dumps(report,indent=2)+'\n');raise SystemExit(0 if report['isGreen'] else 1)
