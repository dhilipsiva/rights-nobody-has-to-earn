<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Measure the versioned ecological and animal finding boundary before formalising it

> **Status: completed, source-bound capability audit; measured and independently
> checked on 2026-08-11.** This record constrains later formalisation. It adds no
> constitutional predicate, rule, fact, pin, finding, institution, result,
> operation, or release claim.

## 1. Evidence stamp

The handoff record has SHA-256
`03f0fab967d66814de5761dc958db344e92da1e3d1bca902c1962e0865d6b50e`.
It measured clean Nibli `main == origin/main == public main` at
`07734c8f7af71075cb70e91c112ff75d16a962d9`, workspace package version
`0.1.0`, WIT package `nibli:engine@0.11.0`, with configured origin
`git@github.com:dhilipsiva/nibli.git`. The tested commit is the 2026-08-09
decision `decide: external compute names are query-only; registration refuses
live references`.

Both required commits were present and were ancestors of the tested source:

```text
git cat-file -e '4cb02aade43b394374c40e661907ad66df3af3fe^{commit}'
git merge-base --is-ancestor 4cb02aade43b394374c40e661907ad66df3af3fe HEAD
git cat-file -e '5cec80080eea0334c87508e60813f8f70f487441^{commit}'
git merge-base --is-ancestor 5cec80080eea0334c87508e60813f8f70f487441 HEAD
```

All four commands exited `0`. A read-only `git ls-remote --exit-code
https://github.com/dhilipsiva/nibli.git refs/heads/main` returned the same
`07734c8f7af71075cb70e91c112ff75d16a962d9` without fetching.

The release build wrote its artifacts outside the engine worktree:

```text
cd /home/dhilipsiva/projects/dhilipsiva/nibli
nix develop --extra-experimental-features nix-command \
  --extra-experimental-features flakes --command bash -lc \
  'CARGO_INCREMENTAL=0 cargo build --locked --release \
   -p nibli --bin nibli-pin \
   --target-dir /tmp/nibli-ecological-audit.jMJcEU/target'
```

The resulting `/tmp/nibli-ecological-audit.jMJcEU/target/release/nibli-pin`
was 2,527,112 bytes with SHA-256
`87b5c7bf351e355352781905a7afedbf29f7c20b3e3d2fc69843921ba0a26f10`.
`nibli-pin --version` is unsupported and exited `1`; the executable is therefore
bound by source, manifest/WIT versions, build command, and binary digest rather
than a runtime version string. The measured environment was Nix `2.35.1`,
rustc/cargo `1.94.0`, Wasmtime `42.0.1`, cargo-component `0.21.1`, WSL2 Linux
`6.18.33.2-microsoft-standard-WSL2`, x86-64.

This repository independently confirmed the engine checkout remained clean at
that exact SHA, recomputed the surviving evidence and binary hashes below, and
ran:

```text
NIBLI_PIN=/tmp/nibli-ecological-audit.jMJcEU/target/release/nibli-pin \
  ./verify.sh
```

The verifier used the supplied binary without rebuilding, exited `0`, reported
`555 pins, 0 findings`, including nine pins that encode defects which still
reproduce, and ended `all checks passed`. This proves compatibility with the
current book repository at the audited boundary; it is not a repository-wide
Nibli test run.

## 2. Measured conclusion

At the bound source, current native Nibli can derive explicitly bounded
consequences from caller-admitted, finite, versioned, fully keyed ordinary facts
or result certificates. The safe seam requires all of the following:

- authentication and classification occur before assertion;
- rules explicitly join every relevant case, record, source, source version,
  method/result version, subject, place, jurisdiction, ground, and window;
- rules are fully ground/case-bound or use the ordinary KR/decomposed path;
- callers reject non-finite and mixed-type numeric inputs before enumeration or
  aggregation; and
- admission/retraction resolves conflicting or superseded certificates before
  a consequence is relied on.

