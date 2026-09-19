<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# The claim registry

The registry records empirical claims, their values, units, sources, methods
and limitations. Part V's figures are written in prose and linked to sources
through its footnotes. Stable registry IDs connect those passages to the
existing bindings in
[claim_discipline_tests.rs](../src/authoring/claim_discipline_tests.rs).
Do not replace readable figures or citations with internal IDs.

For the submission and review workflow, see the
[contribution guide](../CONTRIBUTING.md). The registry supports evidence review;
it does not make a claim true merely by containing it.

## Edit a claim

`claims.json` contains a `spec` block describing the fields and a `claims`
array. Reuse the ID when correcting the same claim. State the quantity being
measured, its population, period, units and denominator, and keep the caveats
with it. Include the primary source's title, version, URL and a page, table or
other locator that lets a reader inspect the support.

For a paper or report (`fetch: null`), identify the actual edition checked.
An old retrieval date does not itself invalidate a historical result; a
pinned version also does not rule out a correction, retraction or better
interpretation. Check the source when revising the claim. Set `retrieved`
only to the date of an actual check.

For a fetched value, use the script named in `fetch` and inspect its output.
Do not invent a value or advance its retrieval date without fetching it.
Review changes in definitions, units and observation years before carrying
an updated value into prose. Preserve the reproducible inputs and their
upstream terms; the [snapshot instructions](data/README.md) explain the
bundled democracy and life-evaluation example.

Update the affected prose and footnotes with the entry. Maintain the existing
claim binding when its identifying phrase or source changes. A new named case
or figure needs corresponding coverage in the existing development checks.
A correction can narrow or remove an unsupported claim; it need not replace
it with another number.

## Check the change

Compare the claim directly with its source, including the qualifications
needed for the comparison made in the book. Then run the relevant existing
claim-discipline development checks described in the contribution guide.
Those checks bind passages to entries and locators; they do not read the
papers or establish that an inference is sound. The complete constitutional
verifier remains separate from this evidence review.

`check.py` is a retained field/date utility. Its schema and age results do
not validate the cited evidence, and it is not a mandatory completion gate
or part of `../verify.sh`. No receipt or freshness audit is required.

## Licences and scope

The claim registry is CC0-1.0 under [LICENSE-CC0](LICENSE-CC0). Fetch and check
scripts are MIT OR Apache-2.0. Snapshots under `data/` retain their stated
upstream licences. This new guidance text is CC BY 4.0; prior CC0 grants remain
in force. See the repository's [licence map](../LICENSING.md).

Entries identify their book scope. Book 2 remains collection-only until
Book 1's release gate; a collected claim is not a completed operational plan.
The registry's existing policy excludes EIU-derived values. The retained
`eiu-2026-democracy-index` and `demo-happy-prior-analysis` entries are reference
and historical records, not data supporting the current open-data analysis.
