<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Book 1 Public-Edition Boundary Decision

> **Status: author-ratified 2026-08-04 — E2 + P1 + D2; refined
> 2026-08-07 by the full-society boundary.** This is a publication decision,
> not a constitutional change. It qualifies the 2026-08-02 reach ruling and
> does not authorise changing a public artifact in place. The later ruling
> defines “completed expansion” as cumulative Gate C completion and controls if
> gate timing conflicts.

## 1. What is being decided

The source is already public under irrevocable open licences. Measured
2026-08-04, however, the repository has no edition tag, GitHub Release,
assembled reader artifact, canonical site, homepage, or immutable public-book
URL. Public source is not yet a citable edition.

The 2026-08-02 reach ruling originally assigned the then-pass-complete T0
manuscript to spine-order serialization, public red-team review, and an
assembled capstone. The 2026-08-03 mandate creates a different future artifact:
an expanded constitution, regenerated spine, new tests, possible reordering,
and revised prose. A completed pass is therefore version-specific. E2 preserves
the public-review and capstone strategy but applies them to the expansion rather
than promoting that T0 manuscript.

At ratification, that tree was a **candidate baseline**, not and never a First
Edition under E2. The evolving source now contains the staged T1/T2/T3 custody
path, but it remains an expansion draft: implementation does not promote it,
freeze its chapter order, or satisfy the separate release gate. The last page's
publication-order promise also remains a pre-edition issue.

On 2026-08-04, the author decided:

1. the candidate baseline does **not** become a promoted edition;
2. coherent, immutable expansion snapshots may be published before completion;
3. the Gate C-complete expansion becomes both Book 1's First Edition and its
   first print-on-demand edition.

The author-ratified 2026-08-07 full-society boundary now defines “completed
expansion” as **cumulative Gate C completion**. The companion decision
`full-society-boundary-decision.md` controls the cross-volume seam, Gate A–E
public labels and claims, and versioned stopping/reopening. This file continues
to control Book 1 publication mechanics.

## 2. Terms that must not be blurred

- **Source:** the evolving public repository. `main` is never an edition URL.
- **Release candidate:** an immutable, tagged source snapshot used for private
  serialization, holdout, and release checks before a final edition.
- **Edition:** a final, immutable assembled artifact with a stable identity.
- **Preview:** an immutable, non-final snapshot of expansion work. It is not a
  promise of final chapter order.
- **Latest:** mutable navigation to a newer object. It is never a citable source.
- **Withdrawn:** no longer recommended by the canonical site, not erased from
  history or recalled from mirrors.

In reader-facing material, use **Book 1 — First Edition** and **Book 1 — Second
Edition**, not bare “v1” and “v2,” which can be confused with Book 2. Namespaced
machine tags may use forms such as `book-1-v1.0.0`. Under E2, the Gate C-complete
expansion is Book 1 — First Edition; “Second Edition” is reserved for a later
major revision, not used as another name for the expansion.

## 3. Non-negotiable publication contract

Every release candidate, edition, or preview must carry the following contract
in the artifact itself or in inseparable release metadata.

1. **Exact identity:** human-readable status; immutable namespaced tag; full
   book-repository commit; canonical versioned URL; and hashes of every
   assembled artifact. A tag is never moved or reused.
2. **Reproducible inputs:** exact, clean nibli commit; ordered prose-input
   manifest; registry snapshot; build command; and licences. The book commit
   alone is insufficient because `verify.sh` currently builds an adjacent,
   mutable nibli checkout.
3. **Verification record:** full `./verify.sh` result against those exact clean
   inputs, command, date, retained transcript, and known external-assurance
   limits. A release run may not contain `+uncommitted`, use an unidentified
   binary override, or fall back silently to an existing binary.
4. **Plain scope:** historical T0 baseline, staged expanded draft, or completed expanded
   interface; implemented guarantees; known defects; and what remains external.
   “Verified” describes checked formal and editorial claims, not successful
   delivery in society or independent validation of the engine.
5. **Stable revision path:** immutable version URLs and assets, public
   changelog, errata route, supersession links, and no silent page replacement.
   Only an index or `latest` pointer may move.
