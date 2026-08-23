#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Sliding, fail-fast execution for ephemeral state-form pin shards.
#
# The caller supplies PIN and STATE_FORM_MAX_PARALLEL. Results are buffered and
# printed in canonical shard order even though workers complete continuously.

state_form_pid_snapshot() {
  local pid=$1 raw tail
  local -a fields=()

  [ -r "/proc/$pid/stat" ] || return 1
  IFS= read -r raw <"/proc/$pid/stat" || return 1
  tail=${raw##*) }
  read -r -a fields <<<"$tail"
  [ "${#fields[@]}" -ge 20 ] || return 1
  printf '%s %s\n' "${fields[0]}" "${fields[19]}"
}

state_form_now_microseconds() {
  local now=$EPOCHREALTIME whole fraction

  whole=${now%.*}
  fraction=${now#*.}000000
  fraction=${fraction:0:6}
  printf '%s\n' "$((10#$whole * 1000000 + 10#$fraction))"
}

cancel_pin_shards() {
  local reason=$1
  local -n active_ref=$2
  local -n index_ref=$3
  local -n log_ref=$4
  local -n detail_ref=$5
  local -n shard_ref=$6
  local grace=${STATE_FORM_TERMINATION_GRACE_SECONDS:-2}
  local grace_whole grace_fraction deadline now
  local pid index log tail_text snapshot state start
  local -a survivors=()
  local -A start_by_pid=()

  touch -- "$(dirname "${shard_ref[0]}")/.retain"
  [ "${#active_ref[@]}" -gt 0 ] || return 0
  if ! [[ "$grace" =~ ^[0-9]+([.][0-9]{1,6})?$ ]]; then
    grace=2
  fi
  grace_whole=${grace%%.*}
  if [[ "$grace" = *.* ]]; then
    grace_fraction=${grace#*.}
  else
    grace_fraction=0
  fi
  grace_fraction=${grace_fraction}000000
  grace_fraction=${grace_fraction:0:6}
  now=$(state_form_now_microseconds)
  deadline=$((now + 10#$grace_whole * 1000000 + 10#$grace_fraction))

  for pid in "${active_ref[@]}"; do
    snapshot=$(state_form_pid_snapshot "$pid" 2>/dev/null || true)
    [ -z "$snapshot" ] || start_by_pid["$pid"]=${snapshot#* }
    kill -TERM "$pid" 2>/dev/null || true
  done

  while :; do
    survivors=()
    for pid in "${active_ref[@]}"; do
      snapshot=$(state_form_pid_snapshot "$pid" 2>/dev/null || true)
      [ -n "$snapshot" ] || continue
      state=${snapshot%% *}
      start=${snapshot#* }
      [ "${start_by_pid[$pid]-}" = "$start" ] || continue
      case "$state" in
        Z|X) ;;
        *) survivors+=("$pid") ;;
      esac
    done
    [ "${#survivors[@]}" -gt 0 ] || break
    now=$(state_form_now_microseconds)
    [ "$now" -lt "$deadline" ] || break
    sleep 0.01
  done

  for pid in "${survivors[@]}"; do
    snapshot=$(state_form_pid_snapshot "$pid" 2>/dev/null || true)
    [ -n "$snapshot" ] || continue
    state=${snapshot%% *}
    start=${snapshot#* }
    [ "${start_by_pid[$pid]-}" = "$start" ] || continue
    case "$state" in
      Z|X) ;;
      *) kill -KILL "$pid" 2>/dev/null || true ;;
    esac
  done

  for pid in "${active_ref[@]}"; do
    wait "$pid" 2>/dev/null || true
    index=${index_ref[$pid]}
    log=${log_ref[$pid]}
    tail_text=$(tail -n 3 "$log" 2>/dev/null || true)
    detail_ref[$index]="$(basename "${shard_ref[$index]}"): $reason
$tail_text"
  done
  active_ref=()
}

run_pin_shards() {
  local kb=$1
  shift
  local -a all_shards=("$@")
  local -a active=() remaining=()
  local -A index_by_pid=() log_by_pid=() summaries=() details=()
  local total=${#all_shards[@]}
  local next=0 index pid finished_pid wait_rc failed=0
  local shard log tail_text scheduler_detail=""

  while [ "$next" -lt "$total" ] \
        && [ "${#active[@]}" -lt "$STATE_FORM_MAX_PARALLEL" ]; do
    shard=${all_shards[$next]}
    log="${shard%.pins.nibli}.out"
    "$PIN" --allow-shell --kb "$kb" "$shard" >"$log" 2>&1 &
    pid=$!
    active+=("$pid")
    index_by_pid["$pid"]=$next
    log_by_pid["$pid"]=$log
    next=$((next + 1))
  done

  while [ "${#active[@]}" -gt 0 ]; do
    finished_pid=""
    wait_rc=0
    wait -n -p finished_pid "${active[@]}" || wait_rc=$?
    if [ -z "$finished_pid" ]; then
      scheduler_detail="state-form scheduler could not identify a completed child"
      failed=1
      cancel_pin_shards \
        "cancelled after scheduler wait failure" \
        active index_by_pid log_by_pid details all_shards
      break
    fi

    index=${index_by_pid[$finished_pid]}
    log=${log_by_pid[$finished_pid]}
    remaining=()
    for pid in "${active[@]}"; do
      [ "$pid" = "$finished_pid" ] || remaining+=("$pid")
    done
    active=("${remaining[@]}")

    if [ "$wait_rc" -eq 0 ] && grep -q 'PASS' "$log"; then
      summaries[$index]="$(basename "${all_shards[$index]}"): $(tail -n 1 "$log")"
    else
      tail_text=$(tail -n 3 "$log" 2>/dev/null || true)
      if [ "$wait_rc" -eq 0 ]; then
        details[$index]="$(basename "${all_shards[$index]}"): no PASS verdict
$tail_text"
      else
        details[$index]="$(basename "${all_shards[$index]}"): execution failed (exit $wait_rc)
$tail_text"
      fi
      failed=1
      cancel_pin_shards \
        "cancelled after another shard failed" \
        active index_by_pid log_by_pid details all_shards
      break
    fi

    while [ "$next" -lt "$total" ] \
          && [ "${#active[@]}" -lt "$STATE_FORM_MAX_PARALLEL" ]; do
      shard=${all_shards[$next]}
      log="${shard%.pins.nibli}.out"
      "$PIN" --allow-shell --kb "$kb" "$shard" >"$log" 2>&1 &
      pid=$!
      active+=("$pid")
      index_by_pid["$pid"]=$next
      log_by_pid["$pid"]=$log
      next=$((next + 1))
    done
  done

  if [ "$failed" -ne 0 ]; then
    [ -z "$scheduler_detail" ] || printf '%s\n' "$scheduler_detail"
    for ((index = 0; index < total; index++)); do
      [ -z "${details[$index]-}" ] || printf '%s\n' "${details[$index]}"
    done
    return 1
  fi
  for ((index = 0; index < total; index++)); do
    printf '%s\n' "${summaries[$index]}"
  done
}