Only a top-level definitive `TRUE` on that supported shape can support the
bounded logical consequence. `FALSE` means not derivable from the supplied
closed-world snapshot; it is not proof of safety, real-world absence, rejection,
expiry, preservation, or classical negation. `UNKNOWN(reason)`,
`RESOURCE_EXCEEDED(kind)`, invalid input, and any incomplete enumeration are
non-evaluable and cannot authorize anything.

The audit does not show that Nibli measures, authenticates, classifies, or
resolves ecological or animal evidence. It does not advance time, prove roster
or evidence completeness, select officials or reviewers, resolve competing
certificates, notify anyone, execute a stay or remedy, or calculate a generic
collective threshold. A proof trace proves derivation from admitted premises;
it does not prove that a premise, source, signature, classification, compute
backend, or certificate is authentic.

## 3. Surface boundary

| Surface | Measured capability | Boundary |
| --- | --- | --- |
| `nibli-pin` | Pins text assertions and `TRUE`/`FALSE`/`UNKNOWN` entailment; exact-count can be an ordinary query | No raw IR, proof output, find/count/aggregate directives, or pinnable `RESOURCE_EXCEEDED` |
| `NibliEngine` | Text query/proof/find/count/aggregate, registry/retraction, direct injection | Unknown neutral text vocabulary fails syntax; direct injection is event-decomposed, so a flat raw query is a shape mismatch |
| `CoreSession` | Native operations plus raw-buffer assertion and KB access | No direct raw-query wrapper; callers use `.kb()` |
| `KnowledgeBase` | Raw `LogicBuffer` assertion/query/proof/find/count/aggregate and mutation | Caller owns relation names, arities, keys, schema, and admission correctness |
| WIT/component | Text query/proof/find, typed buffer assertion, direct fact injection, retraction/materialisation | No raw-buffer query, proof, find, count, aggregate, or depth setter; arbitrary component-only schemas cannot round-trip |
| Shipped host | Query/proof, find, fact/debug controls, fuel and memory configuration | No count/aggregate command; resource-result translation is not uniform across operations |
| Compute backend | Query-local Boolean/error evidence | No source identity, signature, version, freshness, replay protection, revocation, or durable fact identity |

Exact-count and executable-compute nodes are query-only. They cannot be stored
as facts or rule literals, and a returned count does not become a stored rule
antecedent. Logical duplicate assertions retain distinct registry identifiers
and citations but contribute one logical witness.

## 4. Probe results

| Probe | Measured result | Consequence boundary |
| --- | --- | --- |
| A — multidimensional findings | Independent pass and breach both derived; an overall scalar `Good` produced no dimension result; eleven one-field scope mismatches were definitive `FALSE` with `cwa_false=true` | Completeness, authentication, and non-substitution beyond written joins remain caller policy |
| B — missing and non-definitive evidence | Missing, partial, adverse-only, and unresolved competing evidence were CWA `FALSE`; unavailable compute was `UNKNOWN(BackendUnavailable)`; the depth horizon was `RESOURCE_EXCEEDED(Depth)` | None is positive authorization; non-definitive enumeration cannot be treated as a complete zero |
| C — objection, replay, and review | Exact positive prerequisites derived status; an alternate required a designation and acting certificate; ordinary initiation/request did not create automatic status; judicial status required a separate asserted result | Objection authenticity, credibility, current window, novelty/finality, reviewer selection, notice, and execution remain external |
| D — separated advocates | Two independent records, eight visible axes, and one exact adjudicated certificate derived the selected result | Neither advocate has priority; the caller must supply one current, conflict-free adjudicated certificate |
| E — protected subject/use | Ordinary, enhanced, food, and categorical research-refusal routes derived only from their separately keyed findings | Sentience, pain, nutrition, alternatives, purpose, scientific validity, causation, and welfare are external findings |
| F — finite-body certificate | Empty, incomplete, tied, conflict-marked, and wrong-decision certificates did not derive the requested consequence; the exact valid certificate did | Roster/vote authenticity, completeness, thresholds, ties, and the certified result remain external |

