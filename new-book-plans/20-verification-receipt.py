#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Emit and validate content-addressed authoritative-verification receipts.

Schema v2 binds a fully staged prospective Git tree. The compact, tracked
receipt contains digests and identities; the full path manifest and transcript
remain under the Git common directory. Administrative reuse is fail-closed and
is limited to the audit, closure, and tracker transitions defined below.
"""

from __future__ import annotations

import argparse
import contextlib
import copy
import datetime as dt
import hashlib
import json
import os
import pathlib
import platform
import re
import signal
import subprocess
import sys
import time
import uuid
from typing import Iterable, Iterator, Sequence

from verification_lock import (
    EX_TEMPFAIL,
    VerificationLock,
    VerificationLockBusy,
    VerificationLockError,
    git_common_dir,
    repository_root,
)


ROOT = pathlib.Path(__file__).resolve().parents[1]
RECEIPT_DIR = ROOT / "new-book-plans/verification-receipts"
LEDGER_PATH = "new-book-plans/full-society-ledger.json"
TODO_PATH = "TODO.md"
PROTOCOL_PATH = "new-book-plans/full-society-scope-review-protocol.md"
PROTOCOL_VERSION = 5
PROTOCOL_STATUS = (
    "repository-enforced 2026-08-23 -- receipt-aware "
    "mechanical-closure protocol v5"
)
EVIDENCE_CEILING = (
    "Repository verification over the bound staged bytes only; "
    "no external truth, operation, delivery, liveness, feasibility, "
    "calibration, or institutional action follows."
)
EVIDENCE_SUBDIR = pathlib.Path("rights-verification/receipts")
_HEX40_RE = re.compile(r"^[0-9a-f]{40}$")
_HEX64_RE = re.compile(r"^[0-9a-f]{64}$")
_RECEIPT_NAME_RE = re.compile(r"^sha256-([0-9a-f]{64})\.json$")
_UTC_RE = re.compile(
    r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$")
_SAFE_REF_PATH_RE = re.compile(r"^[A-Za-z0-9_.][-A-Za-z0-9_./]*$")

LEGACY_CANDIDATE = "e0e0ca1a09dc8bceaac95f29ab5f1afdc9795bb5"
LEGACY_SOURCE_VERSION = "fs-ledger-2026-08-21-state-form-prose-v1"
LEGACY_AUDIT_ID = "FS-SAU-34"
LEGACY_TRANSCRIPT_SHA256 = (
    "dc0eb1d869629a9093457fcc8a7c48d5a438777bae756e24a0447e4d60e1032f"
)
LEGACY_REQUIRED_COMMANDS = (
    "python3 new-book-plans/14-reader-evidence.py --check",
    "python3 new-book-plans/14-reader-evidence.py --check --execute",
    "python3 new-book-plans/17-full-society-power-source-manifest.py --check",
    "python3 new-book-plans/13-full-society-ledger.py",
    "python3 new-book-plans/13-full-society-ledger.py --check",
    "python3 new-book-plans/16-constitutional-closure.py",
    "python3 new-book-plans/16-constitutional-closure.py --check",
    "./verify.sh --quick",
    "./verify.sh",
    "git diff --check",
)

GENERATED_PATHS = {
    "new-book-plans/3-spine.md",
    "new-book-plans/amendment-semantics-audit.md",
    "new-book-plans/assertion-surface-audit.md",
    "new-book-plans/constitutional-closure-and-model-allocation-audit.md",
    "new-book-plans/full-society-ledger.md",
    "new-book-plans/full-society-reader-ledger.md",
    "new-book-plans/placement-exhaustiveness-audit.md",
    "new-book-plans/reader-evidence.md",
    "new-book-plans/record-integrity-assurance-case.md",
    "new-book-plans/record-integrity-red-team.md",
    "new-book-plans/temporal-assurance-case.md",
}
ADMINISTRATIVE_PATHS = {
    "AGENTS.md",
    "CLAUDE.md",
    "README.md",
    "TODO.md",
    PROTOCOL_PATH,
}
AUDIT_TRANSITION_GENERATED_PATHS = {
    "new-book-plans/full-society-ledger.md",
    "new-book-plans/full-society-reader-ledger.md",
}
CLOSURE_TRANSITION_GENERATED_PATHS = {
    *AUDIT_TRANSITION_GENERATED_PATHS,
}
TRANSITION_BYTE_STABLE_GENERATED_PATHS = {
    "new-book-plans/constitutional-closure-and-model-allocation-audit.md",
}


class ReceiptError(RuntimeError):
    """Receipt emission, integrity, or administrative-reuse failure."""


def _sha256(body: bytes) -> str:
    return hashlib.sha256(body).hexdigest()


def _canonical(value: object) -> bytes:
    return json.dumps(
        value, ensure_ascii=False, sort_keys=True, separators=(",", ":")
    ).encode("utf-8")


def _pretty(value: object) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    ).encode("utf-8")


def _utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace(
        "+00:00", "Z"
    )


def _git(
    *args: str,
    root: pathlib.Path | None = None,
    check: bool = True,
    input_bytes: bytes | None = None,
) -> subprocess.CompletedProcess:
    root = root or ROOT
    proc = subprocess.run(
        ["git", "-C", str(root), *args],
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if check and proc.returncode:
        detail = proc.stderr.decode("utf-8", "replace").strip()
        if not detail:
            detail = proc.stdout.decode("utf-8", "replace").strip()
        raise ReceiptError(detail or f"git {' '.join(args)} failed")
    return proc


def _git_text(*args: str, root: pathlib.Path | None = None) -> str:
    root = root or ROOT
    return _git(*args, root=root).stdout.decode("utf-8", "strict").strip()


def _atomic_write(path: pathlib.Path, body: bytes, *, mode: int = 0o644) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f".{path.name}.{os.getpid()}.{uuid.uuid4().hex}.tmp")
    try:
        with tmp.open("xb") as handle:
            handle.write(body)
            handle.flush()
            os.fsync(handle.fileno())
        os.chmod(tmp, mode)
        os.replace(tmp, path)
    finally:
        with contextlib.suppress(FileNotFoundError):
            tmp.unlink()


def _safe_output_directory(path: pathlib.Path) -> pathlib.Path:
    candidate = (path if path.is_absolute() else ROOT / path).absolute()
    expected = RECEIPT_DIR.absolute()
    if candidate != expected:
        raise ReceiptError(
            "receipt output must be new-book-plans/verification-receipts"
        )
    if candidate.exists() and candidate.resolve() != candidate:
        raise ReceiptError("receipt output directory may not be a symbolic link")
    if not candidate.parent.resolve().is_relative_to(ROOT.resolve()):
        raise ReceiptError("receipt output escapes the repository")
    return candidate


def _fully_staged_candidate() -> tuple[str, str, list[dict]]:
    if _git("ls-files", "-u", "-z").stdout:
        raise ReceiptError("the Git index contains unresolved stages")
    unstaged = _git("diff", "--quiet", "--", check=False)
    if unstaged.returncode not in (0, 1):
        raise ReceiptError("cannot inspect unstaged changes")
    if unstaged.returncode:
        raise ReceiptError("receipt emission requires no unstaged tracked changes")
    untracked = _git("ls-files", "--others", "--exclude-standard", "-z").stdout
    if untracked:
        paths = [
            pathlib.PurePosixPath(item.decode("utf-8", "replace")).name
            for item in untracked.rstrip(b"\0").split(b"\0")
        ]
        raise ReceiptError(
            "receipt emission requires no non-ignored untracked files: "
            + ", ".join(paths[:5])
        )
    staged = _git("diff", "--cached", "--quiet", "HEAD", "--", check=False)
    if staged.returncode not in (0, 1):
        raise ReceiptError("cannot inspect staged changes")
    if staged.returncode == 0:
        raise ReceiptError("receipt emission requires a staged candidate")
    check = _git("diff", "--cached", "--check", check=False)
    if check.returncode:
        raise ReceiptError(
            "staged candidate fails git diff --cached --check: "
            + check.stdout.decode("utf-8", "replace").strip()
        )
    parent = _git_text("rev-parse", "HEAD")
    tree = _git_text("write-tree")
    return parent, tree, _index_manifest()


def _index_manifest() -> list[dict]:
    raw = _git("ls-files", "-s", "-z").stdout
    result: list[dict] = []
    for record in raw.rstrip(b"\0").split(b"\0") if raw else []:
        left, path_raw = record.split(b"\t", 1)
        mode_raw, object_raw, stage_raw = left.split(b" ", 2)
        if stage_raw != b"0":
            raise ReceiptError("index contains a non-zero merge stage")
        mode = mode_raw.decode("ascii")
        result.append(
            {
                "path": path_raw.decode("utf-8", "strict"),
                "mode": mode,
                "type": "commit" if mode == "160000" else "blob",
                "object": object_raw.decode("ascii"),
            }
        )
    return sorted(result, key=lambda item: item["path"].encode("utf-8"))


def _tree_manifest(treeish: str) -> list[dict]:
    raw = _git("ls-tree", "-r", "-z", "--full-tree", treeish).stdout
    result: list[dict] = []
    for record in raw.rstrip(b"\0").split(b"\0") if raw else []:
        left, path_raw = record.split(b"\t", 1)
        mode_raw, type_raw, object_raw = left.split(b" ", 2)
        result.append(
            {
                "path": path_raw.decode("utf-8", "strict"),
                "mode": mode_raw.decode("ascii"),
                "type": type_raw.decode("ascii"),
                "object": object_raw.decode("ascii"),
            }
        )
    return sorted(result, key=lambda item: item["path"].encode("utf-8"))


def _blob(object_sha: str) -> bytes:
    return _git("cat-file", "blob", object_sha).stdout


def _manifest_map(manifest: Iterable[dict]) -> dict[str, dict]:
    return {item["path"]: item for item in manifest}


def _blob_at(manifest: Iterable[dict], path: str) -> bytes:
    item = _manifest_map(manifest).get(path)
    if item is None or item["type"] != "blob":
        raise ReceiptError(f"required blob is absent: {path}")
    return _blob(item["object"])


def _classify(path: str) -> str:
    if path in ADMINISTRATIVE_PATHS or path.startswith(
        "new-book-plans/verification-receipts/"
    ):
        return "administrative"
    if path in GENERATED_PATHS:
        return "generated-artifact"
    if (
        path == "verify.sh"
        or path.endswith(".sh")
        or path == "registry/check.py"
        or (path.startswith("new-book-plans/") and path.endswith(".py"))
    ):
        return "verifier-input"
    if (
        path.endswith(".pins.nibli")
        or "/counterfactual/" in path
        or "/fixtures/" in path
        or "/testdata/" in path
    ):
        return "fixture"
    return "source"


def _classified_manifest(manifest: list[dict]) -> tuple[list[dict], dict]:
    expanded = [{**item, "class": _classify(item["path"])} for item in manifest]
    classes: dict[str, dict] = {}
    for name in (
        "source",
        "verifier-input",
        "fixture",
        "generated-artifact",
        "administrative",
    ):
        rows = [
            {
                "path": item["path"],
                "mode": item["mode"],
                "type": item["type"],
                "object": item["object"],
            }
            for item in expanded
            if item["class"] == name
        ]
        classes[name] = {
            "count": len(rows),
            "sha256": _sha256(_canonical(rows)),
        }
    return expanded, classes


def _sanitized_environment() -> dict:
    values: dict[str, str] = {}
    for key in (
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "TZ",
        "PYTHONHASHSEED",
        "SOURCE_DATE_EPOCH",
        "STATE_FORM_MAX_PARALLEL",
    ):
        if key in os.environ:
            values[key] = os.environ[key]
    hashed_values = {}
    for key in ("PATH", "NIBLI_PIN", "NIBLI_SRC"):
        if key in os.environ:
            hashed_values[key] = _sha256(os.environ[key].encode("utf-8"))
    tools = {}
    for name, command in (
        ("git", ["git", "--version"]),
        ("python", [sys.executable, "--version"]),
        ("bash", ["bash", "--version"]),
    ):
        proc = subprocess.run(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            check=False,
            timeout=10,
        )
        first = proc.stdout.decode("utf-8", "replace").splitlines()
        tools[name] = first[0] if first else f"exit-{proc.returncode}"
    details = {
        "allowlisted_values": values,
        "hashed_values": hashed_values,
        "platform": {
            "system": platform.system(),
            "release": platform.release(),
            "machine": platform.machine(),
        },
        "tools": tools,
    }
    return {"details": details, "sha256": _sha256(_canonical(details))}


def _engine_paths() -> tuple[pathlib.Path, pathlib.Path]:
    source = pathlib.Path(
        os.environ.get(
            "NIBLI_SRC",
            str(pathlib.Path.home() / "projects/dhilipsiva/nibli"),
        )
    ).resolve()
    binary = pathlib.Path(
        os.environ.get("NIBLI_PIN", str(source / "target/release/nibli-pin"))
    ).resolve()
    return source, binary


def _engine_identity(*, require_binary: bool = True) -> dict:
    source, binary = _engine_paths()
    identity = {
        "binary_basename": binary.name,
        "binary_path_sha256": _sha256(os.fsencode(binary)),
        "source_override": "NIBLI_PIN" in os.environ,
        "source_available": (source / ".git").exists()
        or bool(_git("-C", str(source), "rev-parse", "--git-dir", check=False).returncode == 0),
    }
    probe = _git("-C", str(source), "rev-parse", "HEAD", check=False)
    if probe.returncode == 0:
        identity["source_commit_sha"] = probe.stdout.decode("ascii").strip()
        status = _git("-C", str(source), "status", "--porcelain=v1", "-z", check=False)
        if status.returncode:
            raise ReceiptError("cannot inspect Nibli source worktree")
        diff = _git(
            "-C", str(source), "diff", "--binary", "HEAD", "--", check=False
        )
        untracked = _git(
            "-C", str(source), "ls-files", "--others",
            "--exclude-standard", "-z", check=False,
        )
        if diff.returncode or untracked.returncode:
            raise ReceiptError("cannot bind Nibli source worktree bytes")
        untracked_rows = []
        raw_paths = (
            untracked.stdout.rstrip(b"\0").split(b"\0")
            if untracked.stdout else []
        )
        for raw_path in raw_paths:
            relative = raw_path.decode("utf-8", "strict")
            item = source / relative
            metadata = item.lstat()
            if item.is_symlink():
                body = os.fsencode(os.readlink(item))
                kind = "symlink"
            elif item.is_file():
                body = item.read_bytes()
                kind = "file"
            else:
                raise ReceiptError("unsupported untracked Nibli source entry")
            untracked_rows.append({
                "path": relative,
                "kind": kind,
                "mode": metadata.st_mode & 0o177777,
                "sha256": _sha256(body),
            })
        identity["source_dirty"] = bool(status.stdout)
        identity["source_status_sha256"] = _sha256(status.stdout)
        identity["source_diff_sha256"] = _sha256(diff.stdout)
        identity["source_untracked_count"] = len(untracked_rows)
        identity["source_untracked_sha256"] = _sha256(
            _canonical(untracked_rows)
        )
    else:
        identity["source_commit_sha"] = None
        identity["source_dirty"] = None
        identity["source_status_sha256"] = None
        identity["source_diff_sha256"] = None
        identity["source_untracked_count"] = None
        identity["source_untracked_sha256"] = None
    if binary.is_file():
        binary_body = binary.read_bytes()
        identity["binary_sha256"] = _sha256(binary_body)
        identity["binary_size"] = len(binary_body)
    elif require_binary:
        raise ReceiptError(
            "nibli-pin binary is missing after authoritative verification"
        )
    return identity


def _ledger_receipt_context(manifest: list[dict]) -> tuple[str, str]:
    try:
        source = json.loads(_blob_at(manifest, LEDGER_PATH).decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ReceiptError("staged full-society ledger is not valid UTF-8 JSON") from exc
    value = source.get("source_version") if isinstance(source, dict) else None
    if not isinstance(value, str) or not value:
        raise ReceiptError("staged full-society ledger has no source_version")
    audits = source.get("scope_audits")
    pending = audits[-1] if isinstance(audits, list) and audits else None
    pending_id = pending.get("id") if isinstance(pending, dict) else None
    match = re.fullmatch(r"(FS-SAU-[0-9]+)-PENDING", str(pending_id))
    if (
        match is None
        or pending.get("result") != "pending"
        or pending.get("source_version") != value
    ):
        raise ReceiptError(
            "staged ledger must end in one current-source pending FS-SAU audit"
        )
    return value, match.group(1)


def _check_protocol_v5(manifest: list[dict]) -> None:
    try:
        body = _blob_at(manifest, PROTOCOL_PATH).decode("utf-8")
    except UnicodeError as exc:
        raise ReceiptError("scope-review protocol is not UTF-8") from exc
    if PROTOCOL_STATUS not in body:
        raise ReceiptError("staged candidate does not publish protocol-v5 status")


def _command_binding(
    command: Sequence[str], *, expected: Sequence[str] = ("./verify.sh",)
) -> dict:
    if not command:
        raise ReceiptError("receipt emission requires an authoritative command")
    resolved = pathlib.Path(command[0])
    if not resolved.is_absolute():
        resolved = (ROOT / resolved).resolve()
    else:
        resolved = resolved.resolve()
    if (
        resolved != (ROOT / "verify.sh").resolve()
        or list(command) != list(expected)
    ):
        raise ReceiptError(
            "verification command does not match the required exact mode"
        )
    return {
        "display": " ".join(expected),
        "argv_sha256": _sha256(_canonical(list(expected))),
    }


def _run_captured(
    command: Sequence[str],
    *,
    expected: Sequence[str] = ("./verify.sh",),
    lock_fd: int | None = None,
) -> tuple[int, bytes, dict]:
    binding = _command_binding(command, expected=expected)
    started = _utc_now()
    start_mono = time.monotonic()
    pass_fds = (lock_fd,) if lock_fd is not None else ()
    managed = (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)
    old_mask = signal.pthread_sigmask(signal.SIG_BLOCK, managed)
    try:
        child = subprocess.Popen(
            list(command), cwd=ROOT, stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, pass_fds=pass_fds,
            start_new_session=True,
        )
    except BaseException:
        signal.pthread_sigmask(signal.SIG_SETMASK, old_mask)
        raise
    transcript = bytearray()
    prior: dict[int, object] = {}

    def forward(signum, _frame):
        with contextlib.suppress(ProcessLookupError):
            os.killpg(child.pid, signum)

    for signum in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
        prior[signum] = signal.signal(signum, forward)
    signal.pthread_sigmask(signal.SIG_SETMASK, old_mask)
    try:
        assert child.stdout is not None
        while True:
            chunk = child.stdout.read(65536)
            if not chunk:
                break
            transcript.extend(chunk)
            sys.stdout.buffer.write(chunk)
            sys.stdout.buffer.flush()
        returncode = child.wait()
    finally:
        if child.stdout is not None:
            child.stdout.close()
        for signum, handler in prior.items():
            signal.signal(signum, handler)
    finished = _utc_now()
    result = {
        **binding,
        "started_at_utc": started,
        "finished_at_utc": finished,
        "elapsed_milliseconds": int((time.monotonic() - start_mono) * 1000),
        "exit_code": returncode,
    }
    return returncode, bytes(transcript), result


def _receipt_digest(receipt: dict) -> str:
    candidate = copy.deepcopy(receipt)
    candidate.pop("receipt_id", None)
    return _sha256(_canonical(candidate))


def emit_receipt(
    output_directory: pathlib.Path,
    command: Sequence[str],
    *,
    wait_seconds: float = 0,
    _held_lock: VerificationLock | None = None,
) -> pathlib.Path:
    output = _safe_output_directory(output_directory)
    if _held_lock is None:
        try:
            with VerificationLock(
                "verify",
                wait_seconds=wait_seconds,
                engine_path=_engine_paths()[1],
                command=command,
                root=ROOT,
            ) as held:
                return emit_receipt(
                    output,
                    command,
                    wait_seconds=0,
                    _held_lock=held,
                )
        except VerificationLockBusy:
            raise
        except VerificationLockError as exc:
            raise ReceiptError(str(exc)) from exc
    parent, tree, raw_manifest = _fully_staged_candidate()
    expanded_manifest, class_manifests = _classified_manifest(raw_manifest)
    manifest_sha = _sha256(_canonical(raw_manifest))
    source_version, audit_id = _ledger_receipt_context(raw_manifest)
    if source_version == LEGACY_SOURCE_VERSION:
        raise ReceiptError(
            "the allowlisted legacy source may not be relabelled as receipt v2"
        )
    _check_protocol_v5(raw_manifest)
    environment = _sanitized_environment()
    common = git_common_dir(ROOT)

    try:
        with VerificationLock(
            "verify",
            wait_seconds=wait_seconds,
            source_digest=manifest_sha,
            engine_path=_engine_paths()[1],
            command=command,
            root=ROOT,
        ) as held:
            initial_engine = _engine_identity(require_binary=False)
            exit_code, transcript, command_result = _run_captured(
                command,
                lock_fd=(
                    _held_lock.lock_fd if _held_lock is not None else held.lock_fd
                ),
            )
            if exit_code:
                failed_dir = common / EVIDENCE_SUBDIR / (
                    "failed-" + tree + "-" + uuid.uuid4().hex
                )
                failed_dir.mkdir(parents=True, exist_ok=False)
                _atomic_write(failed_dir / "transcript.log", transcript, mode=0o600)
                raise ReceiptError(
                    f"authoritative verification failed with exit {exit_code}; "
                    f"diagnostic transcript retained under the Git common directory"
                )
            after_parent, after_tree, after_manifest = _fully_staged_candidate()
            if (after_parent, after_tree, after_manifest) != (
                parent,
                tree,
                raw_manifest,
            ):
                raise ReceiptError("repository inputs drifted during verification")
            engine = _engine_identity()
            engine_source = {
                key: value for key, value in engine.items()
                if key not in {"binary_sha256", "binary_size"}
            }
            initial_engine_source = {
                key: value for key, value in initial_engine.items()
                if key not in {"binary_sha256", "binary_size"}
            }
            if engine_source != initial_engine_source:
                raise ReceiptError(
                    "Nibli source or selected binary path drifted during verification"
                )
            final_environment = _sanitized_environment()
            if final_environment != environment:
                raise ReceiptError("sanitized verification environment drifted")

            transcript_sha = _sha256(transcript)
            expanded = {
                "schema_version": 2,
                "protocol_version": PROTOCOL_VERSION,
                "protocol_status": PROTOCOL_STATUS,
                "source_version": source_version,
                "audit_id": audit_id,
                "candidate": {
                    "parent_commit_sha": parent,
                    "tree_sha": tree,
                    "path_manifest_sha256": manifest_sha,
                    "path_manifest": expanded_manifest,
                    "class_manifests": class_manifests,
                },
                "verification": {
                    "commands": [command_result],
                    "result": "all-passed",
                    "transcript_sha256": transcript_sha,
                },
                "engine": engine,
                "environment": environment,
                "evidence_ceiling": EVIDENCE_CEILING,
            }
            expanded_bytes = _pretty(expanded)
            compact = {
                "spdx": "CC0-1.0",
                "schema_version": 2,
                "protocol_version": PROTOCOL_VERSION,
                "protocol_status": PROTOCOL_STATUS,
                "receipt_id": "",
                "status": "all-passed",
                "source_version": source_version,
                "audit_id": audit_id,
                "candidate": {
                    "parent_commit_sha": parent,
                    "tree_sha": tree,
                    "path_manifest_sha256": manifest_sha,
                    "path_count": len(raw_manifest),
                    "class_manifests": class_manifests,
                },
                "verification": {
                    "command_sha256": command_result["argv_sha256"],
                    "transcript_sha256": transcript_sha,
                    "started_at_utc": command_result["started_at_utc"],
                    "finished_at_utc": command_result["finished_at_utc"],
                    "results": [
                        {
                            "command": command_result["display"],
                            "exit_code": 0,
                        }
                    ],
                },
                "engine": engine,
                "environment": {
                    "sha256": environment["sha256"],
                    "fields": sorted(
                        set(environment["details"]["allowlisted_values"])
                        | set(environment["details"]["hashed_values"])
                    ),
                },
                "local_evidence": {
                    "expanded_manifest_sha256": _sha256(expanded_bytes),
                    "transcript_sha256": transcript_sha,
                },
                "evidence_ceiling": expanded["evidence_ceiling"],
            }
            compact["receipt_id"] = _receipt_digest(compact)
            receipt_id = compact["receipt_id"]
            evidence_dir = common / EVIDENCE_SUBDIR / f"sha256-{receipt_id}"
            if evidence_dir.exists():
                raise ReceiptError("local evidence directory already exists")
            evidence_dir.mkdir(parents=True, mode=0o700)
            _atomic_write(
                evidence_dir / "expanded-manifest.json",
                expanded_bytes,
                mode=0o600,
            )
            _atomic_write(evidence_dir / "transcript.log", transcript, mode=0o600)
            _atomic_write(
                evidence_dir / "command-results.json",
                _pretty(expanded["verification"]),
                mode=0o600,
            )
            output.mkdir(parents=True, exist_ok=True)
            receipt_path = output / f"sha256-{receipt_id}.json"
            receipt_bytes = _pretty(compact)
            if receipt_path.exists():
                if receipt_path.read_bytes() != receipt_bytes:
                    raise ReceiptError("content-addressed receipt path collision")
            else:
                _atomic_write(receipt_path, receipt_bytes)
            print(receipt_path.relative_to(ROOT).as_posix())
            return receipt_path
    except VerificationLockBusy:
        raise
    except VerificationLockError as exc:
        raise ReceiptError(str(exc)) from exc


def _load_json(path: pathlib.Path, *, raw_bytes: bytes | None = None) -> dict:
    try:
        if raw_bytes is None:
            raw_bytes = path.read_bytes()
        elif not isinstance(raw_bytes, bytes):
            raise ReceiptError("cached receipt content must be exact bytes")
        value = json.loads(raw_bytes.decode("utf-8"))
    except ReceiptError:
        raise
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ReceiptError(f"cannot read receipt {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise ReceiptError("receipt root must be an object")
    return value


def _validate_legacy_v1(
    receipt: dict,
    *,
    source_version: str | None,
    audit_id: str | None,
) -> dict:
    expected_context = (LEGACY_SOURCE_VERSION, LEGACY_AUDIT_ID)
    if (source_version, audit_id) != expected_context:
        raise ReceiptError("legacy receipt is outside the one exact v1 allowlist")
    if receipt.get("candidate_commit_sha") != LEGACY_CANDIDATE:
        raise ReceiptError("legacy candidate is not allowlisted")
    if receipt.get("transcript_sha256") != LEGACY_TRANSCRIPT_SHA256:
        raise ReceiptError("legacy transcript digest is not allowlisted")
    if receipt.get("result") != "all-passed":
        raise ReceiptError("legacy receipt did not pass")
    commands = receipt.get("commands")
    if (
        not isinstance(commands, list)
        or commands != list(LEGACY_REQUIRED_COMMANDS)
    ):
        raise ReceiptError("legacy receipt command list is not the preserved full run")
    return receipt


def _require_sha256(value: object, context: str) -> str:
    if not isinstance(value, str) or not _HEX64_RE.fullmatch(value):
        raise ReceiptError(f"{context} must be lowercase SHA-256")
    return value


def _require_utc(value: object, context: str) -> dt.datetime:
    if not isinstance(value, str) or not _UTC_RE.fullmatch(value):
        raise ReceiptError(f"{context} must be canonical UTC seconds")
    try:
        return dt.datetime.fromisoformat(value[:-1] + "+00:00")
    except ValueError as exc:
        raise ReceiptError(f"{context} is not a valid UTC timestamp") from exc


def _validate_compact_schema(receipt: dict, path: pathlib.Path) -> None:
    expected_keys = {
        "spdx",
        "schema_version",
        "protocol_version",
        "protocol_status",
        "receipt_id",
        "status",
        "source_version",
        "audit_id",
        "candidate",
        "verification",
        "engine",
        "environment",
        "local_evidence",
        "evidence_ceiling",
    }
    if set(receipt) != expected_keys:
        raise ReceiptError("receipt does not use the exact schema-v2 top-level keys")
    if receipt["spdx"] != "CC0-1.0" or receipt["schema_version"] != 2:
        raise ReceiptError("receipt licence/schema must be CC0-1.0 / version 2")
    if (
        receipt["protocol_version"] != PROTOCOL_VERSION
        or receipt["protocol_status"] != PROTOCOL_STATUS
    ):
        raise ReceiptError("receipt does not bind current protocol v5")
    if receipt["status"] != "all-passed":
        raise ReceiptError("receipt is not passing")
    receipt_id = receipt.get("receipt_id")
    if not isinstance(receipt_id, str) or not _HEX64_RE.fullmatch(receipt_id):
        raise ReceiptError("receipt_id must be lowercase SHA-256")
    if _receipt_digest(receipt) != receipt_id:
        raise ReceiptError("receipt self digest does not match receipt_id")
    match = _RECEIPT_NAME_RE.fullmatch(path.name)
    if not match or match.group(1) != receipt_id:
        raise ReceiptError("receipt filename does not match its self digest")
    candidate = receipt.get("candidate")
    if not isinstance(candidate, dict) or set(candidate) != {
        "parent_commit_sha",
        "tree_sha",
        "path_manifest_sha256",
        "path_count",
        "class_manifests",
    }:
        raise ReceiptError("receipt candidate binding has the wrong schema")
    for key in ("parent_commit_sha", "tree_sha"):
        if not isinstance(candidate[key], str) or not _HEX40_RE.fullmatch(
            candidate[key]
        ):
            raise ReceiptError(f"candidate {key} must be a Git SHA-1")
    if not _HEX64_RE.fullmatch(str(candidate["path_manifest_sha256"])):
        raise ReceiptError("candidate path manifest digest is invalid")
    if not isinstance(candidate["path_count"], int) or candidate["path_count"] < 1:
        raise ReceiptError("candidate path count is invalid")
    verification = receipt.get("verification")
    if not isinstance(verification, dict) or verification.get("results") != [
        {"command": "./verify.sh", "exit_code": 0}
    ]:
        raise ReceiptError("receipt does not bind one passing full verifier")
    if verification.get("command_sha256") != _sha256(_canonical(["./verify.sh"])):
        raise ReceiptError("receipt command digest is not the full verifier")
    if verification.get("transcript_sha256") != receipt.get(
        "local_evidence", {}
    ).get("transcript_sha256"):
        raise ReceiptError("receipt transcript bindings disagree")
    _validate_compact_details(receipt)


def _validate_compact_details(receipt: dict) -> None:
    if not isinstance(receipt["source_version"], str) or not receipt["source_version"]:
        raise ReceiptError("receipt source_version must be a nonempty string")
    if not isinstance(receipt["audit_id"], str) or re.fullmatch(
        r"FS-SAU-[0-9]+", receipt["audit_id"]
    ) is None:
        raise ReceiptError("receipt audit_id must be a canonical FS-SAU id")

    if receipt["evidence_ceiling"] != EVIDENCE_CEILING:
        raise ReceiptError("receipt evidence ceiling is not byte-exact")
    candidate = receipt["candidate"]
    _require_sha256(candidate["path_manifest_sha256"], "candidate path manifest")
    classes = candidate["class_manifests"]
    class_names = {
        "source", "verifier-input", "fixture", "generated-artifact",
        "administrative",
    }
    if not isinstance(classes, dict) or set(classes) != class_names:
        raise ReceiptError("candidate class manifests have the wrong schema")
    class_count = 0
    for name in sorted(class_names):
        row = classes[name]
        if not isinstance(row, dict) or set(row) != {"count", "sha256"}:
            raise ReceiptError(f"candidate class manifest is malformed: {name}")
        if not isinstance(row["count"], int) or row["count"] < 0:
            raise ReceiptError(f"candidate class count is invalid: {name}")
        _require_sha256(row["sha256"], f"candidate class digest {name}")
        class_count += row["count"]
    if class_count != candidate["path_count"]:
        raise ReceiptError("candidate class counts do not cover the path manifest")
    verification = receipt["verification"]
    if not isinstance(verification, dict) or set(verification) != {
        "command_sha256", "transcript_sha256", "started_at_utc",
        "finished_at_utc", "results",
    }:
        raise ReceiptError("receipt verification binding has the wrong schema")
    _require_sha256(verification["transcript_sha256"], "verification transcript")
    started = _require_utc(verification["started_at_utc"], "verification start")
    finished = _require_utc(verification["finished_at_utc"], "verification finish")
    if finished < started:
        raise ReceiptError("verification finish predates its start")
    local = receipt["local_evidence"]
    if not isinstance(local, dict) or set(local) != {
        "expanded_manifest_sha256", "transcript_sha256",
    }:
        raise ReceiptError("local evidence binding has the wrong schema")
    _require_sha256(local["expanded_manifest_sha256"], "expanded manifest")
    _require_sha256(local["transcript_sha256"], "local transcript")
    environment = receipt["environment"]
    if not isinstance(environment, dict) or set(environment) != {"sha256", "fields"}:
        raise ReceiptError("receipt environment binding has the wrong schema")
    _require_sha256(environment["sha256"], "environment")
    fields = environment["fields"]
    if (
        not isinstance(fields, list)
        or any(not isinstance(value, str) or not value for value in fields)
        or fields != sorted(set(fields))
    ):
        raise ReceiptError("receipt environment fields must be sorted and unique")
    engine = receipt["engine"]
    engine_keys = {
        "binary_basename", "binary_path_sha256", "binary_sha256", "binary_size",
        "source_override", "source_available", "source_commit_sha", "source_dirty",
        "source_status_sha256", "source_diff_sha256", "source_untracked_count",
        "source_untracked_sha256",
    }
    if not isinstance(engine, dict) or set(engine) != engine_keys:
        raise ReceiptError("receipt engine binding has the wrong schema")
    if not isinstance(engine["binary_basename"], str) or not engine["binary_basename"]:
        raise ReceiptError("receipt engine binary basename is invalid")
    for key in ("binary_path_sha256", "binary_sha256"):
        _require_sha256(engine[key], f"engine {key}")
    if not isinstance(engine["binary_size"], int) or engine["binary_size"] < 1:
        raise ReceiptError("receipt engine binary size is invalid")
    if not isinstance(engine["source_override"], bool) or not isinstance(
        engine["source_available"], bool
    ):
        raise ReceiptError("receipt engine source flags are invalid")
    if engine["source_commit_sha"] is not None and not _HEX40_RE.fullmatch(
        str(engine["source_commit_sha"])
    ):
        raise ReceiptError("receipt engine source commit is invalid")
    for key in (
        "source_status_sha256", "source_diff_sha256", "source_untracked_sha256",
    ):
        if engine[key] is not None:
            _require_sha256(engine[key], f"engine {key}")
    if engine["source_dirty"] is not None and not isinstance(
        engine["source_dirty"], bool
    ):
        raise ReceiptError("receipt engine dirty flag is invalid")
    if engine["source_untracked_count"] is not None and (
        not isinstance(engine["source_untracked_count"], int)
        or engine["source_untracked_count"] < 0
    ):
        raise ReceiptError("receipt engine untracked count is invalid")


def load_and_validate_receipt(
    path: pathlib.Path,
    *,
    require_local: bool = True,
    check_environment: bool = True,
    check_engine: bool = True,
    source_version: str | None = None,
    audit_id: str | None = None,
    root: pathlib.Path | None = None,
    raw_bytes: bytes | None = None,
) -> dict:
    root = root or ROOT
    path = path if path.is_absolute() else root / path
    receipt = _load_json(path, raw_bytes=raw_bytes)
    schema = receipt.get("schema_version")
    if schema in (None, 1):
        return _validate_legacy_v1(
            receipt, source_version=source_version, audit_id=audit_id
        )
    if schema != 2:
        raise ReceiptError("unknown receipt schema; downgrade is forbidden")
    _validate_compact_schema(receipt, path)
    if receipt["source_version"] == LEGACY_SOURCE_VERSION:
        raise ReceiptError("the legacy source may not be relabelled as receipt v2")

    common = git_common_dir(root)
    evidence_dir = (
        common / EVIDENCE_SUBDIR / f"sha256-{receipt['receipt_id']}"
    )
    if require_local:
        expanded_path = evidence_dir / "expanded-manifest.json"
        transcript_path = evidence_dir / "transcript.log"
        results_path = evidence_dir / "command-results.json"
        if not all(path.is_file() for path in (
            expanded_path, transcript_path, results_path
        )):
            raise ReceiptError("local receipt evidence is missing")
        expanded_bytes = expanded_path.read_bytes()
        transcript = transcript_path.read_bytes()
        results_bytes = results_path.read_bytes()
        if _sha256(expanded_bytes) != receipt["local_evidence"][
            "expanded_manifest_sha256"
        ]:
            raise ReceiptError("expanded manifest digest mismatch")
        if _sha256(transcript) != receipt["verification"]["transcript_sha256"]:
            raise ReceiptError("transcript digest mismatch")
        try:
            expanded = json.loads(expanded_bytes.decode("utf-8"))
        except (UnicodeError, json.JSONDecodeError) as exc:
            raise ReceiptError("expanded manifest is invalid") from exc
        if not isinstance(expanded, dict) or set(expanded) != {
            "schema_version", "protocol_version", "protocol_status",
            "source_version", "audit_id", "candidate", "verification", "engine",
            "environment", "evidence_ceiling",
        }:
            raise ReceiptError("expanded manifest has the wrong schema")
        if (
            expanded["schema_version"] != 2
            or expanded["protocol_version"] != receipt["protocol_version"]
            or expanded["protocol_status"] != receipt["protocol_status"]
            or expanded["source_version"] != receipt["source_version"]
            or expanded["audit_id"] != receipt["audit_id"]
            or expanded["evidence_ceiling"] != receipt["evidence_ceiling"]
        ):
            raise ReceiptError("expanded and compact protocol bindings disagree")
        expanded_verification = expanded.get("verification")
        if not isinstance(expanded_verification, dict) or _pretty(
            expanded_verification
        ) != results_bytes:
            raise ReceiptError("command-results evidence is not the exact manifest record")
        commands = expanded_verification.get("commands")
        if (
            expanded_verification.get("result") != "all-passed"
            or not isinstance(commands, list)
            or len(commands) != 1
            or commands[0].get("argv_sha256")
            != receipt["verification"]["command_sha256"]
        ):
            raise ReceiptError("expanded and compact command bindings disagree")

        path_manifest = expanded.get("candidate", {}).get("path_manifest")
        if not isinstance(path_manifest, list):
            raise ReceiptError("expanded path manifest is missing")
        raw_manifest = [
            {
                key: item[key]
                for key in ("path", "mode", "type", "object")
            }
            for item in path_manifest
        ]
        candidate = receipt["candidate"]
        if (
            _sha256(_canonical(raw_manifest))
            != candidate["path_manifest_sha256"]
            or len(raw_manifest) != candidate["path_count"]
        ):
            raise ReceiptError("expanded and compact path manifests disagree")
        git_manifest = _tree_manifest(candidate["tree_sha"])
        expanded_candidate = expanded["candidate"]
        classified_manifest, class_manifests = _classified_manifest(raw_manifest)
        if path_manifest != classified_manifest:
            raise ReceiptError("expanded path classifications are not deterministic")
        if (
            not isinstance(expanded_candidate, dict)
            or set(expanded_candidate) != {
                "parent_commit_sha", "tree_sha", "path_manifest_sha256",
                "path_manifest", "class_manifests",
            }
            or expanded_candidate["parent_commit_sha"]
            != candidate["parent_commit_sha"]
            or expanded_candidate["tree_sha"] != candidate["tree_sha"]
            or expanded_candidate["path_manifest_sha256"]
            != candidate["path_manifest_sha256"]
            or expanded_candidate["class_manifests"] != class_manifests
            or candidate["class_manifests"] != class_manifests
        ):
            raise ReceiptError("expanded and compact candidate bindings disagree")
        if git_manifest != raw_manifest:
            raise ReceiptError("candidate Git tree no longer matches path manifest")
        if expanded.get("engine") != receipt["engine"]:
            raise ReceiptError("expanded and compact engine identities disagree")
        if expanded.get("environment", {}).get("sha256") != receipt[
            "environment"
        ].get("sha256"):
            raise ReceiptError("expanded and compact environment bindings disagree")
    if check_environment:
        current_environment = _sanitized_environment()
        if current_environment["sha256"] != receipt["environment"].get("sha256"):
            raise ReceiptError("sanitized environment drifted from receipt")
    if check_engine and _engine_identity() != receipt["engine"]:
        raise ReceiptError("Nibli binary or source identity drifted from receipt")
    return receipt


def _parents(commit: str) -> list[str]:
    fields = _git_text("rev-list", "--parents", "-n", "1", commit).split()
    if not fields or fields[0] != commit:
        raise ReceiptError(f"cannot inspect commit ancestry: {commit}")
    return fields[1:]


def _require_single_parent(commit: str, expected_parent: str) -> None:
    parents = _parents(commit)
    if parents != [expected_parent]:
        raise ReceiptError("merge or intervening commit invalidates receipt reuse")


def _changed_paths(old_manifest: list[dict], new_manifest: list[dict]) -> dict:
    old = _manifest_map(old_manifest)
    new = _manifest_map(new_manifest)
    result = {}
    for path in sorted(set(old) | set(new)):
        if old.get(path) != new.get(path):
            result[path] = {"old": old.get(path), "new": new.get(path)}
    return result


def _json_at(manifest: list[dict], path: str) -> dict:
    try:
        value = json.loads(_blob_at(manifest, path).decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ReceiptError(f"{path} is not valid UTF-8 JSON") from exc
    if not isinstance(value, dict):
        raise ReceiptError(f"{path} must contain a JSON object")
    return value


def _without_keys(value: dict, keys: set[str]) -> dict:
    return {key: item for key, item in value.items() if key not in keys}


def _receipt_relative(path: pathlib.Path) -> str:
    resolved = path.resolve()
    try:
        relative = resolved.relative_to(ROOT.resolve()).as_posix()
    except ValueError as exc:
        raise ReceiptError("receipt must be inside the repository") from exc
    if not relative.startswith("new-book-plans/verification-receipts/"):
        raise ReceiptError("receipt must be in the tracked receipt directory")
    return relative


def _validate_modes(
    changes: dict, *, receipt_path: str | None = None
) -> None:
    for path, change in changes.items():
        old, new = change["old"], change["new"]
        if old is None:
            if path != receipt_path or new.get("mode") != "100644":
                raise ReceiptError(f"unauthorised added path or mode: {path}")
        elif new is None or old["mode"] != new["mode"] or old["type"] != new["type"]:
            raise ReceiptError(f"mode, type, or deletion is not administrative: {path}")


def _expected_passing_audit(
    pending: object, receipt: dict, receipt_path: str
) -> dict:
    required = {
        "id", "title", "source_version", "scope_sha256",
        "protocol_sha256", "executed_at_utc", "method",
        "criterion_coverage", "control_refs", "commands",
        "finding_refs", "result", "policy_basis", "evidence_ceiling",
    }
    if not isinstance(pending, dict) or set(pending) != required:
        raise ReceiptError(
            "verified candidate must end in one exact protocol-v5 pending audit"
        )
    match = re.fullmatch(r"(FS-SAU-[0-9]+)-PENDING", pending.get("id", ""))
    if match is None or pending.get("result") != "pending":
        raise ReceiptError("candidate audit is not the next pending FS-SAU row")
    if match.group(1) != receipt.get("audit_id"):
        raise ReceiptError("pending audit id does not match the receipt")
    if pending.get("source_version") != receipt.get("source_version"):
        raise ReceiptError("pending audit source does not match the receipt")
    title = pending.get("title")
    if not isinstance(title, str) or title.count("pending") != 1:
        raise ReceiptError("pending audit title must identify its pending state")
    commands = pending.get("commands")
    if not isinstance(commands, list) or not commands:
        raise ReceiptError("pending audit command chain is missing")
    verification = receipt.get("verification")
    finished = (
        verification.get("finished_at_utc")
        if isinstance(verification, dict) else None
    )
    if not isinstance(finished, str):
        raise ReceiptError("receipt has no verification completion timestamp")
    expected = copy.deepcopy(pending)
    expected.update({
        "id": match.group(1),
        "title": title.replace("pending", "passing", 1),
        "executed_at_utc": finished,
        "commands": [
            *commands,
            f"./verify.sh --commit-gate {receipt_path} --transition audit",
        ],
        "result": "passed-with-recorded-limits",
        "verification_receipt_ref": receipt_path,
    })
    return expected


def _validate_audit_transition(
    old_manifest: list[dict],
    new_manifest: list[dict],
    receipt: dict,
    receipt_path: str,
) -> None:
    changes = _changed_paths(old_manifest, new_manifest)
    allowed = {
        LEDGER_PATH,
        receipt_path,
        *AUDIT_TRANSITION_GENERATED_PATHS,
    }
    if set(changes) != allowed:
        raise ReceiptError(
            "audit transition must change exactly the audit source, receipt, "
            "and deterministic Script 13 projections"
        )
    _validate_modes(changes, receipt_path=receipt_path)
    tracked_receipt = _blob_at(new_manifest, receipt_path)
    if tracked_receipt != _pretty(receipt):
        raise ReceiptError("tracked receipt bytes differ from validated receipt")
    old = _json_at(old_manifest, LEDGER_PATH)
    new = _json_at(new_manifest, LEDGER_PATH)
    if old.get("source_version") != receipt.get("source_version"):
        raise ReceiptError("candidate ledger source does not match the receipt")
    if _without_keys(old, {"scope_audits"}) != _without_keys(
        new, {"scope_audits"}
    ):
        raise ReceiptError("audit transition changed non-audit ledger values")
    old_audits = old.get("scope_audits")
    new_audits = new.get("scope_audits")
    if (
        not isinstance(old_audits, list)
        or not old_audits
        or not isinstance(new_audits, list)
        or new_audits[:-1] != old_audits
        or len(new_audits) != len(old_audits) + 1
    ):
        raise ReceiptError("audit history must be exact-prefix append-only")
    audit = new_audits[-1]
    expected = _expected_passing_audit(
        old_audits[-1], receipt, receipt_path
    )
    if audit != expected:
        raise ReceiptError(
            "passing audit is not the exact pending-row/receipt derivation"
        )


def _contains_string(value: object, expected: str) -> bool:
    if isinstance(value, str):
        return value == expected
    if isinstance(value, list):
        return any(_contains_string(item, expected) for item in value)
    if isinstance(value, dict):
        return any(_contains_string(item, expected) for item in value.values())
    return False


def _prior_closed_ledger(receipt: dict) -> dict:
    candidate = receipt.get("candidate")
    parent = (
        candidate.get("parent_commit_sha")
        if isinstance(candidate, dict) else None
    )
    if not isinstance(parent, str):
        raise ReceiptError("receipt has no predecessor commit binding")
    prior = _json_at(_tree_manifest(parent), LEDGER_PATH)
    closure = prior.get("closure_record")
    gate = prior.get("acceptance_gate")
    if not isinstance(closure, dict) or not isinstance(gate, dict):
        raise ReceiptError("receipt predecessor is not a closed Gate A source")
    if gate.get("gate_a_status") != "passed":
        raise ReceiptError("receipt predecessor Gate A was not passed")
    return prior


def _validate_closure_transition(
    old_manifest: list[dict],
    new_manifest: list[dict],
    receipt: dict,
    receipt_path: str,
    audit_commit_sha: str,
) -> None:
    changes = _changed_paths(old_manifest, new_manifest)
    allowed = {LEDGER_PATH, *CLOSURE_TRANSITION_GENERATED_PATHS}
    if set(changes) != allowed:
        raise ReceiptError(
            "closure transition must change exactly closure source and "
            "deterministic Script 13 projections"
        )
    _validate_modes(changes)
    old = _json_at(old_manifest, LEDGER_PATH)
    new = _json_at(new_manifest, LEDGER_PATH)
    mutable = {"closure_record", "acceptance_gate"}
    if _without_keys(old, mutable) != _without_keys(new, mutable):
        raise ReceiptError("closure transition changed non-closure ledger values")
    if old.get("source_version") != receipt.get("source_version"):
        raise ReceiptError("closure source does not match the receipt")
    audits = old.get("scope_audits")
    if not isinstance(audits, list) or len(audits) < 2:
        raise ReceiptError("closure predecessor has no passing current audit")
    audit = audits[-1]
    expected_audit = _expected_passing_audit(
        audits[-2], receipt, receipt_path
    )
    if audit != expected_audit:
        raise ReceiptError("closure predecessor audit is not receipt-derived")
    prior = _prior_closed_ledger(receipt)
    prior_closure = prior["closure_record"]
    prior_gate = prior["acceptance_gate"]
    old_gate = old.get("acceptance_gate")
    new_gate = new.get("acceptance_gate")
    if old.get("closure_record") is not None:
        raise ReceiptError("closure transition did not start from null closure")
    if (
        not isinstance(old_gate, dict)
        or set(old_gate) != {"verdict", "rollup_rule", "gate_a_status"}
        or old_gate.get("gate_a_status") != "not-passed"
    ):
        raise ReceiptError("closure transition did not start from exact open Gate A")
    expected_gate = copy.deepcopy(old_gate)
    expected_gate.update({
        "verdict": prior_gate.get("verdict"),
        "gate_a_status": "passed",
    })
    if new_gate != expected_gate:
        raise ReceiptError("acceptance metadata is not the exact derived closure")
    closure = new.get("closure_record")
    closure_keys = {
        "gate", "permitted_claim", "candidate_commit_sha",
        "source_version", "scope_sha256", "envelope_ref",
        "audit_cutoff_at_utc", "scope_audit_ref",
        "assurance_record_refs", "residual_refs", "claim_limitations",
        "verification_receipt_ref", "closure_policy_ref",
    }
    if not isinstance(closure, dict) or set(closure) != closure_keys:
        raise ReceiptError("closure record does not use the exact v2 schema")
    exact_links = {
        "gate": prior_closure.get("gate"),
        "permitted_claim": prior_closure.get("permitted_claim"),
        "candidate_commit_sha": audit_commit_sha,
        "source_version": receipt.get("source_version"),
        "scope_sha256": audit.get("scope_sha256"),
        "envelope_ref": prior_closure.get("envelope_ref"),
        "audit_cutoff_at_utc": audit.get("executed_at_utc"),
        "scope_audit_ref": audit.get("id"),
        "assurance_record_refs": prior_closure.get("assurance_record_refs"),
        "verification_receipt_ref": receipt_path,
        "residual_refs": prior_closure.get("residual_refs"),
        "claim_limitations": prior_closure.get("claim_limitations"),
        "closure_policy_ref": audit.get("policy_basis"),
    }
    for key, value in exact_links.items():
        if closure.get(key) != value:
            raise ReceiptError(f"closure {key} is not predecessor/audit-derived")
    residuals = closure.get("residual_refs")
    limitations = closure.get("claim_limitations")
    if (
        not isinstance(residuals, list)
        or len(residuals) != len(set(residuals))
        or any(not isinstance(item, str) or not item for item in residuals)
        or not isinstance(limitations, list)
        or len(limitations) != len(residuals)
    ):
        raise ReceiptError("closure residual and limitation sets are malformed")
    limitation_keys = {
        "defect_ref", "affected_claim_ref", "public_claim_restriction",
    }
    if (
        any(
            not isinstance(item, dict)
            or set(item) != limitation_keys
            or item.get("defect_ref") != residuals[index]
            or any(not isinstance(value, str) or not value for value in item.values())
            for index, item in enumerate(limitations)
        )
    ):
        raise ReceiptError(
            "closure limitations are not the exact ordered residual projection"
        )


def _walk_strings(value: object, pointer: str = "") -> Iterator[tuple[str, str]]:
    if isinstance(value, str):
        yield pointer, value
    elif isinstance(value, list):
        for index, item in enumerate(value):
            yield from _walk_strings(item, f"{pointer}/{index}")
    elif isinstance(value, dict):
        for key in sorted(value):
            escaped = key.replace("~", "~0").replace("/", "~1")
            yield from _walk_strings(value[key], f"{pointer}/{escaped}")


def _active_reference_projection(manifest: list[dict]) -> list[dict]:
    paths = _manifest_map(manifest)
    projection: list[dict] = []
    target_cache: dict[str, str] = {}
    for source_path in sorted(paths):
        if not source_path.endswith(".json") or source_path.startswith(
            "new-book-plans/verification-receipts/"
        ):
            continue
        item = paths[source_path]
        if item["type"] != "blob":
            continue
        try:
            source = json.loads(_blob(item["object"]).decode("utf-8"))
        except (UnicodeError, json.JSONDecodeError):
            continue
        for pointer, value in _walk_strings(source):
            if value.count("::") != 1:
                continue
            target, needle = value.split("::", 1)
            if (
                not target
                or not needle
                or not _SAFE_REF_PATH_RE.fullmatch(target)
            ):
                continue
            path_like = (
                "/" in target
                or "." in pathlib.PurePosixPath(target).name
                or target in paths
            )
            if not path_like:
                continue
            if target not in paths or paths[target]["type"] != "blob":
                raise ReceiptError(
                    f"active reference target is absent or not a blob: {target}"
                )
            if target not in target_cache:
                try:
                    target_cache[target] = _blob(
                        paths[target]["object"]
                    ).decode("utf-8")
                except UnicodeError as exc:
                    raise ReceiptError(
                        f"reference target is not UTF-8: {target}"
                    ) from exc
            count = target_cache[target].count(needle)
            if count != 1:
                raise ReceiptError(
                    f"active reference must occur exactly once; found {count}: "
                    f"{target}::{needle}"
                )
            projection.append(
                {
                    "source": source_path,
                    "pointer": pointer,
                    "target": target,
                    "needle": needle,
                    "count": count,
                }
            )
    return projection


def _unchecked_todo_blocks(body: str) -> list[tuple[int, int]]:
    lines = body.splitlines(keepends=True)
    offsets: list[int] = []
    cursor = 0
    for line in lines:
        offsets.append(cursor)
        cursor += len(line)
    starts = [
        index for index, line in enumerate(lines)
        if re.match(r"^- \[ \] ", line)
    ]
    blocks: list[tuple[int, int]] = []
    for position, start_line in enumerate(starts):
        next_item = starts[position + 1] if position + 1 < len(starts) else len(lines)
        end_line = next_item
        for index in range(start_line + 1, next_item):
            if re.match(r"^#{1,6} ", lines[index]):
                end_line = index
                break
        start = offsets[start_line]
        end = offsets[end_line] if end_line < len(lines) else len(body)
        blocks.append((start, end))
    return blocks


def _validate_tracker_transition(
    old_manifest: list[dict], new_manifest: list[dict]
) -> None:
    changes = _changed_paths(old_manifest, new_manifest)
    if set(changes) != {TODO_PATH}:
        raise ReceiptError("tracker transition may change TODO.md only")
    _validate_modes(changes)
    try:
        old_todo = _blob_at(old_manifest, TODO_PATH).decode("utf-8")
        new_todo = _blob_at(new_manifest, TODO_PATH).decode("utf-8")
    except UnicodeError as exc:
        raise ReceiptError("TODO.md must remain UTF-8") from exc
    blocks = _unchecked_todo_blocks(old_todo)
    if not blocks:
        raise ReceiptError("tracker predecessor has no unchecked TODO block")
    start, end = blocks[0]
    if old_todo[:start] + old_todo[end:] != new_todo:
        raise ReceiptError(
            "tracker transition must delete exactly the first whole top-level "
            "unchecked TODO block without replacement or reordering"
        )
    old_projection = _active_reference_projection(old_manifest)
    new_projection = _active_reference_projection(new_manifest)
    if old_projection != new_projection:
        raise ReceiptError("tracker transition changed an active path::needle projection")


def _candidate_commit(receipt: dict, commit: str) -> None:
    candidate = receipt["candidate"]
    if _git_text("rev-parse", f"{commit}^{{tree}}") != candidate["tree_sha"]:
        raise ReceiptError("candidate commit tree differs from receipt")
    _require_single_parent(commit, candidate["parent_commit_sha"])
    manifest = _tree_manifest(commit)
    if (
        _sha256(_canonical(manifest)) != candidate["path_manifest_sha256"]
        or len(manifest) != candidate["path_count"]
    ):
        raise ReceiptError("candidate commit path manifest differs from receipt")


def resolve_candidate_commit(
    receipt: dict,
    *,
    root: pathlib.Path | None = None,
    tip: str = "HEAD",
) -> str:
    """Resolve the unique commit having the receipt's exact tree/parent pair."""

    root = (root or ROOT).resolve()
    candidate = receipt.get("candidate")
    if not isinstance(candidate, dict):
        raise ReceiptError("receipt has no candidate binding")
    commits = _git_text(
        "rev-list", "--first-parent", tip, root=root
    ).splitlines()
    matches = []
    for commit in commits:
        tree = _git_text("rev-parse", f"{commit}^{{tree}}", root=root)
        if tree != candidate.get("tree_sha"):
            continue
        fields = _git_text(
            "rev-list", "--parents", "-n", "1", commit, root=root
        ).split()
        if fields[1:] == [candidate.get("parent_commit_sha")]:
            matches.append(commit)
    if len(matches) != 1:
        raise ReceiptError(
            "receipt candidate does not resolve to one normal first-parent commit"
        )
    return matches[0]


