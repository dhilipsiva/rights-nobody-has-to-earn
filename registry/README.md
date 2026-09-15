# The claim registry

Every empirical figure the books rely on, machine-readable, with its
provenance pinned. The registry exists because the alternative was measured,
twice: a hand-assembled research brief drifted into a spliced quotation, a
superseded working paper, wrong units and a garbled comparison — ten
corrections in one day (commits `773ef68..60f1a85`). Numbers here are cited by
**id** from prose and never restated inline, so a correction lands in one
place.

## Licence

CC0-1.0 (`LICENSE-CC0`), deliberately and irrevocably: `../LICENSING.md`
commits the registry to CC0 so any reader can re-run, extract and republish
it without asking. The fetch and check scripts beside it are code and carry
`MIT OR Apache-2.0` SPDX headers.

## Format

`claims.json` — a `spec` block documenting the fields, then `claims`, one
entry per figure. Two classes:

- **Pinned** (`fetch: null`): papers and reports. The pinned version is the
  provenance; `retrieved` records when the pinned version was last checked.
  These never go stale by date — moving one (working paper → journal) is a
  human re-cite pass, recorded by editing the entry.
- **Fetchable** (`fetch: "<script> <args>"`): sources with APIs (World Bank,
  WHO GHO, OWID, FAOSTAT, …). The named script under `fetch/` writes `value`
  and stamps `retrieved`; hands never do.

## Checks

```
python3 registry/check.py
```

Schema plus the staleness gate: a fetchable entry whose `retrieved` date is
older than the gate's window fails, naming the script that refreshes it. Run it
when you edit an entry. It is **not** part of `../verify.sh`: the 2026-09-12
author decision took registry, hash, history and report-freshness gates out of
verification, which now runs pins and contradiction scans only. Source quality
and the re-cite pass remain editorial, not mechanical.

## Refreshing a fetchable entry

```
python3 registry/fetch/worldbank.py EG.ELC.ACCS.ZS WLD --write registry/claims.json --id example-worldbank-electricity-access
```

## What does not belong here

EIU-derived values, ruled 2026-08-02: the index is non-redistributable and this
registry is CC0, so cite-and-link was refused and the democracy/happiness
analysis was re-derived on openly licensed OWID series instead
(`vdem-2026-*`). Two entries keep the paper trail without carrying a value —
`eiu-2026-democracy-index` as the comparator reference book-2 may cite-and-link,
and `demo-happy-prior-analysis` as the provenance of the superseded prior
analysis. Nothing in this registry depends on either.
