import pathlib
import subprocess
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).parent))

from runtime_benchmark_support import filesystem_metadata


class Result:
    def __init__(self, returncode=0, stdout=b"apfs\n"):
        self.returncode = returncode
        self.stdout = stdout


class FilesystemMetadataTests(unittest.TestCase):
    def test_macos_uses_bsd_stat_and_returns_type(self):
        calls = []

        def runner(command, **kwargs):
            calls.append((command, kwargs))
            if command[0] == "stat":
                return Result(stdout=b"disk3s5\n")
            return Result(stdout=b"   Type (Bundle):             apfs\n")

        value = filesystem_metadata(pathlib.Path("/repo"), system="Darwin", runner=runner)
        self.assertEqual(value, {"available": True, "type": "apfs"})
        self.assertEqual(calls[0][0], ["stat", "-f", "%Sd", "/repo"])
        self.assertEqual(calls[1][0], ["diskutil", "info", "disk3s5"])

    def test_macos_accepts_filesystem_personality_fallback(self):
        def runner(command, **kwargs):
            if command[0] == "stat":
                return Result(stdout=b"disk3s5\n")
            return Result(stdout=b"   File System Personality:  APFS\n")

        value = filesystem_metadata(pathlib.Path("/repo"), system="Darwin", runner=runner)
        self.assertEqual(value, {"available": True, "type": "APFS"})

    def test_linux_uses_gnu_stat(self):
        def runner(command, **kwargs):
            return Result(stdout=b"overlay\n")

        value = filesystem_metadata(pathlib.Path("/repo"), system="Linux", runner=runner)
        self.assertEqual(value, {"available": True, "type": "overlay"})

    def test_probe_failure_is_explicitly_unavailable(self):
        def runner(command, **kwargs):
            return Result(returncode=1, stdout=b"")

        value = filesystem_metadata(pathlib.Path("/repo"), system="Darwin", runner=runner)
        self.assertEqual(value["available"], False)
        self.assertEqual(value["reason"], "filesystem_type_unavailable")

    def test_probe_exception_is_explicitly_unavailable(self):
        def runner(command, **kwargs):
            raise subprocess.TimeoutExpired(command, 30)

        value = filesystem_metadata(pathlib.Path("/repo"), system="Linux", runner=runner)
        self.assertEqual(value["available"], False)
        self.assertEqual(value["reason"], "filesystem_type_unavailable")

    def test_unknown_platform_is_explicitly_unsupported(self):
        value = filesystem_metadata(pathlib.Path("/repo"), system="Plan9")
        self.assertEqual(value["available"], False)
        self.assertEqual(value["reason"], "platform_filesystem_type_unsupported")
