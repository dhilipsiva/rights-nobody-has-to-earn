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
../verify.sh --quick
```

Schema plus the staleness gate: a fetchable entry whose `retrieved` date is
older than the gate's window fails the build and names the script that
refreshes it. The repository-root verifier runs this check when the registry is
present.

## Refreshing a fetchable entry

```
python3 registry/fetch/worldbank.py EG.ELC.ACCS.ZS WLD --write registry/claims.json --id example-worldbank-electricity-access
```

## What does not belong here

The EIU-based democracy/happiness derivation stays out until its licensing
question is ruled (EIU is non-redistributable; V-Dem re-derivation is the
open alternative — see `../TODO.md`, Data section). Nothing in this registry
may depend on that ruling.