def validate_recorded_transition(
    receipt: dict,
    successor_sha: str,
    transition: str,
    *,
    receipt_path: str,
    root: pathlib.Path | None = None,
) -> str:
    """Validate an already committed audit or closure administrative successor."""

    root = (root or ROOT).resolve()
    if root != ROOT.resolve():
        raise ReceiptError("recorded transition root differs from loaded repository")
    if not _HEX40_RE.fullmatch(successor_sha):
        raise ReceiptError("recorded successor must be a full Git commit SHA")
    parents = _parents(successor_sha)
    if len(parents) != 1:
        raise ReceiptError("recorded administrative successor must not be a merge")
    predecessor = parents[0]
    if transition == "audit":
        _candidate_commit(receipt, predecessor)
        _validate_audit_transition(
            _tree_manifest(predecessor),
            _tree_manifest(successor_sha),
            receipt,
            receipt_path,
        )
        return predecessor
    if transition == "closure":
        validate_recorded_transition(
            receipt, predecessor, "audit", receipt_path=receipt_path, root=root
        )
        _validate_closure_transition(
            _tree_manifest(predecessor),
            _tree_manifest(successor_sha),
            receipt,
            receipt_path,
            predecessor,
        )
        return predecessor
    raise ReceiptError(f"unknown recorded transition: {transition}")


