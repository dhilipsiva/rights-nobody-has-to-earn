#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0

"""Shared lock adapter for heavyweight standalone verification checkers."""

from __future__ import annotations

import argparse
import pathlib
import sys
from collections.abc import Callable, Sequence

from verification_lock import (
    EX_TEMPFAIL,
    VerificationLock,
    VerificationLockBusy,
    VerificationLockError,
)


def run_checker(
    checker: Callable[[Sequence[str] | None], int],
    argv: Sequence[str],
    *,
    nonheavy_flags: Sequence[str] = (),
) -> int:
    """Run generation or executable controls under the repository lock.

    Structural ``--check``, help, and declared read-only modes remain lock-free.
    ``VerificationLock`` itself validates and reuses inherited ownership from a
    parent verifier; standalone contention fails immediately unless the caller
    explicitly supplies ``--wait-for-lock``.
    """

    arguments = tuple(argv)
    wait_parser = argparse.ArgumentParser(add_help=False)
    wait_parser.add_argument("--wait-for-lock", type=float, default=0.0)
    wait_args, _ = wait_parser.parse_known_args(arguments)
    nonheavy = (
        "--help" in arguments
        or "-h" in arguments
        or any(flag in arguments for flag in nonheavy_flags)
        or ("--check" in arguments and "--execute" not in arguments)
    )
    if nonheavy:
        return checker(arguments)

    command = (
        sys.executable,
        str(pathlib.Path(sys.argv[0]).resolve()),
        *arguments,
    )
    try:
        with VerificationLock(
            "verify",
            wait_seconds=wait_args.wait_for_lock,
            command=command,
        ):
            return checker(arguments)
    except VerificationLockBusy as exc:
        print(f"verification lock: {exc}", file=sys.stderr)
        return EX_TEMPFAIL
    except VerificationLockError as exc:
        print(f"verification lock: {exc}", file=sys.stderr)
        return 2
