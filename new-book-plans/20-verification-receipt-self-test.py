#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Focused disposable-repository tests for verification lock and receipt v2."""

from __future__ import annotations

import contextlib
import importlib.util
import json
import os
import pathlib
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest


HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import verification_lock as lock
import verification_lock_client as lock_client

SPEC = importlib.util.spec_from_file_location(
    "verification_receipt", HERE / "20-verification-receipt.py"
)
assert SPEC and SPEC.loader
receipt = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(receipt)


def git(repo: pathlib.Path, *args: str, check: bool = True) -> subprocess.CompletedProcess:
    proc = subprocess.run(
        ["git", "-C", str(repo), *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if check and proc.returncode:
        raise AssertionError(proc.stderr.decode("utf-8", "replace"))
    return proc


def write(path: pathlib.Path, body: str, *, executable: bool = False) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8", newline="\n")
    if executable:
        path.chmod(0o755)


def initialise_repo(repo: pathlib.Path) -> None:
    repo.mkdir(parents=True)
    git(repo, "init", "-q")
    git(repo, "config", "user.name", "Receipt Tests")
    git(repo, "config", "user.email", "receipt-tests@example.invalid")


class TemporaryGitTest(unittest.TestCase):
    def setUp(self) -> None:
        self.lock_environment = {
            key: os.environ.get(key) for key in (
                lock.TOKEN_ENV, lock.OWNER_PID_ENV, lock.OWNER_START_ENV,
                lock.COMMON_DIR_ENV, lock.NAME_ENV,
            )
        }
        for key in self.lock_environment:
            os.environ.pop(key, None)
        self.temporary = tempfile.TemporaryDirectory(prefix="rights-receipt-test-")
        self.base = pathlib.Path(self.temporary.name)

    def tearDown(self) -> None:
        self.temporary.cleanup()

        for key, value in self.lock_environment.items():
            if value is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = value

class LockTests(TemporaryGitTest):
    def setUp(self) -> None:
        super().setUp()
        self.repo = self.base / "repo"
        initialise_repo(self.repo)
        write(self.repo / "tracked.txt", "base\n")
        git(self.repo, "add", "tracked.txt")
        git(self.repo, "commit", "-qm", "base")

    def _clean_lock_env(self) -> dict[str, str]:
        env = dict(os.environ)
        for key in (
            lock.TOKEN_ENV,
            lock.OWNER_PID_ENV,
            lock.OWNER_START_ENV,
            lock.COMMON_DIR_ENV,
            lock.NAME_ENV,
        ):
            env.pop(key, None)
        return env

    def test_normal_nested_contention_wait_and_metadata(self) -> None:
        with lock.VerificationLock("verify", root=self.repo) as outer:
            self.assertFalse(outer.inherited)
            owner = json.loads(outer.owner_path.read_text(encoding="utf-8"))
            self.assertNotIn("argv", owner)
            self.assertNotIn(os.environ[lock.TOKEN_ENV], outer.owner_path.read_text())
            with lock.VerificationLock("nested", root=self.repo) as nested:
                self.assertTrue(nested.inherited)
            valid = subprocess.run(
                [
                    sys.executable,
                    str(HERE / "verification_lock.py"),
                    "validate-inherited",
                    "--name",
                    "verify",
                ],
                cwd=self.repo,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(valid.returncode, 0)
            wrong_name = subprocess.run(
                [
                    sys.executable,
                    str(HERE / "verification_lock.py"),
                    "validate-inherited",
                    "--name",
                    "forged-flag",
                ],
                cwd=self.repo,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertNotEqual(wrong_name.returncode, 0)
            started = time.monotonic()
            proc = subprocess.run(
                [
                    sys.executable,
                    str(HERE / "verification_lock.py"),
                    "run",
                    "--name",
                    "contender",
                    "--wait-for-lock",
                    "0.2",
                    "--",
                    "/bin/true",
                ],
                cwd=self.repo,
                env=self._clean_lock_env(),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(proc.returncode, lock.EX_TEMPFAIL)
            self.assertGreaterEqual(time.monotonic() - started, 0.15)
            self.assertNotIn(
                os.environ[lock.TOKEN_ENV].encode("ascii"), proc.stderr
            )
        with lock.VerificationLock("after-normal-exit", root=self.repo):
            pass

    def test_forged_inheritance_fails(self) -> None:
        env = self._clean_lock_env()
        env.update(
            {
                lock.TOKEN_ENV: "f" * 64,
                lock.OWNER_PID_ENV: str(os.getpid()),
                lock.OWNER_START_ENV: lock._process_start_ticks(os.getpid()),
                lock.COMMON_DIR_ENV: str(lock.git_common_dir(self.repo)),
                lock.NAME_ENV: "forged",
            }
        )
        proc = subprocess.run(
            [
                sys.executable,
                str(HERE / "verification_lock.py"),
                "validate-inherited",
                "--name",
                "forged",
            ],
            cwd=self.repo,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertNotEqual(proc.returncode, 0)

    def test_linked_worktree_uses_common_lock(self) -> None:
        linked = self.base / "linked"
        git(self.repo, "worktree", "add", "-qb", "linked-test", str(linked))
        self.assertEqual(lock.lock_paths(self.repo), lock.lock_paths(linked))
        with lock.VerificationLock("main-worktree", root=self.repo):
            proc = subprocess.run(
                [
                    sys.executable,
                    str(HERE / "verification_lock.py"),
                    "run",
                    "--name",
                    "linked-contender",
                    "--",
                    "/bin/true",
                ],
                cwd=linked,
                env=self._clean_lock_env(),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(proc.returncode, lock.EX_TEMPFAIL)

    def test_sigterm_releases_kernel_lock(self) -> None:
        self._signal_release(signal.SIGTERM)

    def test_sigkill_releases_kernel_lock(self) -> None:
        self._signal_release(signal.SIGKILL)

    def _signal_release(self, signum: int) -> None:
        ready = self.base / f"ready-{signum}"
        done = self.base / f"done-{signum}"
        child_code = (
            "import pathlib,sys,time; "
            "pathlib.Path(sys.argv[1]).write_text('ready'); "
            "time.sleep(0.8); pathlib.Path(sys.argv[2]).write_text('done')"
        )
        proc = subprocess.Popen(
            [
                sys.executable,
                str(HERE / "verification_lock.py"),
                "run", "--name", "signal-holder", "--",
                sys.executable, "-c", child_code, str(ready), str(done),
            ],
            cwd=self.repo,
            env=self._clean_lock_env(),
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        deadline = time.monotonic() + 3
        while not ready.exists() and time.monotonic() < deadline:
            time.sleep(0.02)
        self.assertTrue(ready.exists())
        proc.send_signal(signum)
        proc.wait(timeout=3)
        if signum == signal.SIGKILL:
            contender = subprocess.run(
                [
                    sys.executable,
                    str(HERE / "verification_lock.py"),
                    "run", "--name", "overlap-probe", "--", "/bin/true",
                ],
                cwd=self.repo,
                env=self._clean_lock_env(),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            self.assertEqual(contender.returncode, lock.EX_TEMPFAIL)
            deadline = time.monotonic() + 3
            while not done.exists() and time.monotonic() < deadline:
                time.sleep(0.02)
            self.assertTrue(done.exists())
        with lock.VerificationLock(
            "after-signal", root=self.repo, wait_seconds=1
        ):
            pass
    def test_lock_client_rejects_incomplete_inherited_owner(self) -> None:
        os.environ[lock.TOKEN_ENV] = "forged-incomplete-owner"
        try:
            result = lock_client.run_checker(
                lambda _arguments: 0,
                ("--refresh-and-check",),
            )
        finally:
            os.environ.pop(lock.TOKEN_ENV, None)
        self.assertEqual(result, 2)



class ReceiptTests(TemporaryGitTest):
    def tearDown(self) -> None:
        receipt.ROOT = self.old_root
        receipt.RECEIPT_DIR = self.old_receipt_dir
        if self.old_nibli_src is None:
            os.environ.pop("NIBLI_SRC", None)
        else:
            os.environ["NIBLI_SRC"] = self.old_nibli_src
        if self.old_nibli_pin is None:
            os.environ.pop("NIBLI_PIN", None)
        else:
            os.environ["NIBLI_PIN"] = self.old_nibli_pin
        super().tearDown()

    def _write_ledger(self) -> None:
        write(
            self.repo / receipt.LEDGER_PATH,
            json.dumps(self.ledger, indent=2, sort_keys=True) + "\n",
        )

    def _emit(self) -> pathlib.Path:
        self._stage_candidate()
        return receipt.emit_receipt(
            pathlib.Path("new-book-plans/verification-receipts"),
            ["./verify.sh"],
        )

    def setUp(self) -> None:
        TemporaryGitTest.setUp(self)
        self.repo = self.base / "repo"
        self.nibli = self.base / "nibli"
        self.source_version = "fs-ledger-2027-obligations-v1"
        self.audit_id = "FS-SAU-43"
        initialise_repo(self.repo)
        initialise_repo(self.nibli)
        verify_body = (
            "#!/usr/bin/env bash\nset -eu\n"
            "if [ \"${RECEIPT_TEST_DRIFT:-}\" = 1 ]; then "
            "printf 'drift\\n' > candidate.txt; fi\n"
            "if [ \"${RECEIPT_TEST_REBUILD_ENGINE:-}\" = 1 ]; then "
            "mkdir -p \"$NIBLI_SRC/target/release\"; "
            "printf '#!/usr/bin/env sh\\nexit 0\\n' > "
            "\"$NIBLI_SRC/target/release/nibli-pin\"; "
            "chmod 755 \"$NIBLI_SRC/target/release/nibli-pin\"; fi\n"
            "if [ \"${1:-}\" = --quick ] && "
            "[ \"${RECEIPT_TEST_QUICK_ENGINE_DRIFT:-}\" = 1 ]; then "
            "printf mutation >> \"$NIBLI_SRC/target/release/nibli-pin\"; fi\n"
            "if [ \"${1:-}\" = --quick ] && "
            "[ -n \"${RECEIPT_TEST_QUICK_MARKER:-}\" ]; then "
            "printf 'quick\\n' > \"$RECEIPT_TEST_QUICK_MARKER\"; fi\n"
            "printf 'verifier PASS\\n'\n"
        )
        write(self.repo / "verify.sh", verify_body, executable=True)
        write(
            self.repo / receipt.PROTOCOL_PATH,
            "Status: " + receipt.PROTOCOL_STATUS + "\n\nPolicy\n",
        )
        self.todo_body = (
            "# TODO\n\n"
            "- [ ] **Temporary verifier item.**\n"
            "  Remove this exact block after closure.\n\n"
            "- [ ] **Specify obligations without reciprocal bargains.**\n"
            "  This remains unfinished.\n"
        )
        write(self.repo / "TODO.md", self.todo_body)
        self.ledger = {
            "source_version": "fs-ledger-2026-prior-closed-v2",
            "scope_audits": [],
            "closure_record": {
                "gate": "gate-a",
                "permitted_claim": "prior structural claim",
                "envelope_ref": "FS-ENV-01",
                "assurance_record_refs": ["FS-ASR-01"],
                "residual_refs": ["FS-DEF-01"],
                "claim_limitations": [{
                    "defect_ref": "FS-DEF-01",
                    "affected_claim_ref": "FS-CLM-01",
                    "public_claim_restriction": "No operational claim.",
                }],
            },
            "acceptance_gate": {
                "verdict": "prior structural verdict",
                "rollup_rule": "all conditions",
                "gate_a_status": "passed",
            },
            "owner_ref": "TODO.md::Specify obligations",
        }
        self._write_ledger()
        write(self.repo / "candidate.txt", "base\n")
        write(self.repo / "new-book-plans/testdata/fixture.txt", "fixture\n")
        write(self.repo / "verification-shard-runner.sh", "#!/bin/sh\nexit 0\n", executable=True)
        write(self.repo / "new-book-plans/verification_lock_client.py", "# helper\n")
        for path in (
            receipt.CLOSURE_TRANSITION_GENERATED_PATHS
            | receipt.TRANSITION_BYTE_STABLE_GENERATED_PATHS
        ):
            write(self.repo / path, "base projection\n")
        git(self.repo, "add", ".")
        git(self.repo, "commit", "-qm", "base closed source")
        write(self.nibli / ".gitignore", "target/\n")
        write(self.nibli / "Cargo.toml", "[package]\nname='fake'\nversion='0.0.0'\n")
        git(self.nibli, "add", ".gitignore", "Cargo.toml")
        git(self.nibli, "commit", "-qm", "fake engine source")
        write(
            self.nibli / "target/release/nibli-pin",
            "#!/usr/bin/env sh\nexit 0\n",
            executable=True,
        )
        self.old_root = receipt.ROOT
        self.old_receipt_dir = receipt.RECEIPT_DIR
        receipt.ROOT = self.repo
        receipt.RECEIPT_DIR = self.repo / "new-book-plans/verification-receipts"
        self.old_nibli_src = os.environ.get("NIBLI_SRC")
        self.old_nibli_pin = os.environ.get("NIBLI_PIN")
        os.environ["NIBLI_SRC"] = str(self.nibli)
        os.environ.pop("NIBLI_PIN", None)

    def _pending_audit(self) -> dict:
        return {
            "id": f"{self.audit_id}-PENDING",
            "title": "Receipt-aware repository audit pending",
            "source_version": self.source_version,
            "scope_sha256": "1" * 64,
            "protocol_sha256": "2" * 64,
            "executed_at_utc": "2026-08-23T00:00:00Z",
            "method": "repository adversarial audit",
            "criterion_coverage": ["semantic scope"],
            "control_refs": ["CTRL-01"],
            "commands": ["python3 new-book-plans/13-full-society-ledger.py --refresh-and-check"],
            "finding_refs": ["FS-DEF-01"],
            "result": "pending",
            "policy_basis": "new-book-plans/full-society-scope-review-protocol.md::Policy",
            "evidence_ceiling": "Repository structure only.",
        }

    def _stage_candidate(self) -> None:
        self.ledger = {
            **self.ledger,
            "source_version": self.source_version,
            "scope_audits": [self._pending_audit()],
            "closure_record": None,
            "acceptance_gate": {
                "verdict": "pending structural verdict",
                "rollup_rule": "all conditions",
                "gate_a_status": "not-passed",
            },
        }
        self._write_ledger()
        write(self.repo / "candidate.txt", "candidate\n")
        git(self.repo, "add", "candidate.txt", receipt.LEDGER_PATH)

    def _stage_projections(self, label: str, paths: set[str]) -> None:
        for path in paths:
            write(self.repo / path, label + " projection\n")
        git(self.repo, "add", *sorted(paths))

    def test_dirty_and_untracked_inputs_fail(self) -> None:
        self._stage_candidate()
        write(self.repo / "TODO.md", "dirty\n")
        with self.assertRaises(receipt.ReceiptError):
            receipt._fully_staged_candidate()
        git(self.repo, "restore", "TODO.md")
        write(self.repo / "untracked.txt", "x\n")
        with self.assertRaises(receipt.ReceiptError):
            receipt._fully_staged_candidate()

    def test_integrity_engine_environment_missing_evidence_and_v1(self) -> None:
        path = self._emit()
        loaded = receipt.load_and_validate_receipt(path, root=self.repo)
        self.assertEqual(loaded["schema_version"], 2)
        self.assertEqual(loaded["audit_id"], self.audit_id)

        cached_receipt = path.read_bytes()
        path.unlink()
        try:
            cached_loaded = receipt.load_and_validate_receipt(
                path,
                root=self.repo,
                raw_bytes=cached_receipt,
                require_local=False,
                check_environment=False,
                check_engine=False,
            )
            self.assertEqual(cached_loaded["receipt_id"], loaded["receipt_id"])
        finally:
            path.write_bytes(cached_receipt)

        wrong_name = path.with_name("sha256-" + "0" * 64 + ".json")
        shutil.copyfile(path, wrong_name)
        with self.assertRaises(receipt.ReceiptError):
            receipt.load_and_validate_receipt(wrong_name, root=self.repo)
        wrong_name.unlink()

        original_receipt = path.read_bytes()
        mutated = json.loads(original_receipt)
        mutated["evidence_ceiling"] += " mutated"
        path.write_text(json.dumps(mutated), encoding="utf-8")
        with self.assertRaises(receipt.ReceiptError):
            receipt.load_and_validate_receipt(path, root=self.repo)
        path.write_bytes(original_receipt)

        common = lock.git_common_dir(self.repo)
        evidence = (
            common
            / receipt.EVIDENCE_SUBDIR
            / f"sha256-{loaded['receipt_id']}"
        )
        transcript = evidence / "transcript.log"
        transcript_bytes = transcript.read_bytes()
        transcript.unlink()
        with self.assertRaises(receipt.ReceiptError):
            receipt.load_and_validate_receipt(path, root=self.repo)
        transcript.write_bytes(transcript_bytes)

        binary = self.nibli / "target/release/nibli-pin"
        binary_bytes = binary.read_bytes()
        binary.write_bytes(binary_bytes + b"mutation")
        with self.assertRaises(receipt.ReceiptError):
            receipt.load_and_validate_receipt(path, root=self.repo)
        binary.write_bytes(binary_bytes)

        engine_source = self.nibli / "Cargo.toml"
        engine_source_bytes = engine_source.read_bytes()
        engine_source.write_bytes(engine_source_bytes + b"# source mutation\n")
        with self.assertRaises(receipt.ReceiptError):
            receipt.load_and_validate_receipt(path, root=self.repo)
        engine_source.write_bytes(engine_source_bytes)

        old_lang = os.environ.get("LANG")
        os.environ["LANG"] = "receipt-test-drift"
        try:
            with self.assertRaises(receipt.ReceiptError):
                receipt.load_and_validate_receipt(path, root=self.repo)
        finally:
            if old_lang is None:
                os.environ.pop("LANG", None)
            else:
                os.environ["LANG"] = old_lang

        legacy = {
            "candidate_commit_sha": receipt.LEGACY_CANDIDATE,
            "verified_at_utc": "2026-08-22T06:26:52Z",
            "commands": list(receipt.LEGACY_REQUIRED_COMMANDS),
            "result": "all-passed",
            "transcript_sha256": receipt.LEGACY_TRANSCRIPT_SHA256,
        }
        self.assertIs(
            receipt._validate_legacy_v1(
                legacy,
                source_version=receipt.LEGACY_SOURCE_VERSION,
                audit_id=receipt.LEGACY_AUDIT_ID,
            ),
            legacy,
        )
        invalid_command_lists = (
            legacy["commands"][:-1],
            legacy["commands"] + ["unexpected command"],
            list(reversed(legacy["commands"])),
        )
        for commands in invalid_command_lists:
            with self.subTest(commands=commands):
                mutated = dict(legacy)
                mutated["commands"] = commands
                with self.assertRaises(receipt.ReceiptError):
                    receipt._validate_legacy_v1(
                        mutated,
                        source_version=receipt.LEGACY_SOURCE_VERSION,
                        audit_id=receipt.LEGACY_AUDIT_ID,
                    )
        with self.assertRaises(receipt.ReceiptError):
            receipt._validate_legacy_v1(
                legacy,
                source_version=self.source_version,
                audit_id=receipt.LEGACY_AUDIT_ID,
            )

    def test_environment_field_names_are_globally_sorted(self) -> None:
        old_tz = os.environ.get("TZ")
        os.environ["TZ"] = "UTC"
        try:
            path = self._emit()
            loaded = receipt.load_and_validate_receipt(path, root=self.repo)
            fields = loaded["environment"]["fields"]
            self.assertEqual(fields, sorted(set(fields)))
        finally:
            if old_tz is None:
                os.environ.pop("TZ", None)
            else:
                os.environ["TZ"] = old_tz

    def test_audit_rejects_unclassified_path(self) -> None:
        path = self._emit()
        relative = path.relative_to(self.repo).as_posix()
        git(self.repo, "commit", "-qm", "candidate")
        self.ledger["scope_audits"].append(
            {
                "id": self.audit_id,
                "source_version": self.source_version,
                "result": "passed-with-recorded-limits",
                "verification_receipt_ref": relative,
            }
        )
        self._write_ledger()
        write(self.repo / "unexpected.txt", "not administrative\n")
        git(self.repo, "add", receipt.LEDGER_PATH, relative, "unexpected.txt")
        with self.assertRaises(receipt.ReceiptError):
            receipt.validate_commit_gate(path, "audit", root=self.repo)


    def test_audit_rejects_verifier_and_fixture_mutations(self) -> None:
        path = self._emit()
        relative = path.relative_to(self.repo).as_posix()
        git(self.repo, "commit", "-qm", "candidate")
        self.ledger["scope_audits"].append(
            {
                "id": self.audit_id,
                "source_version": self.source_version,
                "result": "passed-with-recorded-limits",
                "verification_receipt_ref": relative,
            }
        )
        self._write_ledger()
        write(
            self.repo / "verify.sh",
            "#!/usr/bin/env bash\nprintf 'mutated verifier\\n'\n",
            executable=True,
        )
        git(self.repo, "add", receipt.LEDGER_PATH, relative, "verify.sh")
        with self.assertRaises(receipt.ReceiptError):
            receipt.validate_commit_gate(path, "audit", root=self.repo)

        git(self.repo, "restore", "--staged", "verify.sh")
        git(self.repo, "restore", "verify.sh")
        fixture = "new-book-plans/testdata/fixture.txt"
        write(self.repo / fixture, "mutated fixture\n")
        git(self.repo, "add", fixture)
        with self.assertRaises(receipt.ReceiptError):
            receipt.validate_commit_gate(path, "audit", root=self.repo)

    def _stage_passing_audit(
        self, path: pathlib.Path, compact: dict, *, mutate_title: bool = False
    ) -> str:
        relative = path.relative_to(self.repo).as_posix()
        passing = receipt._expected_passing_audit(
            self.ledger["scope_audits"][-1], compact, relative
        )
        if mutate_title:
            passing["title"] += " mutated"
        self.ledger["scope_audits"].append(passing)
        self._write_ledger()
        self._stage_projections(
            "audit", receipt.AUDIT_TRANSITION_GENERATED_PATHS)
        git(self.repo, "add", receipt.LEDGER_PATH, relative)
        return relative

    def test_all_three_strict_transitions_and_reference_projection(self) -> None:
        path = self._emit()
        compact = json.loads(path.read_text(encoding="utf-8"))
        git(self.repo, "commit", "-qm", "candidate")
        closure_projection = (
            "new-book-plans/"
            "constitutional-closure-and-model-allocation-audit.md"
        )
        closure_projection_before = (
            self.repo / closure_projection
        ).read_bytes()
        relative = self._stage_passing_audit(path, compact)
        self.assertEqual(
            (self.repo / closure_projection).read_bytes(),
            closure_projection_before,
        )
        marker = self.base / "quick-marker"
        os.environ["RECEIPT_TEST_QUICK_MARKER"] = str(marker)
        try:
            receipt.run_commit_gate(path, "audit", root=self.repo)
        finally:
            os.environ.pop("RECEIPT_TEST_QUICK_MARKER", None)
        self.assertTrue(marker.is_file())
        git(self.repo, "commit", "-qm", "audit")
        audit_commit = git(
            self.repo, "rev-parse", "HEAD"
        ).stdout.decode("ascii").strip()
        candidate_commit = receipt.validate_recorded_transition(
            compact, audit_commit, "audit",
            receipt_path=relative, root=self.repo,
        )
        self.assertEqual(
            candidate_commit,
            receipt.resolve_candidate_commit(compact, root=self.repo),
        )

        audit = self.ledger["scope_audits"][-1]
        self.ledger["closure_record"] = {
            "gate": "gate-a",
            "permitted_claim": "prior structural claim",
            "candidate_commit_sha": audit_commit,
            "source_version": self.source_version,
            "scope_sha256": audit["scope_sha256"],
            "envelope_ref": "FS-ENV-01",
            "audit_cutoff_at_utc": audit["executed_at_utc"],
            "scope_audit_ref": audit["id"],
            "assurance_record_refs": ["FS-ASR-01"],
            "residual_refs": ["FS-DEF-01"],
            "claim_limitations": [{
                "defect_ref": "FS-DEF-01",
                "affected_claim_ref": "FS-CLM-01",
                "public_claim_restriction": "No operational claim.",
            }],
            "verification_receipt_ref": relative,
            "closure_policy_ref": audit["policy_basis"],
        }
        self.ledger["acceptance_gate"] = {
            "verdict": "prior structural verdict",
            "rollup_rule": "all conditions",
            "gate_a_status": "passed",
        }
        self._write_ledger()
        self._stage_projections(
            "closure", receipt.CLOSURE_TRANSITION_GENERATED_PATHS)
        git(self.repo, "add", receipt.LEDGER_PATH)
        self.assertEqual(
            (self.repo / closure_projection).read_bytes(),
            closure_projection_before,
        )
        write(self.repo / closure_projection, "unexpected closure projection\n")
        git(self.repo, "add", closure_projection)
        with self.assertRaises(receipt.ReceiptError):
            receipt.validate_commit_gate(path, "closure", root=self.repo)
        git(self.repo, "restore", "--staged", closure_projection)
        git(self.repo, "restore", closure_projection)
        receipt.run_commit_gate(path, "closure", root=self.repo)
        git(self.repo, "commit", "-qm", "closure")
        closure_commit = git(
            self.repo, "rev-parse", "HEAD"
        ).stdout.decode("ascii").strip()
        self.assertEqual(
            receipt.validate_recorded_transition(
                compact, closure_commit, "closure",
                receipt_path=relative, root=self.repo,
            ),
            audit_commit,
        )

        remaining = (
            "# TODO\n\n"
            "- [ ] **Specify obligations without reciprocal bargains.**\n"
            "  This remains unfinished.\n"
        )
        write(self.repo / "TODO.md", remaining)
        git(self.repo, "add", "TODO.md")
        receipt.run_commit_gate(path, "tracker", root=self.repo)
        wrong_block = (
            "# TODO\n\n"
            "- [ ] **Temporary verifier item.**\n"
            "  Remove this exact block after closure.\n"
        )
        write(self.repo / "TODO.md", wrong_block)
        git(self.repo, "add", "TODO.md")
        with self.assertRaises(receipt.ReceiptError):
            receipt.validate_commit_gate(path, "tracker", root=self.repo)
        duplicate_needle = remaining + (
            "\n- [ ] **Specify obligations again.**\n"
            "  Duplicate Specify obligations needle.\n"
        )
        write(self.repo / "TODO.md", duplicate_needle)
        git(self.repo, "add", "TODO.md")
        with self.assertRaises(receipt.ReceiptError):
            receipt.validate_commit_gate(path, "tracker", root=self.repo)
        write(self.repo / "TODO.md", remaining.replace(
            "This remains unfinished.", "This was silently rewritten."
        ))
        git(self.repo, "add", "TODO.md")
        with self.assertRaises(receipt.ReceiptError):
            receipt.validate_commit_gate(path, "tracker", root=self.repo)

    def test_strict_audit_rejects_semantic_mutation(self) -> None:
        path = self._emit()
        compact = json.loads(path.read_text(encoding="utf-8"))
        git(self.repo, "commit", "-qm", "candidate")
        self._stage_passing_audit(path, compact, mutate_title=True)
        with self.assertRaises(receipt.ReceiptError):
            receipt.validate_commit_gate(path, "audit", root=self.repo)

    def test_audit_requires_exact_script_13_projection_set(self) -> None:
        path = self._emit()
        compact = json.loads(path.read_text(encoding="utf-8"))
        git(self.repo, "commit", "-qm", "candidate")
        self._stage_passing_audit(path, compact)

        missing = sorted(receipt.AUDIT_TRANSITION_GENERATED_PATHS)[0]
        git(self.repo, "restore", "--staged", missing)
        git(self.repo, "restore", missing)
        with self.assertRaises(receipt.ReceiptError):
            receipt.validate_commit_gate(path, "audit", root=self.repo)

        write(self.repo / missing, "audit projection\n")
        git(self.repo, "add", missing)
        unexpected = (
            "new-book-plans/"
            "constitutional-closure-and-model-allocation-audit.md"
        )
        write(self.repo / unexpected, "unexpected audit projection\n")
        git(self.repo, "add", unexpected)
        with self.assertRaises(receipt.ReceiptError):
            receipt.validate_commit_gate(path, "audit", root=self.repo)

    def test_emit_rejects_staged_input_drift_during_run(self) -> None:
        self._stage_candidate()
        os.environ["RECEIPT_TEST_DRIFT"] = "1"
        try:
            with self.assertRaises(receipt.ReceiptError):
                receipt.emit_receipt(
                    pathlib.Path("new-book-plans/verification-receipts"),
                    ["./verify.sh"],
                )
        finally:
            os.environ.pop("RECEIPT_TEST_DRIFT", None)

    def test_commit_gate_rejects_quick_input_drift(self) -> None:
        path = self._emit()
        compact = json.loads(path.read_text(encoding="utf-8"))
        git(self.repo, "commit", "-qm", "candidate")
        self._stage_passing_audit(path, compact)
        os.environ["RECEIPT_TEST_DRIFT"] = "1"
        try:
            with self.assertRaises(receipt.ReceiptError):
                receipt.run_commit_gate(path, "audit", root=self.repo)
        finally:
            os.environ.pop("RECEIPT_TEST_DRIFT", None)

    def test_commit_gate_rejects_post_quick_engine_drift(self) -> None:
        path = self._emit()
        compact = json.loads(path.read_text(encoding="utf-8"))
        git(self.repo, "commit", "-qm", "candidate")
        self._stage_passing_audit(path, compact)
        os.environ["RECEIPT_TEST_QUICK_ENGINE_DRIFT"] = "1"
        try:
            with self.assertRaises(receipt.ReceiptError):
                receipt.run_commit_gate(path, "audit", root=self.repo)
        finally:
            os.environ.pop("RECEIPT_TEST_QUICK_ENGINE_DRIFT", None)

    def test_stale_engine_binary_may_be_rebuilt_and_final_bytes_bind(self) -> None:
        binary = self.nibli / "target/release/nibli-pin"
        write(binary, "#!/usr/bin/env sh\nexit 99\n", executable=True)
        os.environ["RECEIPT_TEST_REBUILD_ENGINE"] = "1"
        try:
            path = self._emit()
        finally:
            os.environ.pop("RECEIPT_TEST_REBUILD_ENGINE", None)
        loaded = receipt.load_and_validate_receipt(path, root=self.repo)
        self.assertEqual(loaded["engine"]["binary_sha256"], receipt._sha256(
            binary.read_bytes()
        ))

    def test_missing_engine_binary_may_be_built(self) -> None:
        (self.nibli / "target/release/nibli-pin").unlink()
        os.environ["RECEIPT_TEST_REBUILD_ENGINE"] = "1"
        try:
            path = self._emit()
        finally:
            os.environ.pop("RECEIPT_TEST_REBUILD_ENGINE", None)
        receipt.load_and_validate_receipt(path, root=self.repo)

    def test_shell_and_lock_client_are_verifier_inputs(self) -> None:
        self.assertEqual(
            receipt._classify("verification-shard-runner.sh"), "verifier-input"
        )
        self.assertEqual(
            receipt._classify("new-book-plans/verification_lock_client.py"),
            "verifier-input",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