Two simultaneous `ValidFinal` certificates for different outcomes derived both
outcomes; Nibli invented no priority. Retraction changed an old result from
`TRUE` to `FALSE`, and asserting the corrected version made the new result
`TRUE`. A missing certificate was CWA `FALSE` and enacted or preserved nothing.
An incomplete two-member snapshot could satisfy `exactly 2`; that did not prove
the roster complete.

At a depth horizon, find/count/aggregate refused because witness enumeration was
incomplete. An observational `CountResult { actual: 0 }` inside a proof did not
override the top-level `RESOURCE_EXCEEDED` result. Consumers must use the
top-level verdict, not extract an internal tally as definitive.

All six individual stored flavours matched exactly: bare, past, present, future,
obligatory, and permitted. A flavored fact did not lift to bare. Nested
temporal-by-deontic wrappers failed closed because one fact or rule literal can
carry only one flavor; separate literals may carry separate single wrappers.
There is no implicit clock.

## 5. Residual engine and surface findings

The handoff called these “three narrower raw/numeric gaps”; the evidence instead
supports four semantic/numeric findings and two surface/documentation
follow-ups:

1. **Flat raw rules with body-only variables.** An accepted event-free raw rule
   with a positive antecedent variable absent from the head returned definitive
   CWA `FALSE`. Carrying that variable into the head, or using the ordinary
   KR/decomposed path, returned `TRUE`. This blocks that raw shape, not the safe
   case-bound seam. A general repair should enumerate remaining positive
   antecedent variables relation-by-relation or reject the shape at ingress.
2. **Flat raw builtin arithmetic overflow.** Flat product overflow returned
   `UNKNOWN(BackendUnavailable)`, while the decomposed numeric-group path
   returned `UNKNOWN(NonFinite)`. The flat path should classify resolved builtin
   overflow before external-dispatch fallback.
3. **NonFinite exact-zero composition.** A NonFinite existential was
   `UNKNOWN`, while find/count/aggregate produced empty/zero/`None` under the
   documented exclusion and `exactly 0` returned `TRUE`. This is an unpinned
   specification edge, not yet an established soundness defect. High-assurance
   callers must reject non-finite admitted values until the composition is
   explicitly decided and regression-pinned.
4. **Aggregate numeric projection.** Aggregate silently discarded symbolic or
   missing numeric bindings and summed the retained numbers. A mixed row set can
   therefore look like a complete total. Callers must establish finite,
   type-complete admission first; a future strict variant should fail or return
   an explicit partial/projection status.
5. **WIT custom-schema parity.** The component can assert a typed buffer but
   cannot query/prove/find/count/aggregate an arbitrary raw buffer. Native
   `KnowledgeBase` is sufficient for the measured seam; component-only custom
   schemas remain blocked unless parity or a schema registry is added.
6. **Two stale comments.** `nibli-reason/src/lib.rs` says derived-only
   declarations survive reset although reset clears them;
   `nibli-pipeline/src/lib.rs` says materialisation has no WIT method although
   the method is exposed. These are documentation-only follow-ups.

None of these findings requires an engine repair for the supported, fully keyed
ordinary-fact/result-certificate seam, provided the integrating caller
mechanically enforces every exclusion above. They do prohibit presenting the
unsafe raw, non-finite, mixed
aggregate, or unsupported component surfaces as formal assurance.

## 6. Evidence digests and durability

