"""Small, deterministic helpers shared by the runtime benchmark and tests."""

from __future__ import annotations

import platform
import re
import subprocess
from pathlib import Path
from typing import Callable, Sequence


def filesystem_metadata(
    repo: Path,
    *,
    system: str | None = None,
    runner: Callable[..., object] = subprocess.run,
) -> dict[str, object]:
    """Return the filesystem type without turning a failed probe into a fact.

    macOS uses BSD ``stat`` to resolve the filesystem device and ``diskutil``
    to read its type; Linux uses GNU ``stat``'s ``-f -c %T`` form. ``runner``
    is injectable so both normal and failure paths are tested without
    depending on the host filesystem.
    """

    system = system or platform.system()
    if system == "Darwin":
        # BSD stat's %T is the path's file type (for example, "/" for a
        # directory), not the mounted filesystem type. Resolve the device
        # with %Sd, then ask diskutil for the volume metadata.
        device_command: Sequence[str] = ("stat", "-f", "%Sd", str(repo))
    elif system == "Linux":
        command = ("stat", "-f", "-c", "%T", str(repo))
    else:
        return {
            "available": False,
            "type": None,
            "reason": "platform_filesystem_type_unsupported",
        }
    try:
        if system == "Darwin":
            device_result = runner(
                list(device_command),
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                check=False,
                timeout=30,
            )
            if getattr(device_result, "returncode", 1) != 0:
                return {"available": False, "type": None, "reason": "filesystem_type_unavailable"}
            device_stdout = getattr(device_result, "stdout", b"")
            device = (
                device_stdout.decode("utf-8", "replace")
                if isinstance(device_stdout, bytes)
                else str(device_stdout)
            ).strip()
            if not device or device in {"/", "?", "unknown"}:
                return {"available": False, "type": None, "reason": "filesystem_type_unavailable"}
            command = ("diskutil", "info", device)
        result = runner(
            list(command),
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired):
        return {"available": False, "type": None, "reason": "filesystem_type_unavailable"}
    returncode = getattr(result, "returncode", 1)
    stdout = getattr(result, "stdout", b"")
    value = stdout.decode("utf-8", "replace") if isinstance(stdout, bytes) else str(stdout)
    if returncode != 0:
        return {"available": False, "type": None, "reason": "filesystem_type_unavailable"}
    if system == "Darwin":
        match = re.search(r"^\s*Type \(Bundle\):\s*(\S+)\s*$", value, re.MULTILINE)
        if not match:
            match = re.search(r"^\s*File System Personality:\s*(\S+)\s*$", value, re.MULTILINE)
        value = match.group(1) if match else ""
    else:
        value = value.strip()
    if not value or value in {"/", "?", "unknown"}:
        return {"available": False, "type": None, "reason": "filesystem_type_unavailable"}
    return {"available": True, "type": value}
