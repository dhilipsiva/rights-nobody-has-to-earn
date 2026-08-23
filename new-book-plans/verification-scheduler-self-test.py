#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Focused behavioral tests for the sliding state-form shard runner."""

from __future__ import annotations

import json
import os
import pathlib
import shlex
import subprocess
import tempfile


ROOT = pathlib.Path(__file__).resolve().parents[1]
RUNNER = ROOT / "new-book-plans" / "verification-shard-runner.sh"

FAKE_ENGINE = r'''#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
import fcntl
import json
import os
import pathlib
import signal
import sys
import time

state_path = pathlib.Path(os.environ["FAKE_SCHEDULER_STATE"])
lock_path = state_path.with_suffix(".lock")
name = pathlib.Path(sys.argv[-1]).name
finished = False
termination_signals = {signal.SIGINT, signal.SIGTERM}


def mutate(kind):
    previous_mask = signal.pthread_sigmask(
        signal.SIG_BLOCK, termination_signals)
    try:
        mutate_with_signals_blocked(kind)
    finally:
        signal.pthread_sigmask(signal.SIG_SETMASK, previous_mask)


def mutate_with_signals_blocked(kind):
    global finished
    with lock_path.open("a+") as handle:
        fcntl.flock(handle.fileno(), fcntl.LOCK_EX)
        if state_path.exists():
            state = json.loads(state_path.read_text(encoding="utf-8"))
        else:
            state = {
                "active": 0, "max_active": 0, "starts": {},
                "ends": {}, "events": [], "pids": {},
            }
        now = time.monotonic_ns()
        if kind == "start":
            state["active"] += 1
            state["max_active"] = max(
                state["max_active"], state["active"])
            state["starts"][name] = now
            state["pids"][name] = os.getpid()
        elif not finished:
            state["active"] -= 1
            state["ends"][name] = now
            finished = True
        state["events"].append([now, kind, name])
        temporary = state_path.with_name(
            state_path.name + f".{os.getpid()}.tmp")
        temporary.write_text(
            json.dumps(state, sort_keys=True), encoding="utf-8")
        os.replace(temporary, state_path)
        fcntl.flock(handle.fileno(), fcntl.LOCK_UN)


def terminate(signum, _frame):
    if name == os.environ.get("FAKE_SCHEDULER_IGNORE_TERM", ""):
        return
    mutate("terminated")
    os._exit(128 + signum)


signal.signal(signal.SIGTERM, terminate)
signal.signal(signal.SIGINT, terminate)
mutate("start")
fail_name = os.environ.get("FAKE_SCHEDULER_FAIL", "")
if name == fail_name:
    time.sleep(0.02)
    mutate("failed")
    print("FAIL injected", flush=True)
    raise SystemExit(2)
delay = 0.60 if name.endswith("-01.pins.nibli") else 0.12
time.sleep(delay)
mutate("end")
print("PASS", flush=True)
'''


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def require_children_reaped(state: dict) -> None:
    for name, pid in state["pids"].items():
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            continue
        raise RuntimeError(f"scheduler did not reap {name} child {pid}")


def make_fixture(base: pathlib.Path, count: int) -> tuple[pathlib.Path, ...]:
    paths = []
    for index in range(1, count + 1):
        path = base / f"main-{index:02d}.pins.nibli"
        path.write_text(f"# shard {index}\n", encoding="utf-8")
        paths.append(path)
    return tuple(paths)


