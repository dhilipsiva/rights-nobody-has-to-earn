<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Measure multi-power, multi-window protective authority composition before formalising it

> **Status: completed, source-bound capability audit; measured and independently
> checked on 2026-08-12.** This record constrains later formalisation. It adds no
> constitutional predicate, rule, fact, pin, authority, finding, institution,
> operation, result, liveness, delivery, or release claim.

## 1. Evidence stamp

The handoff record has SHA-256
`ab973c768964178c9aa751e5f377d9e28732f27f1d7b5f34231000c360b69411`.
It measured clean Nibli `main == origin/main == public main` at
`07734c8f7af71075cb70e91c112ff75d16a962d9`, workspace version `0.1.0`,
WIT package `nibli:engine@0.11.0`, with configured origin
`git@github.com:dhilipsiva/nibli.git`. A read-only public query returned the same
source without fetching:

```text
git ls-remote --exit-code \
  https://github.com/dhilipsiva/nibli.git refs/heads/main
```

```text
07734c8f7af71075cb70e91c112ff75d16a962d9	refs/heads/main
```

Both required engine commits were present and were ancestors of the tested
source:

```text
git cat-file -e '4cb02aade43b394374c40e661907ad66df3af3fe^{commit}'
git merge-base --is-ancestor 4cb02aade43b394374c40e661907ad66df3af3fe HEAD
git cat-file -e '5cec80080eea0334c87508e60813f8f70f487441^{commit}'
git merge-base --is-ancestor 5cec80080eea0334c87508e60813f8f70f487441 HEAD
```

All four commands exited `0`. The release build wrote outside the engine
worktree:

```text
cd /home/dhilipsiva/projects/dhilipsiva/nibli
/usr/bin/time -v \
  nix develop --extra-experimental-features nix-command \
  --extra-experimental-features flakes --command bash -lc \
  'CARGO_INCREMENTAL=0 cargo build --locked --release \
   -p nibli --bin nibli-pin \
   --target-dir /tmp/nibli-multipower-audit.xycP4Y/baseline/target'
```

The build exited `0` after Cargo reported `25.66s`; full wall time was `28.57s`
and peak RSS was `418032 KB`. The resulting
`/tmp/nibli-multipower-audit.xycP4Y/baseline/target/release/nibli-pin` was
2,527,112 bytes with SHA-256
`87b5c7bf351e355352781905a7afedbf29f7c20b3e3d2fc69843921ba0a26f10`.
`nibli-pin --version` is unsupported and exited `1`; the executable is bound by
source, manifest/WIT versions, build command, and binary digest rather than a
runtime version string. The measured environment was Nix `2.35.1`, rustc/cargo
`1.94.0`, LLVM `21.1.8`, Wasmtime `42.0.1`, cargo-component `0.21.1`, and WSL2
Linux `6.18.33.2-microsoft-standard-WSL2`, x86-64.

This repository independently confirmed the engine checkout remained clean at
that exact SHA, recomputed the listed surviving evidence hashes in section 7,
including the `nibli-pin` hash, and ran the full verifier with that exact binary
selected:

```text
NIBLI_PIN=/tmp/nibli-multipower-audit.xycP4Y/baseline/target/release/nibli-pin \
  ./verify.sh
```

The command exited `0` and ended `all checks passed`. This proves compatibility
with the current book repository at the audited boundary. It is not a
repository-wide Nibli test run or a timeless engine guarantee.

## 2. Measured conclusion

At the bound source, one supplied source record can support many separately
keyed measures through exact, case-bound ground queries. The measured rule
retained every antecedent variable in its conclusion:

```text
source_record(act, version)
and measure_record(measure, subject, scope, basis, authority, act, version)
and authority_record(authority, measure, window, Open)
-> bounded_output(
     act, version, measure, subject, scope, basis, authority, window
   )
```

Within that shape:

- each measure rejoins its own authority, subject, scope, basis, window, source
  identity, and exact source version;
