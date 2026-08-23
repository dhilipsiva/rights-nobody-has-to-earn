#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Focused behavioral tests for atomic reviewed-artifact refreshes."""

from __future__ import annotations

import importlib.util
import os
import pathlib
import stat
import subprocess
import tempfile
import unittest


HERE = pathlib.Path(__file__).resolve().parent
LEDGER_PATH = HERE / "13-full-society-ledger.py"
SPEC = importlib.util.spec_from_file_location(
    "verification_refresh_ledger", LEDGER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("cannot load the full-society ledger refresh hooks")
LEDGER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(LEDGER)


class RefreshSelfTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temporary.name)
        self.source = self.root / "immutable-source.txt"
        self.first = self.root / "first-report.md"
        self.second = self.root / "second-report.md"
        self.source.write_bytes(b"source-v1\n")
        self.first.write_bytes(b"already-current\n")
        self.second.write_bytes(b"old-second\n")
        os.chmod(self.first, 0o640)
        os.chmod(self.second, 0o750)
        self._git("init", "--quiet")
        self._git("config", "user.name", "Verification Refresh Test")
        self._git("config", "user.email", "refresh-test@example.invalid")
        self._git("add", ".")
        self._git("commit", "--quiet", "-m", "fixture")

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def _git(self, *arguments: str) -> None:
        subprocess.run(
            ["git", *arguments],
            cwd=self.root,
            check=True,
            capture_output=True,
            text=True,
        )

    def _snapshot(self):
        snapshot = LEDGER.ImmutableRepositoryInputs(self.root)
        snapshot.read_bytes(self.source)
        return snapshot

    def _outputs(self) -> list[tuple[pathlib.Path, str]]:
        return [
            (self.first, "already-current\n"),
            (self.second, "new-second\n"),
        ]

    def _output_state(self) -> tuple[tuple[bytes, int], tuple[bytes, int]]:
        return (
            (self.first.read_bytes(), stat.S_IMODE(self.first.stat().st_mode)),
            (self.second.read_bytes(), stat.S_IMODE(self.second.stat().st_mode)),
        )

    def _assert_no_refresh_artifacts(self) -> None:
        leftovers = sorted(
            path.name
            for path in self.root.iterdir()
            if ".refresh-" in path.name or ".backup-" in path.name
        )
        self.assertEqual(leftovers, [])

    def test_success_preserves_bytes_modes_and_one_read_contract(self) -> None:
        snapshot = self._snapshot()

        LEDGER.atomic_refresh_and_check(self._outputs(), snapshot)

        self.assertEqual(self.first.read_bytes(), b"already-current\n")
        self.assertEqual(self.second.read_bytes(), b"new-second\n")
        self.assertEqual(stat.S_IMODE(self.first.stat().st_mode), 0o640)
        self.assertEqual(stat.S_IMODE(self.second.stat().st_mode), 0o750)
        expected_paths = {
            self.source.resolve(), self.first.resolve(), self.second.resolve()
        }
        self.assertEqual(set(snapshot._initial_reads), expected_paths)
        self.assertTrue(
            all(count == 1 for count in snapshot._initial_reads.values()))
        self.assertEqual(set(snapshot._rehashes), expected_paths)
        self.assertTrue(all(count == 1 for count in snapshot._rehashes.values()))
        self._assert_no_refresh_artifacts()

    def test_preinstall_input_drift_rejects_without_output_change(self) -> None:
        snapshot = self._snapshot()
        original = self._output_state()
        self.source.write_bytes(b"source-drift-before-install\n")

        with self.assertRaises(LEDGER.LedgerError):
            LEDGER.atomic_refresh_and_check(self._outputs(), snapshot)

        self.assertEqual(self._output_state(), original)
        self._assert_no_refresh_artifacts()

    def test_second_replace_failure_restores_all_outputs(self) -> None:
        snapshot = self._snapshot()
        original = self._output_state()
        real_replace = LEDGER.os.replace
        forward_replacements = 0

        def fail_second_refresh(source, destination):
            nonlocal forward_replacements
            if ".refresh-" in pathlib.Path(source).name:
                forward_replacements += 1
                if forward_replacements == 2:
                    raise OSError("injected second replacement failure")
            return real_replace(source, destination)

        LEDGER.os.replace = fail_second_refresh
        try:
            with self.assertRaises(LEDGER.LedgerError):
                LEDGER.atomic_refresh_and_check(self._outputs(), snapshot)
        finally:
            LEDGER.os.replace = real_replace

        self.assertEqual(forward_replacements, 2)
        self.assertEqual(self._output_state(), original)
        self._assert_no_refresh_artifacts()

    def test_postinstall_input_drift_rolls_back_and_rejects(self) -> None:
        snapshot = self._snapshot()
        original = self._output_state()
        real_replace = LEDGER.os.replace
        forward_replacements = 0

        def drift_after_second_refresh(source, destination):
            nonlocal forward_replacements
            result = real_replace(source, destination)
            if ".refresh-" in pathlib.Path(source).name:
                forward_replacements += 1
                if forward_replacements == 2:
                    self.source.write_bytes(b"source-drift-after-install\n")
            return result

        LEDGER.os.replace = drift_after_second_refresh
        try:
            with self.assertRaises(LEDGER.LedgerError):
                LEDGER.atomic_refresh_and_check(self._outputs(), snapshot)
        finally:
            LEDGER.os.replace = real_replace

        self.assertEqual(forward_replacements, 2)
        self.assertEqual(self._output_state(), original)
        self._assert_no_refresh_artifacts()


if __name__ == "__main__":
    unittest.main(verbosity=2)
