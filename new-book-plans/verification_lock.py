#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Repository-wide, Git-common-dir verification lock.

The kernel ``flock`` is authoritative. The adjacent JSON file is diagnostic
only: it deliberately contains neither the ownership token nor arbitrary
environment or command-line values. Linked worktrees share the same lock
because the location is derived from ``git rev-parse --git-common-dir``.
"""

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import fcntl
import hmac
import hashlib
import json
import os
import pathlib
import re
import signal
import shutil
import subprocess
import sys
import time
import uuid
from typing import Sequence


EX_TEMPFAIL = 75
LOCK_SUBDIR = "rights-verification"
LOCK_FILENAME = "heavyweight.lock"
OWNER_FILENAME = "heavyweight-owner.json"
TOKEN_ENV = "RIGHTS_VERIFY_LOCK_TOKEN"
OWNER_PID_ENV = "RIGHTS_VERIFY_LOCK_OWNER_PID"
OWNER_START_ENV = "RIGHTS_VERIFY_LOCK_OWNER_START"
COMMON_DIR_ENV = "RIGHTS_VERIFY_LOCK_COMMON_DIR"
NAME_ENV = "RIGHTS_VERIFY_LOCK_NAME"
_SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
_SAFE_NAME_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.:-]{0,95}$")


class VerificationLockError(RuntimeError):
    """A lock request is malformed or inherited ownership is forged/stale."""


class VerificationLockBusy(VerificationLockError):
    """The shared heavyweight lock was not available in the allowed window."""


def _run_git(root: pathlib.Path, *args: str) -> str:
    proc = subprocess.run(
        ["git", "-C", str(root), *args],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
    )
    if proc.returncode:
        detail = proc.stderr.strip() or proc.stdout.strip() or "git failed"
        raise VerificationLockError(detail)
    return proc.stdout.strip()


def repository_root(start: pathlib.Path | None = None) -> pathlib.Path:
    start = (start or pathlib.Path.cwd()).resolve()
    return pathlib.Path(_run_git(start, "rev-parse", "--show-toplevel")).resolve()


def git_common_dir(root: pathlib.Path | None = None) -> pathlib.Path:
    repo = repository_root(root)
    raw = _run_git(
        repo, "rev-parse", "--path-format=absolute", "--git-common-dir"
    )
    return pathlib.Path(raw).resolve()


def lock_paths(root: pathlib.Path | None = None) -> tuple[pathlib.Path, pathlib.Path]:
    common = git_common_dir(root)
    directory = common / LOCK_SUBDIR
    return directory / LOCK_FILENAME, directory / OWNER_FILENAME


def _process_stat_tail(pid: int) -> list[str]:
    try:
        raw = pathlib.Path(f"/proc/{pid}/stat").read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise VerificationLockError(f"cannot inspect lock process {pid}") from exc
    close = raw.rfind(")")
    if close < 0 or close + 2 >= len(raw):
        raise VerificationLockError(f"malformed process identity for {pid}")
    fields = raw[close + 2:].split()
    if len(fields) < 20:
        raise VerificationLockError(f"malformed process identity for {pid}")
    return fields


def _process_start_ticks(pid: int) -> str:
    return _process_stat_tail(pid)[19]


def _parent_pid(pid: int) -> int:
    try:
        return int(_process_stat_tail(pid)[1])
    except (VerificationLockError, ValueError):
        return 0


def _is_ancestor(ancestor_pid: int, descendant_pid: int) -> bool:
    seen: set[int] = set()
    current = descendant_pid
    while current > 1 and current not in seen:
        if current == ancestor_pid:
            return True
        seen.add(current)
        current = _parent_pid(current)
    return current == ancestor_pid


def _token_digest(token: str) -> str:
    return hashlib.sha256(token.encode("ascii")).hexdigest()


def _utc_now() -> str:
    return dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat().replace(
        "+00:00", "Z"
    )


def _validate_name(name: str) -> str:
    if not _SAFE_NAME_RE.fullmatch(name):
        raise VerificationLockError(
            "lock name must be 1-96 safe identifier characters"
        )
    return name


def _validate_wait(wait_seconds: float) -> float:
    if not isinstance(wait_seconds, (int, float)) or not 0 <= wait_seconds <= 86400:
        raise VerificationLockError("wait timeout must be between 0 and 86400 seconds")
    return float(wait_seconds)


def _file_identity(path: pathlib.Path | None) -> dict | None:
    if path is None:
        return None
    resolved = path.resolve()
    if not resolved.is_file():
        return {
            "basename": resolved.name,
            "exists": False,
        }
    body = resolved.read_bytes()
    return {
        "basename": resolved.name,
        "exists": True,
        "sha256": hashlib.sha256(body).hexdigest(),
        "size": len(body),
    }


def _repository_source_digest(root: pathlib.Path) -> str:
    index = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-s", "-z"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if index.returncode:
        raise VerificationLockError(
            "cannot bind the repository index identity"
        )
    head = _run_git(root, "rev-parse", "HEAD").encode("ascii")
    return hashlib.sha256(head + b"\0" + index.stdout).hexdigest()


def _default_engine_path() -> pathlib.Path:
    source = pathlib.Path(
        os.environ.get(
            "NIBLI_SRC",
            str(pathlib.Path.home() / "projects/dhilipsiva/nibli"),
        )
    )
    return pathlib.Path(
        os.environ.get(
            "NIBLI_PIN", str(source / "target/release/nibli-pin")
        )
    )


def _command_identity(
    command: Sequence[str] | None, token: str
) -> dict | None:
    if command is None:
        return None
    if not command:
        raise VerificationLockError("lock command identity may not be empty")
    executable = pathlib.Path(command[0])
    if not executable.is_absolute():
        located = shutil.which(command[0])
        if located is not None:
            executable = pathlib.Path(located)
    binding = hmac.new(
        token.encode("ascii"),
        json.dumps(
            list(command),
            ensure_ascii=False,
            separators=(",", ":"),
        ).encode("utf-8"),
        hashlib.sha256,
    ).hexdigest()
    return {
        "executable": _file_identity(executable),
        "argument_count": len(command) - 1,
        "argv_binding_hmac_sha256": binding,
    }


def _owner_document(
    *,
    name: str,
    token: str,
    source_digest: str | None,
    engine_path: pathlib.Path | None,
    command: Sequence[str] | None,
) -> dict:
    if source_digest is not None and not _SHA256_RE.fullmatch(source_digest):
        raise VerificationLockError("source digest must be lowercase SHA-256")
    pid = os.getpid()
    return {
        "schema_version": 1,
        "name": _validate_name(name),
        "owner_pid": pid,
        "owner_process_start_ticks": _process_start_ticks(pid),
        "owner_process_group_id": os.getpgrp(),
        "started_at_utc": _utc_now(),
        "token_sha256": _token_digest(token),
        "source_sha256": source_digest,
        "engine": _file_identity(engine_path),
        "command": _command_identity(command, token),
    }


def _atomic_json(path: pathlib.Path, value: dict) -> None:
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    tmp = path.with_name(f".{path.name}.{os.getpid()}.{uuid.uuid4().hex}.tmp")
    try:
        with tmp.open("x", encoding="utf-8", newline="\n") as handle:
            json.dump(value, handle, indent=2, sort_keys=True)
            handle.write("\n")
            handle.flush()
            os.fsync(handle.fileno())
        os.chmod(tmp, 0o600)
        os.replace(tmp, path)
    finally:
        with contextlib.suppress(FileNotFoundError):
            tmp.unlink()


def _read_owner(path: pathlib.Path) -> dict | None:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (FileNotFoundError, OSError, json.JSONDecodeError):
        return None
    if not isinstance(value, dict):
        return None
    return value


def sanitized_owner_details(owner_path: pathlib.Path) -> dict:
    """Return only public diagnostic fields, never token/path/argv material."""

    owner = _read_owner(owner_path) or {}
    result: dict[str, object] = {}
    for key in (
        "schema_version",
        "name",
        "owner_pid",
        "owner_process_start_ticks",
        "owner_process_group_id",
        "started_at_utc",
        "source_sha256",
        "engine",
        "command",
    ):
        value = owner.get(key)
        if value is not None:
            result[key] = value
    return result


def _inherited_context(
    common: pathlib.Path,
    lock_path: pathlib.Path,
    owner_path: pathlib.Path,
) -> dict | None:
    token = os.environ.get(TOKEN_ENV)
    if token is None:
        return None
    owner_pid_raw = os.environ.get(OWNER_PID_ENV, "")
    owner_start = os.environ.get(OWNER_START_ENV, "")
    inherited_common = os.environ.get(COMMON_DIR_ENV, "")
    inherited_name = os.environ.get(NAME_ENV, "")
    if (not token or not owner_pid_raw.isdecimal() or not owner_start
            or not inherited_name):
        raise VerificationLockError("inherited lock ownership is incomplete")
    try:
        inherited_path = pathlib.Path(inherited_common).resolve()
    except (OSError, RuntimeError) as exc:
        raise VerificationLockError("inherited lock common directory is invalid") from exc
    if inherited_path != common:
        raise VerificationLockError("inherited lock belongs to a different Git common dir")
    owner_pid = int(owner_pid_raw)
    if owner_pid <= 1 or not _is_ancestor(owner_pid, os.getpid()):
        raise VerificationLockError("inherited lock owner is not a live ancestor")
    if _process_start_ticks(owner_pid) != owner_start:
        raise VerificationLockError("inherited lock owner identity is stale")
    owner = _read_owner(owner_path)
    if not owner:
        raise VerificationLockError("inherited lock owner metadata is missing")
    expected = {
        "name": inherited_name,
        "owner_pid": owner_pid,
        "owner_process_start_ticks": owner_start,
        "token_sha256": _token_digest(token),
    }
    for key, value in expected.items():
        if owner.get(key) != value:
            raise VerificationLockError("inherited lock ownership does not match metadata")
    with lock_path.open("a+b") as probe:
        try:
            fcntl.flock(probe.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            pass
        else:
            fcntl.flock(probe.fileno(), fcntl.LOCK_UN)
            raise VerificationLockError(
                "inherited owner metadata exists without a held kernel lock"
            )
    return owner


class VerificationLock:
    """Context manager for a fail-fast or explicitly bounded shared lock."""

    def __init__(
        self,
        command_name: str,
        *,
        wait_seconds: float = 0,
        source_digest: str | None = None,
        engine_path: pathlib.Path | None = None,
        command: Sequence[str] | None = None,
        root: pathlib.Path | None = None,
    ) -> None:
        self.command_name = _validate_name(command_name)
        self.wait_seconds = _validate_wait(wait_seconds)
        self.source_digest = source_digest
        self.engine_path = engine_path
        self.command = tuple(command) if command is not None else None
        self.root = repository_root(root)
        self.common = git_common_dir(self.root)
        self.lock_path, self.owner_path = lock_paths(self.root)
        self._handle = None
        self._token: str | None = None
        self._prior_env: dict[str, str | None] = {}
        self.inherited = False

    def __enter__(self) -> "VerificationLock":
        inherited = _inherited_context(
            self.common, self.lock_path, self.owner_path
        )
        if inherited is not None:
            self.inherited = True
            return self

        self.lock_path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        self._handle = self.lock_path.open("a+b")
        deadline = time.monotonic() + self.wait_seconds
        while True:
            try:
                fcntl.flock(self._handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError as exc:
                if time.monotonic() >= deadline:
                    details = json.dumps(
                        sanitized_owner_details(self.owner_path),
                        sort_keys=True,
                        separators=(",", ":"),
                    )
                    self._handle.close()
                    self._handle = None
                    raise VerificationLockBusy(
                        f"heavyweight verifier lock is busy: {details}"
                    ) from exc
                time.sleep(min(0.1, max(0.0, deadline - time.monotonic())))

        self._token = uuid.uuid4().hex + uuid.uuid4().hex
        try:
            source_digest = self.source_digest or _repository_source_digest(
                self.root
            )
            owner = _owner_document(
                name=self.command_name,
                token=self._token,
                source_digest=source_digest,
                engine_path=self.engine_path or _default_engine_path(),
                command=self.command,
            )
            _atomic_json(self.owner_path, owner)
            values = {
                TOKEN_ENV: self._token,
                OWNER_PID_ENV: str(owner["owner_pid"]),
                OWNER_START_ENV: str(owner["owner_process_start_ticks"]),
                COMMON_DIR_ENV: str(self.common),
                NAME_ENV: self.command_name,
            }
            for key, value in values.items():
                self._prior_env[key] = os.environ.get(key)
                os.environ[key] = value
        except BaseException:
            current = _read_owner(self.owner_path)
            if current and current.get("token_sha256") == _token_digest(
                self._token
            ):
                with contextlib.suppress(OSError):
                    self.owner_path.unlink()
            if self._handle is not None:
                with contextlib.suppress(OSError):
                    fcntl.flock(self._handle.fileno(), fcntl.LOCK_UN)
                self._handle.close()
                self._handle = None
            raise
        return self

    def __exit__(self, exc_type, exc, traceback) -> None:
        if self.inherited:
            return
        for key, prior in self._prior_env.items():
            if prior is None:
                os.environ.pop(key, None)
            else:
                os.environ[key] = prior
        owner = _read_owner(self.owner_path)
        if owner and self._token and owner.get("token_sha256") == _token_digest(
            self._token
        ):
            with contextlib.suppress(OSError):
                self.owner_path.unlink()
        if self._handle is not None:
            with contextlib.suppress(OSError):
                fcntl.flock(self._handle.fileno(), fcntl.LOCK_UN)
            self._handle.close()
            self._handle = None

    @property
    def lock_fd(self) -> int | None:
        """Return the owning descriptor for inheritance by the heavy child."""

        return self._handle.fileno() if self._handle is not None else None


def inherited_environment() -> dict[str, str]:
    """Return the validated lock environment for an explicitly spawned child."""

    keys = (TOKEN_ENV, OWNER_PID_ENV, OWNER_START_ENV, COMMON_DIR_ENV, NAME_ENV)
    result = {key: os.environ[key] for key in keys if key in os.environ}
    if len(result) != len(keys):
        raise VerificationLockError("no complete lock ownership is active")
    return result


def validate_inherited(
    command_name: str,
    *,
    root: pathlib.Path | None = None,
) -> dict:
    """Validate inherited ownership without acquiring or waiting for the lock."""

    name = _validate_name(command_name)
    repo = repository_root(root)
    common = git_common_dir(repo)
    lock_path, owner_path = lock_paths(repo)
    owner = _inherited_context(common, lock_path, owner_path)
    if owner is None:
        raise VerificationLockError("no inherited verification lock is active")
    if owner.get("name") != name:
        raise VerificationLockError(
            "inherited lock owner name does not match the internal verifier"
        )
    return owner


def _run_child(command: Sequence[str], *, lock_fd: int | None = None) -> int:
    if not command:
        raise VerificationLockError("a child command is required")
    pass_fds = (lock_fd,) if lock_fd is not None else ()
    managed = (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)
    old_mask = signal.pthread_sigmask(signal.SIG_BLOCK, managed)
    try:
        child = subprocess.Popen(
            list(command),
            pass_fds=pass_fds,
            start_new_session=True,
        )
    except BaseException:
        signal.pthread_sigmask(signal.SIG_SETMASK, old_mask)
        raise
    prior: dict[int, object] = {}

    def forward(signum, _frame):
        with contextlib.suppress(ProcessLookupError):
            os.killpg(child.pid, signum)

    for signum in managed:
        prior[signum] = signal.signal(signum, forward)
    signal.pthread_sigmask(signal.SIG_SETMASK, old_mask)
    try:
        return child.wait()
    finally:
        for signum, handler in prior.items():
            signal.signal(signum, handler)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    run = sub.add_parser("run", help="run one command under the shared lock")
    run.add_argument("--name", required=True)
    run.add_argument("--wait-for-lock", type=float, default=0)
    run.add_argument("--source-digest")
    run.add_argument("--engine", type=pathlib.Path)
    run.add_argument("command", nargs=argparse.REMAINDER)
    inspect = sub.add_parser("owner", help="print sanitized current owner details")
    inspect.add_argument("--root", type=pathlib.Path, default=pathlib.Path.cwd())
    inherited = sub.add_parser(
        "validate-inherited",
        help="validate a live inherited owner without acquiring the lock",
    )
    inherited.add_argument("--name", required=True)
    inherited.add_argument(
        "--root", type=pathlib.Path, default=pathlib.Path.cwd()
    )
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        if args.action == "owner":
            _, owner_path = lock_paths(args.root)
            print(json.dumps(sanitized_owner_details(owner_path), sort_keys=True))
            return 0
        if args.action == "validate-inherited":
            validate_inherited(args.name, root=args.root)
            return 0
        command = list(args.command)
        if command and command[0] == "--":
            command.pop(0)
        with VerificationLock(
            args.name,
            wait_seconds=args.wait_for_lock,
            source_digest=args.source_digest,
            engine_path=args.engine,
            command=command,
        ) as held:
            return _run_child(command, lock_fd=held.lock_fd)
    except VerificationLockBusy as exc:
        print(f"verification lock: {exc}", file=sys.stderr)
        return EX_TEMPFAIL
    except VerificationLockError as exc:
        print(f"verification lock: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