- sibling authority cannot substitute;
- removing one authority closes only that measure;
- removing the shared source closes every dependent conclusion; and
- stale versions and absent renewal evidence fail closed.

Only a top-level definitive `TRUE` on that fully keyed supported shape can
support the bounded logical consequence. `FALSE` means not derivable from the
supplied closed-world snapshot; it is not proof of expiry, cessation,
revocation, rejection, real-world absence, or classical negation.
`UNKNOWN(reason)`, `RESOURCE_EXCEEDED(kind)`, invalid input, process failure,
and incomplete enumeration are non-evaluable and cannot authorize anything.

The engine does not decide which source version or authority record is current.
It does not authenticate a writer, detect replay, advance a clock, ensure
publication, schedule or compel review, reconcile inconsistent current-state
records, execute an institutional act, or establish that any route remains live.

## 3. Capability and surface matrix

| Capability | Measured result | Boundary |
| --- | --- | --- |
| Exact ground multi-measure composition | Definitive `TRUE` | Every body variable was carried through the derived tuple |
| Sibling-authority isolation | Definitive `FALSE`, CWA | Exact measure/authority mismatch |
| One-measure nonrenewal or retraction | Definitive `FALSE`, CWA | Other measures remained `TRUE` |
| Shared source removal | Every dependent query `FALSE`, CWA | Not affirmative revocation, expiry, or execution |
| Exact source-version rejoin | Enforced by the tested rule | Selection of the current version remains caller policy |
| Compact status tag | Did not bridge the tested mismatch | Tags have no privileged semantics; another rule could give one meaning |
| Duplicate renewal | One logical witness, two provenance citations | Replay versus live observation remains external |
| Frozen window across snapshots | Same definitive `TRUE` twice | Nibli has no advancing clock |
| Alternate or substitute route | Definitive only after all positive evidence | Does not prove publication, action, or institutional liveness |
| Instrument non-cascade | Definitive in the tested rules | Future rules can change the compiled dependency graph |
| Compiled no-reader check | Mechanically observable | Predicate-level and current-program only, not an admission seal |
| Eight-variable unbound `find` | Unsafe process-level memory exhaustion | Corresponding count/aggregate consume `query_find` |
| Flat raw antecedent-only witnesses | Silent definitive false negative | Unsupported raw-IR shape |
| Arbitrary neutral text schema | Refused by the closed compiler | Native raw IR remains available |

| Surface | Exposes | Does not expose |
| --- | --- | --- |
| `nibli-pin` | Text assertion/refusal, `TRUE`/`FALSE`/`UNKNOWN`, `--kb`, `--strata` | Proof, raw IR, find/count/aggregate, pinnable `RESOURCE_EXCEEDED` |
| `NibliEngine` | Text query/proof/find/count/aggregate, retraction, `.kb()` | Dedicated raw-buffer query wrapper |
| `CoreSession` | Text operations, raw-buffer assertion/replay, `.kb()` | Dedicated raw-buffer query wrapper |
| `KnowledgeBase` | Raw assertion/query/proof/find/count/aggregate and graph report | Authentication, clock, publication, scheduling, or execution |
| WIT/component | Text query/proof/find and raw assertion | Raw query, count, aggregate, stratification report |
| Shipped host | `?` proof and `??` find | Raw query, count, aggregate, stratification report |

Proofs preserve assertion and rule citations and establish derivation from
admitted premises only. Timings and compiled-graph checks are observational.
Native unbound enumeration terminated at process level rather than returning a
typed `ResourceExceeded`. Aggregate silently ignores symbolic or missing
numeric bindings, so a caller must not treat its output as a complete threshold
total without validating every returned binding.

## 4. A — one source, many separately renewed measures

The functional probe produced:

```text
initial-0                                      True
initial-1                                      True
initial-2                                      True
sibling-authority-non-substitution             False
closed-marker-alone-does-not-override-open     True
unit0-open-removed                             False
unit1-unaffected                               True
unit2-unaffected                               True
renewed-window-works                           True
old-window-stays-closed                        False
source-removes-unit0                           False
source-removes-unit1                           False
source-removes-unit2                           False
```