def validate_commit_gate(
    receipt_path: pathlib.Path,
    transition: str,
    *,
    root: pathlib.Path | None = None,
) -> str:
    root = root or ROOT
    receipt_path = receipt_path if receipt_path.is_absolute() else root / receipt_path
    receipt = load_and_validate_receipt(receipt_path, root=root)
    if receipt.get("schema_version") != 2:
        raise ReceiptError("administrative commit gates require receipt schema v2")
    relative_receipt = _receipt_relative(receipt_path)
    _, staged_tree, staged_manifest = _fully_staged_candidate()
    head = _git_text("rev-parse", "HEAD")

    if transition == "audit":
        candidate = head
        _candidate_commit(receipt, candidate)
        _validate_audit_transition(
            _tree_manifest(candidate), staged_manifest, receipt, relative_receipt
        )
    elif transition == "closure":
        audit = head
        audit_parents = _parents(audit)
        if len(audit_parents) != 1:
            raise ReceiptError("audit successor must not be a merge")
        candidate = audit_parents[0]
        _candidate_commit(receipt, candidate)
        _validate_audit_transition(
            _tree_manifest(candidate),
            _tree_manifest(audit),
            receipt,
            relative_receipt,
        )
        _validate_closure_transition(
            _tree_manifest(audit), staged_manifest, receipt, relative_receipt,
            audit,
        )
    elif transition == "tracker":
        closure = head
        closure_parents = _parents(closure)
        if len(closure_parents) != 1:
            raise ReceiptError("closure successor must not be a merge")
        audit = closure_parents[0]
        audit_parents = _parents(audit)
        if len(audit_parents) != 1:
            raise ReceiptError("audit successor must not be a merge")
        candidate = audit_parents[0]
        _candidate_commit(receipt, candidate)
        _validate_audit_transition(
            _tree_manifest(candidate),
            _tree_manifest(audit),
            receipt,
            relative_receipt,
        )
        _validate_closure_transition(
            _tree_manifest(audit),
            _tree_manifest(closure),
            receipt,
            relative_receipt,
            audit,
        )
        _validate_tracker_transition(_tree_manifest(closure), staged_manifest)
    else:
        raise ReceiptError(f"unknown transition: {transition}")
    return staged_tree