6. **Correction and withdrawal:** every content change creates a new tag,
   artifact, and version. A critical defect may receive an immediate warning or
   withdrawal notice on the mutable index; the old artifact remains identified
   and accessible unless a legal, privacy, or security duty requires removal,
   in which case a dated tombstone records what was removed and why. Openly
   licensed copies elsewhere cannot be recalled.
7. **Print provenance:** every printed copy names the edition, publication date,
   source commit, licence, canonical errata URL, and print-file identity. A
   changed interior is a new version, never a replacement under the same name.
   Release candidates and previews are not print editions.
8. **Deferred-work pointer:** Book 2 is defined by scope, not promised
   publication order. An already frozen edition keeps its text and receives a
   dated status note; an unfrozen candidate corrects the promise before release.
   An expanded Book 1 is not the separate operational Book 2.

A generated release manifest may record the final tag, commits, and hashes
outside the tagged source tree; this avoids requiring a source file to contain
the hash of the commit that contains it.

## 4. Ratified choice 1 — canonical baseline

| Choice | Benefit | Cost / condition |
| --- | --- | --- |
| **E1. Give the pre-expansion T0 baseline its own First Edition.** | Preserves a reproducible object, honours the existing serialization ruling, and lets outside review attack a stable claim set. | It must be labelled as narrow and later superseded; the pre-freeze and release-candidate gates apply. |
| **E2. Withhold a promoted edition until expansion is complete.** | Avoids giving the narrow design a durable canonical identity. | Cannot make the manuscript private; it remains in public git history. It gives up baseline serialization as the planned red-team route, so another review route and a clear completion event are required. The expanded work would become Book 1’s First Edition, not its Second. |

**Ruling: E2.** The author rejected the draft recommendation to give the
pre-expansion T0 baseline a separate First Edition. It remains public source and git
history, but receives no canonical serialization, assembled edition, edition
tag, or print identity. The Gate C-complete expansion becomes Book 1's First
Edition. P1 supplies the replacement public-review route. No additional rationale was
supplied; do not invent one.

## 5. Ratified choice 2 — expansion previews

| Choice | Benefit | Cost / condition |
| --- | --- | --- |
| **P1. Permit immutable expansion snapshots.** | Restores public red-team value while the larger redesign is being built. | Each snapshot needs its own full gate and permanent URL. It is a design preview, not final spine-order serialization. Under E1, begin these after the First Edition capstone so two moving public narratives do not compete. |
| **P2. Keep expansion private until its final release candidate.** | Simplifies the reader-facing story and avoids provisional-order confusion. | Gives up review during the longest and riskiest design phase. |

**Ruling: P1.** Publish milestone snapshots, not a mutable living page. An
individual preview chapter may be linked only as part of a tagged coherent
snapshot whose provisional order and supersession status are explicit. Because
E2 creates no baseline capstone, Book 1 previews may begin only after Gate B and
their snapshot-specific gates pass. Every public pre-Gate-C object remains a
preview; a release candidate may be used privately for holdout and release checks
but cannot acquire an edition identity early.

## 6. Ratified choice 3 — print boundary

| Choice | Benefit | Cost / condition |
| --- | --- | --- |
| **D1. Print the narrow baseline if it receives a First Edition.** | Gives the verified baseline a durable, accessible physical form and fulfils the original reach ruling sooner. | Physical copies will outlive their scope warning and may be mistaken for the comprehensive redesign. This option has no object under E2. |
| **D2. Make the Gate C-complete expansion the first POD edition.** | Reserves the most durable format for the mandate the author now intends Book 1 to fulfil. | Delays the print-on-demand companion and forgoes physical reach for the baseline. |

**Ruling: D2.** The pre-expansion T0 baseline receives no print edition. The
Gate C-complete expansion is the first physical Book 1 as well as its First
Edition. Gate C publication is atomic: if the matching source, digital artifact,
POD identity, provenance, or release record fails, the object remains a preview.

## 7. Gate before any public expansion snapshot

P1 does not authorise publishing current HEAD or an incoherent intermediate
state. Gate B and every snapshot-specific condition below must pass before the
first expansion-preview tag:

1. **Align current time claims.** Completed for the staged custody path on
   2026-08-05: the constitution and affected prose distinguish supplied-record
   safety from outside clock and publication liveness. Recheck this boundary
   whenever another public power gains a temporal contract.
2. **Audit standing claims in all public entry points.** The root README says no
   registration is required, and the opening note says rights attach from the
   day a person exists, while universal unregistered service remains expansion
   work. Correct or explicitly qualify every such claim. First-person changes
   in the opening note remain author-supplied.
3. **Remove the order promise.** The author supplies an order-neutral,
   scope-only replacement for the final-page “next book” wording, preserving
   Book 1’s single Book 2 pointer and the voice boundary.
4. **Define the artifact.** Create a machine-readable ordered-input manifest and
   a reproducible build for versioned Markdown/HTML/PDF/EPUB outputs as actually
   offered. A repository archive is not the book: it also contains legacy
   manuscripts, reviews, plans, and verification files.
5. **Lock verification.** Pin or otherwise reproduce the exact nibli source,
   run the full suite from clean book and engine trees, retain the result, and
   prove the public “run it yourself” path works from documented inputs.
6. **Verify the ratified closure hardening.** L1 + D1 must be implemented and
   pass the current full suite and counterfactual fixtures before the first
   snapshot. A preview cannot imply that a ratified current-kernel change has
   landed unless its exact source and verification record contain it.

## 8. Ratified release sequence under E2 + P1 + D2

1. Keep the pre-expansion T0 baseline in public source and history without an edition
   tag, canonical serialization, assembled release, or print file.
2. Build the expansion one bounded rule family at a time under the mandate,
   taxonomy, T3 gate, coverage contracts, and narrowness-impact gate.
3. When a coherent milestone has passed Gate B, Section 7, and the fresh gate
   below, publish it as an immutable First-Edition preview such as
   `book-1-v1.0.0-preview.1`. A later preview receives a new tag and URL; it
   never replaces the prior snapshot.
4. After the expanded constitution and spine are frozen, create a First-Edition
   release candidate for private holdout and release checks. Serialize its
   chapters in computed order, with every whole-chapter pass rerun against that
   exact candidate. Any public pre-Gate-C object remains labelled a preview.
5. Close review on an explicit published event, resolve every release-blocking
   finding, and rerun the exact release gate. Only after cumulative Gate C
   passes may the project tag `book-1-v1.0.0`.
6. Atomically publish the assembled digital capstone and matching POD files.
   Both identify the same final source, Gate C closure record, and release
   manifest; a failure in either artifact or its provenance leaves a preview.
7. After final release, typography or metadata-only corrections use patch
   versions; a correction that changes a public claim uses a visibly named
   corrected edition. Any later constitutional/spine redesign is a new major
   edition.

## 9. Fresh gate for every expansion snapshot

No expansion rule family inherits baseline publication clearance. Before a
snapshot is public, it must have passed Gate B and needs completed coverage and
taxonomy contracts,
constitution/spine/pin and counterfactual validation, full-suite verification,
and the coverage map’s narrowness-impact disposition.

“Affected” includes indirectly falsified claims, changed spine positions, Part
V verdicts, method-part refusal or uniqueness claims, registry entries, and
counted-claim guards—not only chapters whose own derivation changed. A preview
pass never substitutes for validation against the final expanded source.

## 10. Ratification record

The author selected one option on each axis on 2026-08-04:

- [ ] **E1 — distinct narrow First Edition.**
- [x] **E2 — no promoted edition until expansion is complete.**
- [x] **P1 — immutable expansion snapshots.**
- [ ] **P2 — no public expansion snapshots before the final candidate.**
- [ ] **D1 — POD for the narrow First Edition.**
- [x] **D2 — first POD is the expanded edition.**

Ratified composite ruling: **E2 + P1 + D2**.

This ruling closed the 2026-08-04 baseline, preview, and Book 1 POD choice axes.
It did not settle the later full-society volume, claim, or stopping boundary;
that author gate was separately ratified on 2026-08-07 in
`full-society-boundary-decision.md`. Neither ruling itself creates a tag,
release, site, preview, or print file; those remain ordered Reach implementation
work in `TODO.md`.