The renewed proof was `True`, four steps, `cwa_false=false`. The source-absent
proof was `False`, two steps, `cwa_false=true`. Adding a `Closed` sibling record
did not override a retained `Open` record. Formalisation must retract the active
record or admit a separately resolved current-state record; it must not assign
implicit priority to `Closed`.

The exact-ground scale runs used a fresh knowledge base per repetition:

| Measures | Build min/median/max ms | Selected ground min/median/max ms | All ground min/median/max ms | Peak RSS |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 0.092 / 0.126 / 0.297 | 0.011 / 0.015 / 0.029 | 0.032 / 0.033 / 0.038 | 7,536 KB |
| 100 | 0.702 / 0.718 / 1.218 | 0.081 / 0.089 / 0.119 | 0.315 / 0.318 / 0.343 | 7,744 KB |
| 1,000 | 6.470 / 6.806 / 19.676 | 1.032 / 1.130 / 1.778 | 3.228 / 3.299 / 4.577 | 18,100 KB |
| 5,000 | 34.181 / 35.104 / 61.752 | 5.518 / 6.796 / 7.284 | 16.680 / 17.439 / 18.091 | 75,348 KB |

From 100 to 5,000 measures, median build time increased about 49 times and
all-ground time about 55 times for 50 times the input. On this source, binary,
host, and date, the exact keyed ground seam was approximately linear. The
release build exposes no internal candidate-attempt counter; these are wall
time, result-cardinality, and RSS measurements, not a general complexity proof.

The corresponding eight-variable unbound `query_find` did not share that
behaviour. With one rule, one source, three measures, three authority records,
and three expected rows, the capped run produced the three preliminary ground
checks and then aborted:

```text
CHECK initial-unit-0 True
CHECK initial-unit-1 True
CHECK initial-unit-2 True
memory allocation of 1056 bytes failed
Command terminated by signal 6
Elapsed: 0:03.20
Maximum resident set size: 523100 KB
```

The outer exit was `134` under a `524288` KB virtual-memory cap. An earlier
uncapped observation reached `23,750,628 KB` RSS after 163 seconds but was not a
completed peak receipt. Exact ground performance must not be generalized to
unbound find/count/aggregate.

## 5. B–E — versions, renewals, alternate routes, and non-cascade

### B. Exact versions and compact status

The stale version was definitive `FALSE` with `cwa_false=true`; stale
find/count returned zero rows and zero. An exact matching version returned
`TRUE` with a five-step derivation. `compact_current_tag` was an independently
asserted ordinary fact, was absent from the compiled dependencies, and did not
bridge the stale tuple. Exact equality is structural only where an encoded rule
carries both identity and version. Current-version selection, uniqueness,
authenticity, and correction remain caller policy.

### C. Withheld, duplicated, and frozen renewals

- An absent renewal was definitive `FALSE`, `cwa_false=true`, and derived no
  affirmative terminal marker.
- Duplicate assertions received distinct registry IDs `6` and `7` while
  find/count returned one logical witness and the proof retained both citations.
- Two fresh snapshots containing identical `WindowFrozen` evidence both
  returned definitive `TRUE`.

Nothing measured distinguishes replay from a second live observation or detects
a renewal window that never advances. Freshness, clock progression, sequence,
writer identity, replay protection, publication, and correction belong to the
authenticated external-record seam.

### D. Alternate authoriser and substitute reviewer

```text
alternate without positive unavailability       False, CWA
alternate with positive unavailability          True
unavailable writer via alternate                 True
unavailable reviewer via substitute              True
silence as approval                              False, CWA
vacancy as approval                              False, CWA
conflict without clean witness                   False, CWA
later evaluation without ordinary ratification  False, CWA
later evaluation with ratification               True
```

These results establish derivability only after every required positive record
and action is supplied. They do not prove that somebody will classify or
publish unavailability, that an alternate will act, that ratification will
occur, or that any route cannot be withheld permanently. That last proposition
is an institutional liveness claim and was not established by this audit.