def run_commit_gate(
    receipt_path: pathlib.Path,
    transition: str,
    *,
    wait_seconds: float = 0,
    root: pathlib.Path | None = None,
) -> str:
    """Validate one administrative successor and rerun structural verification."""

    root = (root or ROOT).resolve()
    absolute_receipt = (
        receipt_path if receipt_path.is_absolute() else root / receipt_path
    )
    relative_receipt = _receipt_relative(absolute_receipt)
    lock_command = (
        "./verify.sh", "--commit-gate", relative_receipt,
        "--transition", transition,
    )
    try:
        with VerificationLock(
            "verify",
            wait_seconds=wait_seconds,
            engine_path=_engine_paths()[1],
            command=lock_command,
            root=root,
        ) as held:
            before = _fully_staged_candidate()
            tree = validate_commit_gate(
                absolute_receipt, transition, root=root
            )
            returncode, _, _ = _run_captured(
                ["./verify.sh", "--quick"],
                expected=("./verify.sh", "--quick"),
                lock_fd=held.lock_fd,
            )
            if returncode:
                raise ReceiptError(
                    "structural quick verification failed; no full fallback was run"
                )
            after = _fully_staged_candidate()
            if after != before:
                raise ReceiptError(
                    "staged repository inputs drifted during structural verification"
                )
            load_and_validate_receipt(
                absolute_receipt,
                root=root,
                require_local=True,
                check_environment=True,
                check_engine=True,
            )
            return tree
    except VerificationLockBusy:
        raise
    except VerificationLockError as exc:
        raise ReceiptError(str(exc)) from exc


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    emit = sub.add_parser("emit", help="run one staged full verifier and emit v2")
    emit.add_argument("--output", required=True, type=pathlib.Path)
    emit.add_argument("--wait-for-lock", type=float, default=0)
    emit.add_argument("command", nargs=argparse.REMAINDER)
    validate = sub.add_parser("validate", help="validate a compact or allowlisted v1 receipt")
    validate.add_argument("receipt", type=pathlib.Path)
    validate.add_argument("--source-version")
    validate.add_argument("--audit-id")
    validate.add_argument("--no-environment-check", action="store_true")
    validate.add_argument("--no-engine-check", action="store_true")
    gate = sub.add_parser("commit-gate", help="validate one administrative successor")
    gate.add_argument("receipt", type=pathlib.Path)
    gate.add_argument("--transition", required=True, choices=("audit", "closure", "tracker"))
    gate.add_argument("--wait-for-lock", type=float, default=0)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        if args.action == "emit":
            command = list(args.command)
            if command and command[0] == "--":
                command.pop(0)
            emit_receipt(
                args.output,
                command,
                wait_seconds=args.wait_for_lock,
            )
        elif args.action == "validate":
            receipt = load_and_validate_receipt(
                args.receipt,
                source_version=args.source_version,
                audit_id=args.audit_id,
                check_environment=not args.no_environment_check,
                check_engine=not args.no_engine_check,
            )
            schema = receipt.get("schema_version", 1)
            print(f"verification receipt schema v{schema}: ok")
        else:
            tree = run_commit_gate(
                args.receipt,
                args.transition,
                wait_seconds=args.wait_for_lock,
            )
            print(f"verification commit gate {args.transition}: ok ({tree})")
        return 0
    except VerificationLockBusy as exc:
        print(f"verification receipt: {exc}", file=sys.stderr)
        return EX_TEMPFAIL
    except (ReceiptError, VerificationLockError) as exc:
        print(f"verification receipt: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
