import json
import os
from pathlib import Path
import time

directory = Path('/workspace/ai-cockpit-wi1081-append-only-evidence-observations/.ai/evidence/WI-1081-append-only-evidence-observations.pause-observation-probe3-20261014')
deadline = time.monotonic() + 30
while not (directory / 'reuse-accepted.signal').exists():
    if time.monotonic() >= deadline:
        raise RuntimeError('bounded monitor did not receive the reuse signal')
    time.sleep(0.005)
child = os.fork()
if child == 0:
    try:
        os.chdir('/workspace')
        (directory / 'independent-monitor-child.json').write_text(json.dumps({
            'pid': os.getpid(), 'ppid': os.getppid(), 'uid': os.getuid(),
            'startedUnixNs': time.time_ns(),
            'userNamespace': os.readlink('/proc/self/ns/user'),
            'stat': Path('/proc/self/stat').read_text(),
            'kind': 'ordinary_read_only_monitor_child'}, indent=2) + '\n')
        os.statvfs('/workspace')
        time.sleep(2)
    finally:
        os._exit(0)
_, status = os.waitpid(child, 0)
print(json.dumps({'monitorChild': child, 'waitStatus': status, 'reaped': True}), flush=True)