### E. Instrument non-cascade

```text
alpha instrument complete          True
beta instrument incomplete         False, CWA
alpha conclusion filling beta      False, CWA
subject-only consequence           False, CWA
alpha enumeration/count            1 row / 1
```

The compiled graph had independent inputs and no readers of either capability:

```text
alpha_capability -> alpha_authority, alpha_instrument
beta_capability  -> beta_authority, beta_instrument
readers(alpha_capability) = []
readers(beta_capability)  = []
```

`KnowledgeBase::stratification_report()` and `nibli-pin --strata` can support a
mechanical reverse scan after each program mutation. The check is
predicate-level, excludes ad hoc queries, says nothing about future rules, and
is unavailable through the current WIT component and shipped host. It is not a
permanent non-readability seal.

## 6. Residual findings and disposition

Two general engine gaps remain separate from completion of this audit:

1. **High-arity unbound enumeration.** Global candidate expansion exhausted
   memory even though only three rows were expected. This blocks any
   formalisation that requires this unbound find/count/aggregate shape. A
   general repair should extend relation-scoped, left-deep candidate binding to
   ordinary flat positive bodies. A minimal acceptance test should return the
   three rows deterministically in under one second and under 64 MiB
   incremental RSS on the recorded host, with test-only 10/100/1,000-row
   candidate counts scaling linearly.
2. **Flat raw antecedent-only variables.** An accepted event-free raw rule whose
   positive witness identifiers occurred only in antecedents returned a silent
   definitive `FALSE`; carrying those identifiers in the head returned `TRUE`.
   This is a raw-IR conformance blocker. A general repair should bind remaining
   positive variables relation-by-relation or reject the unsupported rule shape
   at ingress. Silent under-derivation is not an acceptable contract.

Two narrower follow-ups do not block the supported exact-ground seam:

3. **Permanent non-readability.** External parsing of the current compiled graph
   can gate publication after every program mutation. A first-class
   dependency-forbid declaration is needed only if runtime sealing is required;
   acceptance would have to cover positive, NAF, nested, raw-IR, retraction, and
   replay paths.
4. **Aggregate numeric projection.** The API documents `None` only when no
   numeric witnesses exist but silently filters symbolic or missing bindings
   from a mixed result. Documentation should state that projection explicitly,
   or a strict variant should fail when any returned witness lacks the requested
   finite numeric binding.

No authority-specific engine feature is warranted. Formalisation can use the
safe bounded seam only if it remains in exact keyed ground queries, carries all
positive witnesses through the derived tuple, admits resolved authenticated
external records, and treats every non-definitive or process-level failure as
non-evaluable. The residual findings must not be relabelled as repaired.

## 7. Evidence digests and durability

