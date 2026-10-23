"""Native synthetic M6 fixture. No real consumer authority or Runtime adoption."""
from pathlib import Path
from datetime import datetime, timezone
import copy
import hashlib
import json
import os
import signal
import subprocess
import sys
import time

base = Path('/workspace/scratch/wi1090-m6-execution-20261019')
owner = Path('/workspace/ai-cockpit-wi1090-material-replacement')
gov = Path('/workspace/scratch/wi1087-fixed-runtime-b8ec269-20261018/target/release/ai-cockpit')
owner_item = 'WI-1090-authority-source-material-review-replacement'
checks = ['cargo test --locked --offline --test material', 'cargo fmt --all -- --check',
          'cargo test --locked --offline', 'cargo clippy --locked --offline --all-targets -- -D warnings']

def sha(data):
    return hashlib.sha256(data).hexdigest()

def canonical(value):
    return 'sha256:' + sha(json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode())

class Fixture:
    def __init__(self, case):
        assert case in ['leaf', 'replace']
        identity = json.loads((base / 'evidence/candidate-binary.actual.agent-recorded.json').read_text())
        self.binary = Path(identity['path'])
        self.binary_digest = 'sha256:' + sha(self.binary.read_bytes())
        assert self.binary_digest == 'sha256:' + identity['rawSha256']
        self.case = case
        self.directory = base / 'fixtures' / case
        self.directory.mkdir(parents=True, exist_ok=False)
        self.root = self.directory / 'repository'; self.root.mkdir()
        self.logs = base / 'evidence' / case; self.logs.mkdir()
        self.id = 'WI-M6-' + case.upper() + '-SYNTHETIC-FIXTURE'
        self.sequence = 0; self.deadline = time.monotonic() + 850
        self.env = os.environ.copy()
        self.env.update(PATH='/workspace/.cloud-tools/cargo/bin:' + self.env['PATH'],
            CARGO_HOME='/workspace/.cloud-tools/cargo', RUSTUP_HOME='/workspace/.cloud-tools/rustup',
            CARGO_TARGET_DIR=str(base / 'fixture-targets' / case), CARGO_BUILD_JOBS='1',
            CARGO_INCREMENTAL='0', CARGO_NET_OFFLINE='true', RUST_TEST_THREADS='1',
            CARGO_PROFILE_DEV_DEBUG='0', CARGO_PROFILE_TEST_DEBUG='0')
        self.old_source = '.ai/evidence/m6-synthetic-consent/old.agent-recorded.json'
        self.new_source = '.ai/evidence/m6-synthetic-consent/current.agent-recorded.json'
        self.state = {'case': case, 'workItemId': self.id, 'candidateRuntimeDigest': self.binary_digest,
                      'completedStages': [], 'currentStage': 'setup', 'state': 'running'}
        self.save_state()

    def save_state(self):
        (self.directory / 'progress-state.agent-recorded.json').write_text(json.dumps(self.state, indent=2) + '\n')

    def stage(self, name):
        self.state['currentStage'] = name; self.save_state()
        print(self.case + ': stage ' + name, flush=True)

    def done(self):
        self.state['completedStages'].append(self.state['currentStage']); self.save_state()

    def immutable(self, path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        if path.exists():
            assert path.read_bytes() == data, path
        else:
            with path.open('xb') as stream: stream.write(data)

    def data(self, name, value):
        path = self.logs / (name + '.input.json')
        self.immutable(path, (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode())
        return path

    def run(self, label, argv, expect=0, cwd=None):
        self.sequence += 1
        prefix = self.logs / f'{self.sequence:03}-{label}'
        arguments = [str(x) for x in argv]
        meta = {'argv': arguments, 'startedAt': datetime.now(timezone.utc).isoformat(), 'terminal': 'running'}
        metadata = Path(str(prefix) + '.metadata.actual.json')
        assert not metadata.exists()
        started = time.monotonic()
        with Path(str(prefix) + '.stdout.actual').open('xb') as stdout, Path(str(prefix) + '.stderr.actual').open('xb') as stderr:
            process = subprocess.Popen(arguments, cwd=cwd or self.root, env=self.env, stdout=stdout,
                                       stderr=stderr, start_new_session=True)
            meta.update(pid=process.pid, processGroup=process.pid)
            metadata.write_text(json.dumps(meta, indent=2) + '\n')
            try:
                code = process.wait(timeout=max(1, min(600, self.deadline - time.monotonic())))
                meta.update(exitCode=code, terminal='returned')
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGTERM)
                try: process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(process.pid, signal.SIGKILL); process.wait(timeout=5)
                meta.update(exitCode=process.returncode, terminal='timeout_children_state_not_inferred')
        stdout = Path(str(prefix) + '.stdout.actual').read_bytes()
        stderr = Path(str(prefix) + '.stderr.actual').read_bytes()
        meta.update(endedAt=datetime.now(timezone.utc).isoformat(), elapsedSeconds=time.monotonic() - started,
                    stdoutSha256=sha(stdout), stderrSha256=sha(stderr))
        metadata.write_text(json.dumps(meta, indent=2) + '\n')
        print(self.case + f' {self.sequence:03} {label}: exit ' + str(meta['exitCode']), flush=True)
        assert meta['terminal'] == 'returned', meta
        if expect == 'nonzero':
            assert meta['exitCode'] != 0, label
        elif expect is not None:
            assert meta['exitCode'] == expect, (label, stderr.decode(errors='replace')[-2500:], stdout.decode(errors='replace')[-2500:])
        return stdout

    def cli(self, label, args, expect=0):
        return self.run(label, [self.binary, *args, '--repo', self.root], expect)

    def git(self, label, *args):
        return self.run(label, ['git', *args])

    def document(self, suffix, archived=False):
        path = self.root / (f'.ai/work-items/{"archive" if archived else "active"}/{self.id}.{suffix}.json')
        return json.loads(path.read_bytes())

    def status(self, label):
        return json.loads(self.cli(label, ['work-item', 'status', '--id', self.id, '--json']))

    def controls(self, label, values):
        path = self.data(label, values)
        return json.loads(self.cli(label, ['work-item', 'controls', '--id', self.id, '--input', path]))

    def review(self, label):
        summary = self.document('summary')
        return self.controls(label, {'decisionEvidence': {
            'schemaVersion': 1, 'decisionId': 'contract-preflight-review', 'decision': 'confirm_review',
            'workItemId': self.id, 'repositoryId': self.document('contract')['repositoryId'],
            'contractDigest': summary['preflightContractDigest'],
            'preflightDecisionDigest': summary['preflightDecisionDigest'],
            'repositorySnapshotDigest': summary['preflightRepositorySnapshotDigest'],
            'recordedAt': datetime.now(timezone.utc).isoformat(timespec='microseconds').replace('+00:00', 'Z'),
            'recordedBy': 'human:isolated-m6-fixture',
            'reason': 'Synthetic self-declared fixture review for this exact binding; no real human identity or consumer authority.'}})

    def preflight(self, label):
        return json.loads(self.cli(label, ['preflight', '--contract', f'.ai/work-items/active/{self.id}.contract.json']))

    def amend(self, label, changes):
        path = self.data(label, {'schemaVersion': 1, 'changeId': label,
            'expectedContractDigest': canonical(self.document('contract')),
            'reason': 'Synthetic M6 native fixture only; no real consumer decision or authority.', 'changes': changes})
        return json.loads(self.cli(label, ['work-item', 'amend', '--id', self.id, '--request', path]))

    def plan_material(self, label):
        request = json.loads(self.cli(label, ['work-item', 'material-review', 'plan', '--id', self.id]))
        assert not request['findingCodes'] and request['reviewEnabled'], request
        assert any(x.get('reviewable') for x in request['entries']), request
        return request

    def record_material(self, label, request, authority):
        consent = {'kind': 'synthetic_fixture_material_risk_decision_not_real_human_approval',
            'actor': 'agent:isolated-m6-fixture', 'assurance': 'self_declared',
            'request': request, 'decision': 'accept_permitted_unknowns',
            'limits': 'Only this actual current synthetic fixture request; no Ray or real consumer risk acceptance.'}
        self.immutable(self.root / authority, (json.dumps(consent, ensure_ascii=False, indent=2) + '\n').encode())
        value = {'schemaVersion': 1, 'decision': 'accept_permitted_unknowns',
            'requestDigest': request['requestDigest'], 'reviewerActor': 'agent:isolated-m6-fixture',
            'authoritySource': authority, 'assurance': 'self_declared',
            'evidenceRefs': [{'path': authority, 'digest': 'sha256:' + sha((self.root / authority).read_bytes())}],
            'rationale': 'Reviewed the exact synthetic fixture members and bounded readable syntax Unknown.',
            'residualRisk': 'Scanner incompleteness accepted for this synthetic fixture only; self_declared does not authenticate humans.'}
        path = self.data(label, value)
        result = json.loads(self.cli(label, ['work-item', 'material-review', 'record', '--id', self.id, '--input', path]))
        return result, path

    def verify_metrics(self, evidence, label):
        assert evidence['passed'] and evidence['runtimeDigest'] == self.binary_digest, label
        receipt = evidence['receipt']
        for key, expected in [('nodesExecuted', 4), ('nodesReused', 0), ('processesSpawned', 4), ('maxConcurrentProcesses', 1)]:
            assert receipt[key] == expected, (label, key, receipt.get(key))
        records = receipt['executionRecords']; assert len(records) == 4
        assert all(x['spawned'] and x['passed'] and x['exitCode'] == 0 and not x['timedOut'] for x in records), label
        return receipt

    def execute(self):
        self.stage('initial-native-R')
        self.git('git-init', 'init', '-q', '--initial-branch=main')
        self.git('git-name', 'config', 'user.name', 'Isolated M6 Synthetic Fixture')
        self.git('git-email', 'config', 'user.email', 'm6-fixture@example.invalid')
        (self.root / 'Cargo.toml').write_text('[package]\nname = "retained_fixture"\nversion = "0.1.0"\nedition = "2024"\n')
        (self.root / 'src').mkdir()
        (self.root / 'src/lib.rs').write_text('pub fn expected_date() -> &\'static str {\n    "2026-10-14"\n}\n')
        (self.root / 'README.md').write_text('Synthetic isolated M6 lifecycle fixture.\n')
        self.run('generate-actual-lockfile', ['cargo', 'generate-lockfile', '--offline'])
        self.git('base-add', 'add', '-A'); self.git('base-commit', 'commit', '-qm', 'Synthetic fixture source baseline')
        self.cli('attach', ['attach'])
        inspection = json.loads(self.cli('candidate-inspect', ['inspect']))
        assert inspection['runtimeDigest'] == self.binary_digest
        self.immutable(self.logs / 'candidate-inspect.identity.actual.json', (json.dumps(inspection, indent=2) + '\n').encode())
        self.git('attach-add', 'add', '-A'); self.git('attach-commit', 'commit', '-qm', 'Attach synthetic repository before work item start')
        args = ['start', '--id', self.id, '--intent', 'Synthetic combined M6 retained-history replacement',
            '--goal', 'Run old and new four-node cycles through actual synthetic fixture archive',
            '--risk', 'normal', '--authority', 'authorized', '--out-of-scope', 'No real consumer or human authorization',
            '--acceptance', 'Preserve native historical evidence and independently decide current synthetic material',
            '--required-evidence', 'verification_receipt']
        for path in ['README.md', 'Cargo.toml', 'Cargo.lock', 'src/lib.rs', 'tests/material.rs']: args += ['--scope', path]
        for command in checks: args += ['--verification', command]
        self.cli('start', args)
        old_scenario = {'scenario': 'old fixture four-check cycle', 'required': True, 'status': 'unverified',
            'evidence': [], 'expected': 'All four old checks execute and immutable original evidence is retained.',
            'verificationPlan': 'Native verify of the actual four fixture Cargo commands.'}
        self.amend('initial-profile-R', [
            {'path': '/governanceProfile', 'operation': 'set', 'value': {'materialInspectionReview': {
                'schemaVersion': 1, 'permittedUnknown': 'repository_material_inspection_unavailable',
                'permittedCause': 'readable_committed_rust_syntax_unknown', 'assurance': 'self_declared',
                'reviewerActor': 'agent:isolated-m6-fixture', 'authoritySource': self.old_source, 'acceptResidualRisk': True}}},
            {'path': '/scenarioCoverage', 'operation': 'add', 'value': old_scenario}])
        self.preflight('R-initial-preflight'); self.review('R-initial-synthetic-review')
        self.controls('R-planned-scenario', {'scenarioCoverage': [old_scenario]})
        self.cli('checkpoint', ['checkpoint', '--id', self.id])
        (self.root / 'tests').mkdir()
        (self.root / 'tests/material.rs').write_text('fn material() -> String {\n    let mut label = String::from("token");\n    label.push_str("ization");\n    label\n}\n\n#[test]\nfn composed_label_matches() {\n    assert_eq!(material(), "tokenization");\n    assert_eq!(retained_fixture::expected_date(), "2026-10-14");\n}\n')
        self.git('material-add', 'add', '-A'); self.git('material-commit', 'commit', '-qm', 'Commit exact readable synthetic Rust material')
        self.preflight('R-current-source-preflight'); self.review('R-current-source-synthetic-review')
        old_request = self.plan_material('R-old-material-plan')
        self.record_material('R-old-material-record', old_request, self.old_source)
        self.done()
        self.stage('old-four-checks-and-actual-blocked-outcome')
        self.cli('R-validate', ['work-item', 'validate', '--id', self.id, '--json'])
        self.cli('R-old-four-execution', ['verify', '--work-item', self.id, '--workers', '1'])
        canonical_path = self.root / f'.ai/evidence/{self.id}.verification.json'
        old_bytes = canonical_path.read_bytes(); old_envelope = json.loads(old_bytes)
        self.verify_metrics(old_envelope, 'old cycle')
        self.cli('actual-blocked-finish', ['finish', '--id', self.id, '--json'], 'nonzero')
        outcome_path = self.root / f'.ai/work-items/active/{self.id}.outcome.json'
        old_outcome = outcome_path.read_bytes(); assert json.loads(old_outcome)['state'] == 'blocked'
        old_summary = self.document('summary'); old_pointer = old_summary['materialReviewReceipt']
        old_receipt_path = self.root / old_pointer['path']; old_receipt_bytes = old_receipt_path.read_bytes()
        old_history = old_summary.get('materialReviewReceiptHistory', [])
        retained = {}; immutable_live = {}
        paths = [canonical_path, outcome_path, self.root / f'.ai/work-items/active/{self.id}.contract.json',
                 self.root / f'.ai/work-items/active/{self.id}.summary.json', old_receipt_path]
        paths += list((self.root / '.ai/evidence').glob(self.id + '.verification-attempt.*.json'))
        paths += list((self.root / '.ai/evidence' / (self.id + '.contract-amendments')).glob('*.json'))
        paths += list((self.root / '.ai/decisions').glob(self.id + '.preflight-review*.json'))
        for path in paths:
            relative = str(path.relative_to(self.root)); data = path.read_bytes()
            self.immutable(self.logs / 'before' / relative, data); retained[relative] = sha(data)
            if '.verification-attempt.' in path.name or '.contract-amendments/' in relative or '.preflight-review' in path.name or path == old_receipt_path:
                immutable_live[relative] = sha(data)
        self.immutable(self.logs / 'before/raw-manifest.agent-recorded.json', (json.dumps(retained, indent=2) + '\n').encode())
        self.immutable(self.root / '.ai/evidence/m6-preserved-old-canonical.json', old_bytes)
        self.immutable(self.root / '.ai/evidence/m6-preserved-blocked-outcome.json', old_outcome)
        old_scenario['status'] = 'verified'; old_scenario['evidence'] = ['.ai/evidence/m6-preserved-old-canonical.json']
        self.controls('old-cycle-actual-evidence', {'scenarioCoverage': [old_scenario]})
        self.done()
        self.stage('native-20-operation-mixed-history-A')
        mixed = []
        for index in range(2): mixed.append({'path': '/scope', 'operation': 'add', 'value': f'docs/synthetic-history-{index}.md'})
        for index in range(8): mixed.append({'path': '/acceptanceCriteria', 'operation': 'add', 'value': f'Synthetic reviewed historical requirement {index}'})
        scenarios = []
        for index in range(3):
            scenario = {'scenario': f'synthetic mixed historical proof {index}', 'required': True, 'status': 'unverified',
                'evidence': [], 'expected': 'The old receipt stays historical and a new exact decision preserves the old pointer once.',
                'verificationPlan': 'Inspect actual native H/P/old invalid projection and successful independent current material receipt.'}
            scenarios.append(scenario); mixed.append({'path': '/scenarioCoverage', 'operation': 'add', 'value': scenario})
        for index in range(7): mixed.append({'path': '/sources', 'operation': 'add', 'value': {'path': f'.ai/evidence/synthetic-history-source-{index}', 'reason': 'Synthetic native historical context, no real consumer authority.'}})
        contract_R = self.document('contract'); self.amend('reviewed-mixed-history-A', mixed)
        contract_A = self.document('contract'); assert contract_R['governanceProfile'] == contract_A['governanceProfile']
        self.preflight('A-current-preflight'); self.review('A-required-synthetic-review')
        self.done()
        self.stage('independently-changed-current-source-and-adjacent-B')
        (self.root / 'README.md').write_text('A real committed synthetic source change after reviewed A and before adjacent B.\n')
        self.git('current-source-add', 'add', 'README.md'); self.git('current-source-commit', 'commit', '-qm', 'Current fixture source before adjacent source binding')
        if self.case == 'leaf':
            changes = [{'path': '/governanceProfile/materialInspectionReview/authoritySource', 'operation': 'set', 'value': self.new_source}]
        else:
            profile = copy.deepcopy(contract_A['governanceProfile']); profile['materialInspectionReview']['authoritySource'] = self.new_source
            changes = [{'path': '/governanceProfile', 'operation': 'replace', 'value': profile}]
        self.amend('adjacent-source-only-B', changes)
        contract_B = self.document('contract'); expected_B = copy.deepcopy(contract_A)
        expected_B['governanceProfile']['materialInspectionReview']['authoritySource'] = self.new_source
        assert expected_B == contract_B, 'B changes only authoritySource'
        history = json.loads(self.cli('actual-native-history', ['work-item', 'amendments', '--id', self.id]))
        assert len(history) == 3 and len(history[1]['changedValues']) == 20 and len(history[2]['changedValues']) == 1
        self.preflight('B-current-preflight'); self.review('B-current-synthetic-review')
        self.done()
        self.stage('old-invalid-no-discharge-and-independent-new-material')
        status = self.status('B-before-material-status')
        assert status['reviewReceiptDigest'] is None
        assert 'material_review_receipt_invalid' in status['effectiveUnknowns']
        assert 'repository_material_inspection_unavailable' in status['effectiveUnknowns']
        assert 'record_material_review_decision' in status['safeActions'], status
        assert canonical_path.read_bytes() == old_bytes and outcome_path.read_bytes() == old_outcome
        current_request = self.plan_material('B-current-exact-material-plan')
        assert current_request['sourceSnapshotDigest'] != old_request['sourceSnapshotDigest']
        self.record_material('B-independent-current-material-record', current_request, self.new_source)
        summary_path = self.root / f'.ai/work-items/active/{self.id}.summary.json'
        after_bytes = summary_path.read_bytes(); after = json.loads(after_bytes)
        assert after['materialReviewReceiptHistory'] == old_history + [old_pointer]
        assert after['materialReviewReceipt']['requestDigest'] == current_request['requestDigest']
        assert old_receipt_path.read_bytes() == old_receipt_bytes
        new_receipt = json.loads((self.root / after['materialReviewReceipt']['path']).read_bytes())
        assert new_receipt['requestDigest'] == current_request['requestDigest'] and new_receipt['authoritySource'] == self.new_source
        status = self.status('B-after-material-status')
        assert status['reviewReceiptDigest'] == new_receipt['receiptDigest']
        assert 'repository_material_inspection_unavailable' not in status['effectiveUnknowns']
        replay = self.logs / 'B-independent-current-material-record.input.json'
        self.cli('committed-material-replay-rejection', ['work-item', 'material-review', 'record', '--id', self.id, '--input', replay], 'nonzero')
        assert summary_path.read_bytes() == after_bytes
        assert canonical_path.read_bytes() == old_bytes and outcome_path.read_bytes() == old_outcome
        observation = {'kind': 'actual_synthetic_fixture_observations_not_human_authentication',
            'oldInvalidBeforeNewDecision': True, 'oldReceiptNoDischargeBeforeNewDecision': True,
            'actualNativeMixedOperationCount': 20, 'adjacentAuthoritySourceOnly': True,
            'actualNewRequestDigest': current_request['requestDigest'], 'actualNewReceiptDigest': new_receipt['receiptDigest'],
            'oldPointerHistoryExactlyOnce': True, 'committedReplayRejected': True}
        self.immutable(self.root / '.ai/evidence/m6-current-observations.agent-recorded.json', (json.dumps(observation, indent=2) + '\n').encode())
        for scenario in scenarios:
            scenario['status'] = 'verified'; scenario['evidence'] = ['.ai/evidence/m6-current-observations.agent-recorded.json']
        self.controls('actual-current-history-scenario-evidence', {'scenarioCoverage': [old_scenario, *scenarios]})
        self.done()
        self.stage('four-new-fresh-executions')
        self.cli('B-validate', ['work-item', 'validate', '--id', self.id, '--json'])
        plan = json.loads(self.cli('B-four-node-plan-only', ['verify', '--work-item', self.id, '--workers', '1', '--plan-only']))
        assert plan['processesSpawned'] == 0 and plan['nodesPlanned'] == 4 and plan['nodesToExecute'] == 4 and plan['nodesReused'] == 0
        assert canonical_path.read_bytes() == old_bytes
        self.cli('B-four-actual-executions', ['verify', '--work-item', self.id, '--workers', '1'])
        new_bytes = canonical_path.read_bytes(); envelope = json.loads(new_bytes)
        self.verify_metrics(envelope, 'current cycle')
        summary = self.document('summary')
        assert summary['preflightState'] == 'green'
        assert summary['preflightContractDigest'] == envelope['contractDigest']
        assert summary['preflightRepositorySnapshotDigest'] == envelope['repositorySnapshotDigest']
        status = self.status('B-after-verification-status')
        assert status['verification'] == 'verified' and status['evidenceFreshness']['state'] == 'fresh'
        self.done()
        self.stage('actual-fixture-finish-archive-and-historical-byte-audit')
        self.cli('actual-current-finish', ['finish', '--id', self.id, '--json'])
        self.cli('actual-current-archive', ['archive', '--id', self.id, '--json'])
        archive = self.root / f'.ai/work-items/archive/{self.id}.archive.json'; assert archive.is_file()
        archive_status = self.status('actual-archive-status'); assert archive_status['lifecyclePhase'] == 'archived'
        for relative, digest in retained.items(): assert sha((self.logs / 'before' / relative).read_bytes()) == digest, relative
        for relative, digest in immutable_live.items(): assert sha((self.root / relative).read_bytes()) == digest, relative
        assert (self.root / '.ai/evidence/m6-preserved-old-canonical.json').read_bytes() == old_bytes
        assert (self.root / '.ai/evidence/m6-preserved-blocked-outcome.json').read_bytes() == old_outcome
        assert old_receipt_path.read_bytes() == old_receipt_bytes
        self.cli('actual-archived-human-outcome', ['work-item', 'outcome', '--id', self.id, '--view', 'full', '--language', 'zh-CN'])
        report = {'kind': 'actual_synthetic_m6_combined_fixture_acceptance_not_real_consumer_recovery',
            'case': self.case, 'workItemId': self.id, 'candidateRuntimeDigest': self.binary_digest,
            'sourceManifestRawSha256': '299d90d475d2905a43868e5dc995993f0b79fafc7b9f32c452114833d37c50b8',
            'R': canonical(contract_R), 'A': canonical(contract_A), 'B': canonical(contract_B),
            'oldCanonicalRawSha256': sha(old_bytes), 'oldBlockedOutcomeRawSha256': sha(old_outcome),
            'oldMaterialReceiptRawSha256': sha(old_receipt_bytes), 'newCanonicalRawSha256': sha(new_bytes),
            'newMaterialRequestDigest': current_request['requestDigest'], 'newMaterialReceiptDigest': new_receipt['receiptDigest'],
            'planOnlyProcessesSpawned': 0, 'currentNodesExecuted': 4, 'currentNodesReused': 0,
            'maxConcurrentProcesses': 1, 'freshGreenPreflight': True, 'actualFixtureFinish': True,
            'actualFixtureArchive': True, 'historicalOriginalBytesUnchanged': True,
            'oldAndCurrentProducerAreSameActualCandidate': True,
            'limits': ['Synthetic normal-risk scaffold, explicit fixture self_declared review/risk decisions only.',
                       'No real person authentication, real consumer recovery, old producer upgrade compatibility or physical multi-file durability claim.']}
        self.immutable(self.logs / 'acceptance.actual.agent-recorded.json', (json.dumps(report, indent=2) + '\n').encode())
        self.done(); self.state.update(state='completed', currentStage=None); self.save_state()
        with (base / 'progress.md').open('a') as stream:
            stream.write('\nTask ' + self.case + ': complete; actual old and new four-node native cycles, source-only B, new decision, finish/archive and immutable history audit all passed. Runtime ' + self.binary_digest + '.\n')
        print(json.dumps(report, indent=2), flush=True)

if __name__ == '__main__':
    result = subprocess.run([str(gov), 'work-item', 'status', '--repo', str(owner), '--id', owner_item, '--json'], capture_output=True, timeout=90)
    assert result.returncode == 0
    state = json.loads(result.stdout)
    assert not state['blocking'] and not state['humanDecisionRequired'] and 'run_verification' in state['safeActions']
    fixture = Fixture(sys.argv[1])
    fixture.immutable(fixture.logs / 'owner-current-admission.actual.json', result.stdout)
    try:
        fixture.execute()
    except BaseException as error:
        fixture.state.update(state='stopped', stoppingReason=repr(error)); fixture.save_state()
        with (base / 'progress.md').open('a') as stream:
            stream.write('\nTask ' + fixture.case + ': stopped at ' + str(fixture.state['currentStage']) + '; actual preserved reason ' + repr(error) + '. No automatic retry.\n')
        raise