def run_case(
    base: pathlib.Path,
    *,
    count: int,
    fail_name: str = "",
    ignore_term_name: str = "",
    inject_wait_no_pid: bool = False,
) -> tuple[subprocess.CompletedProcess[str], dict]:
    engine = base / "fake-nibli-pin.py"
    engine.write_text(FAKE_ENGINE, encoding="utf-8")
    engine.chmod(0o755)
    state_path = base / "state.json"
    kb = base / "constitution.nibli"
    kb.write_text("# fixture\n", encoding="utf-8")
    shards = make_fixture(base, count)
    invocation = " ".join(
        [
            "run_pin_shards",
            shlex.quote(str(kb)),
            *(shlex.quote(str(path)) for path in shards),
        ]
    )
    command_lines = [
        "set -uo pipefail",
        f"PIN={shlex.quote(str(engine))}",
        "STATE_FORM_MAX_PARALLEL=4",
        "STATE_FORM_TERMINATION_GRACE_SECONDS=0.10",
        f". {shlex.quote(str(RUNNER))}",
    ]
    if inject_wait_no_pid:
        command_lines.extend(
            [
                "WAIT_NO_PID_ONCE=1",
                "wait() {",
                "  if [ \"$WAIT_NO_PID_ONCE\" = 1 ] && [ \"${1:-}\" = -n ]; then",
                "    WAIT_NO_PID_ONCE=0",
                "    sleep 0.05",
                "    return 127",
                "  fi",
                "  builtin wait \"$@\"",
                "}",
            ]
        )
    command_lines.append(invocation)
    command = "\n".join(command_lines)
    env = os.environ.copy()
    env["FAKE_SCHEDULER_STATE"] = str(state_path)
    if fail_name:
        env["FAKE_SCHEDULER_FAIL"] = fail_name
    if ignore_term_name:
        env["FAKE_SCHEDULER_IGNORE_TERM"] = ignore_term_name
    result = subprocess.run(
        ["bash", "-c", command],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if not state_path.exists():
        raise RuntimeError(
            "scheduler child produced no state\n"
            + result.stdout + result.stderr + "\n" + command
        )
    state = json.loads(state_path.read_text(encoding="utf-8"))
    return result, state


def main() -> int:
    with tempfile.TemporaryDirectory(
        prefix="verification-scheduler."
    ) as temporary:
        root = pathlib.Path(temporary)

        success_dir = root / "success"
        success_dir.mkdir()
        result, state = run_case(success_dir, count=8)
        require(result.returncode == 0, result.stdout + result.stderr)
        require(state["max_active"] == 4, "scheduler did not cap at four")
        require(state["active"] == 0, "successful children were not reaped")
        require(
            state["starts"]["main-05.pins.nibli"]
            < state["ends"]["main-01.pins.nibli"],
            "shard five did not start while slow shard one remained active",
        )
        ordered = [
            line.split(":", 1)[0]
            for line in result.stdout.splitlines()
            if line.strip()
        ]
        require(
            ordered
            == [f"main-{index:02d}.pins.nibli" for index in range(1, 9)],
            "successful summaries were not printed in canonical order",
        )

        failure_dir = root / "failure"
        failure_dir.mkdir()
        result, state = run_case(
            failure_dir,
            count=12,
            fail_name="main-05.pins.nibli",
        )
        require(result.returncode != 0, "injected failure was accepted")
        require(state["max_active"] <= 4, "failure case exceeded four workers")
        require_children_reaped(state)
        require(
            "main-12.pins.nibli" not in state["starts"],
            "scheduler kept launching after failure",
        )
        require(
            (failure_dir / ".retain").exists(),
            "failure diagnostics were not retained",
        )
        require(
            "main-05.pins.nibli" in result.stdout,
            "failing shard diagnostic was not printed",
        )

        no_pid_dir = root / "no-pid"
        no_pid_dir.mkdir()
        result, state = run_case(
            no_pid_dir,
            count=8,
            ignore_term_name="main-01.pins.nibli",
            inject_wait_no_pid=True,
        )
        require(result.returncode != 0, "missing wait PID was accepted")
        require(
            set(state["starts"]) == {
                "main-01.pins.nibli",
                "main-02.pins.nibli",
                "main-03.pins.nibli",
                "main-04.pins.nibli",
            },
            "scheduler launched more shards after wait lost the child PID",
        )
        require(
            (no_pid_dir / ".retain").exists(),
            "wait failure diagnostics were not retained",
        )
        require(
            "scheduler could not identify a completed child" in result.stdout,
            "wait failure diagnostic was not printed",
        )
        require(
            state["active"] > 0,
            "forced-KILL fixture did not retain stale child bookkeeping",
        )
        require_children_reaped(state)

    print("verification scheduler self-test passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