| Artifact | SHA-256 |
| --- | --- |
| Handoff record | `ab973c768964178c9aa751e5f377d9e28732f27f1d7b5f34231000c360b69411` |
| Release `nibli-pin` | `87b5c7bf351e355352781905a7afedbf29f7c20b3e3d2fc69843921ba0a26f10` |
| Release build log | `4269cc7880343df7eabf60937cb5510d7e131cd9aed862437c3b35ae17266630` |
| A bounded fixture | `dd6e52d0746712c95acf24b55f2caeb9aee6e3864982c99113993f0ed0ab36f2` |
| A unbound fixture | `838aa782c3d7633b01cf23a079a08e99e18691854853856e8a4e2e210213786b` |
| A capped runner | `e30238c2a88ea3bc247acd5263879ec5aac6959d4d65d5bc92f9658e22f2a0fb` |
| A functional transcript | `e37469162d623f8d0f9cb5c4773238928248d06ec0aba9ca73066f57c948a120` |
| A scale-10 transcript | `cafc385c34b8effe02390c332160918de8c8e118571ed35276381f57bf3257ad` |
| A scale-100 transcript | `08d92f24baeb719a571b77550096fd3893fa96d97ea6d829284604d8d94cdd79` |
| A scale-1,000 transcript | `f594fc19da069e251115d09a163a0f454f55c8166743e69146bbce0f65c945fc` |
| A scale-5,000 transcript | `b7a39f3d7f65a4323326cf81ab1bede788de4a97e5654a0b447f346590cec76c` |
| A capped unbound transcript | `cacefd23537fee03b89329e904dca46df999292575b76bf6b402bb93f100f5fe` |
| B–E fixture | `621f4c55ae0c684b233f069b87ecfe949595580cd00cad902904b8b807d003f9` |
| Raw body-only control | `6b7d72b40ee3d927dfb4fc6c150365d0b8c194979921c4f919794472505c745c` |
| Neutral text boundary pin | `ba2666ef740141ad994c6b3aa37c32f80d36683179904ef084087a85472f816b` |
| B–E stdout | `d8256511344e97fc723f803b2ceafba92e9af8bbbb8a96780d3e5b306040207c` |
| B–E stderr | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| Raw body-only stdout | `08af786c1f1090970daaeca512ddeba43737d405cc0f02512ce4d9d718b3249c` |
| Neutral text-boundary stdout | `75414eda8cb2dc990a3a476bba97d1ba1b4daaafab22db7f09aeeba5a051fc0f` |

The listed probe sources and transcripts, and the listed release `nibli-pin`,
remained under `/tmp/nibli-multipower-audit.xycP4Y` when this record was written;
their listed hashes were independently recomputed. Custom probe executable
binaries also remained there, but were not digest-bound in this record. None of
those paths is durable. This record preserves measured conclusions and listed
digests; it does not claim permanent reproducibility after `/tmp` cleanup. The
strongest future closure would convert the two decisive general engine findings
into licensed upstream regression tests or retain a reviewed evidence bundle.

The handoff omitted several exact build and probe command lines from its prose;
the surviving build log and transcript headers contain them. The complete
before/after `git show-ref --head` preimages were likewise represented by their
common digest rather than retained in the handoff. These are evidence-durability
limitations, not stronger claims supplied by this record.

## 8. Mutation and formalisation boundary

The handoff recorded identical before/after engine status, unstaged and staged
diffs, refs, and remote configuration. The common before/after
`git show-ref --head` digest was
`ab634043111045f4af7d18a3e7ab0622bdd7fd03c2d7529163573f65c952e8b4`.
Before and after status was:

```text
## main...origin/main
```

Both `git diff --exit-code` and `git diff --cached --exit-code` exited `0` with
no output. No engine file, generated artifact, index entry, branch, ref, tag,
remote, commit, or documentation changed; no fetch, commit, or push occurred.

This audit closes only the read-only measurement prerequisite. The protective,
emergency, and force families remain unimplemented. Their later formalisation
must:

- use the exact-ground, fully keyed seam and avoid high-arity unbound
  enumeration and flat raw body-only-witness rules;
- resolve current-state conflicts externally rather than assume `Closed`
  overrides a retained `Open` record;
- keep authentication, completeness, freshness, replay detection, clock,
  classification, publication, correction, scheduling, challenge, and
  institutional execution as separately admitted or operational
  responsibilities;
- preserve `FALSE`, `UNKNOWN`, `RESOURCE_EXCEEDED`, invalid-input, and
  process-failure distinctions;
- treat alternate routes as derivability after supplied positive evidence, not
  proof that no actor can withhold permanently;
- recheck the compiled no-reader condition after every program mutation if that
  external gate is selected; and
- rerun the full engine suite at the exact selected source and this repository's
  full verifier with the exact selected binary before formal rules merge.

The audit also corrected unsupported assumptions in its prompt: concurrent
windows are opaque supplied terms rather than engine time; duplicate or frozen
renewals are not intrinsically recognizable as replay; exact-version equality
belongs to the encoded rule rather than a global invariant; a current no-reader
property is checkable but not permanently sealed; and exact ground performance
does not establish safe unbound enumeration.