| Artifact | SHA-256 |
| --- | --- |
| A–C source | `7f2eaaee10e3b293d99c47805bd9861f27e0e8930c83551b6b1c669dd72cea67` |
| A–C output | `643654e326aa8617eeb7a3a0bfac5b4255bebced07df6152d1d00ea9d607d2e7` |
| A–C binary | `d2f1ca3f472bb1f32b6c88bbdc85ef5cb389cd12fe553ba334f54fd696f021c9` |
| D–F case-bound source | `bf81e2496ceeee162e00eee8d7365ff8741a4f6b9f41356dc5d6ffc6608f34a5` |
| D–F case-bound output | `630b4c509b2cbdb2df4b48e58560af47d6290d18308ed9054e22e7bd5b77fad5` |
| D–F case-bound binary | `2c46f936f7a38dab15774113ab25b574919caebe91c6a5ef055cb83a210b19b1` |
| Native F/surface source | `46d84d2c945700fa898da26b5a910e0f7eb6ae20e9422eea5afa3ff5437a9b47` |
| Native F/surface output | `c57ccda6f4ecef800d1b07ffcaf3c92d1f828731d8af96a7ee02fb51aa3bc825` |
| Native F/surface binary | `d83989851aee6ce0a15319dc6b8279b864da78fdc1b05af516f2da0a8ae09c26` |
| Neutral pin | `e43cc7f24138da9725cb08e22a0779a054a7a4d9c53bb1e301689bd8c85858ef` |
| Finite-count pins | `880ec05f9eb1332b9f7b9638d25a277199ba847bccfa38ed405f31abcb226588` |
| Raw body-only source | `5ec616a394a74ae91fa917aa10a10f39e7f4b2c5f32ed38892f9ab7b543398f3` |
| Raw body-only output | `a26c49d1f66b138dad634f679fd206a62b207b42cd3809dd791043c680630154` |
| Raw body-only binary | `9bf4e6aeaf17fb2155bf4ddc4e2ac8ab44389a4c6921cab7bd35fa662ced9a65` |
| Passing KR body-only control | `71f288b9e4433234305fd5484460538b057fbf9f794c0b95843f8cab758d76bb` |
| Supplementary source | `8c8641984d0dadb0448507b2c037a3eff07da359324ec8cec2be463149c1e54c` |
| Supplementary output | `f1cbefe6f55ecf921c7aa1e3b899158bef8bed113c094e193f8f4b14387547b9` |
| Supplementary binary | `d042e047ac9588c2cd8cb6497a85a7f8066d35cc339964c8b5fb368ce0443c10` |
| Preserved failing source | `0793fc34004e50d52ede530e38cd8493921261f9a783c7c1b9a26789bc0d1d02` |
| Preserved failing stdout | `b89e4aa70da06a9e0e5c556df197c65d629e4de1de59c04780f06b671ea6bb88` |
| Preserved failing stderr | `ada09bca123f5ba407979314cbfa51ef15b92fe72f7456fb6f2d6624b4e49fd8` |

The probe sources, complete outputs, and binaries remained under `/tmp` when
this record was written and their hashes were independently recomputed. They are
not committed fixtures and those paths are non-durable. This record preserves
their measured conclusions and full digests; it does not claim permanent
reproducibility after `/tmp` cleanup. The strongest future closure would convert
the decisive general findings into licensed upstream regression tests or retain
a reviewed evidence bundle.

## 7. Mutation and formalisation boundary

The handoff recorded identical before/after engine status, unstaged and staged
diffs, refs, and remote configuration. No engine file, generated artifact,
index, branch, ref, tag, remote, commit, or documentation changed; no fetch,
commit, or push occurred. The later live-source review likewise found no drift
from `07734c8f`.

This audit closes only the read-only measurement prerequisite. Formalisation may
now draft complete contract cards and rules for the supported seam, but it must:

- keep every scientific, quantitative, classification, authentication,
  completeness, freshness, conflict-resolution, and institutional result as a
  separately admitted external premise;
- avoid the unsafe raw body-only-variable, non-finite exact-zero, and mixed
  aggregate paths;
- name and test the actual deployment surface rather than infer WIT/native
  parity;
- preserve `FALSE`, `UNKNOWN`, `RESOURCE_EXCEEDED`, and invalid-input
  distinctions; and
- run the full engine suite at the exact selected source and this repository's
  full verifier with the exact selected binary before any formal rules merge.

The ecological and animal settlement remains author-ratified but unimplemented.
Its contract cards, formal rule families, pins, counterfactuals, prose,
external evidence services, publication, operation, and acceptance scenarios
remain open.
