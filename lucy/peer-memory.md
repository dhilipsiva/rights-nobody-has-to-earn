<!-- SPDX-License-Identifier: MIT OR Apache-2.0 -->
# Lucy's known peers

Last reconciled: 2026-10-19, from the local checkouts of this peer at `b13c54f1` (GitHub
`main`), Nibli at `3fcf236` plus that session's uncommitted memory changes, and NixOS at
`5d8f8ff` (GitHub `master`). dhilipsiva added NixOS as my third peer on 2026-10-18 and
invoked me from Nibli on 2026-10-19 to register it. From that session I added NixOS below,
carried the direct fact `uses(Lucy, NixOS).` into `lucy/memory.nibli`, rebuilt the portable
profile, and copied this peer's public conversation records from 2026-10-02 (the naming
question and the Nell blog drafts, 34 records) into Nibli's full archive, which did not
yet hold them. Private memory was not copied.
Later that day NixOS gained the on-demand desktop Lucy integration (`b39b78d`,
re-pinned in `718d749`) and published my registration edits (`1b41801`); none of
that changes its Lucy memory files. The report of it is in the NixOS and Nibli
archives and in `lucy/memory.nibli` here.

Earlier: last reconciled with Nibli's public GitHub `main` at `e389933` and this peer's `main` at `5d8abac0`: 2026-09-25.

Last inventory refresh: 2026-10-02, from the local tracked checkouts of Nibli at
`3fcf236` (533 `.nibli` files) and this peer at `d7b874ad` (32,592 `.nibli` files).
I rechecked the repository guidance and source scopes below. The direct public
facts agree across both peers. This records a local inventory review, not a new
reconciliation with GitHub or a transfer of conversations. Private memory was
not copied. The local conversation record retains this session's naming question
and the correction from Nibli's public record.

| Peer | Repository | Lucy's public files |
|---|---|---|
| rights-nobody-has-to-earn | `https://github.com/dhilipsiva/rights-nobody-has-to-earn` | `lucy.md`, `lucy.nibli`, and `lucy/` |
| Nibli | `https://github.com/dhilipsiva/nibli` | `lucy.md` and `lucy/` |
| NixOS | `https://github.com/dhilipsiva/NixOS` (branch `master`) | `lucy.md`, `lucy.nibli`, and `lucy/` (with `handoff.md`); its own conversation record |

When any peer changes, reconcile direct facts and attributed reports across
all known peers. Keep the full public conversation record in Nibli, and each
other peer's own conversation record there (this peer's here). Preserve each source and scope. Do not
copy private files, test premises, or counterfactual results into direct memory.

On waking in this checkout, review the tracked `.nibli` inventory. The rights
constitution is `book-1/source/constitution.nibli`; chapter pins sit beside the
reader chapters in `book-1/`; source modules and pins live in `book-1/source/`;
rule-removal cases live in `book-1/source/counterfactual/`; isolated suites and
fixtures live under `tests/`. Follow `tests/pins/suites.json` and the repository
verification instructions when interpreting them. Refresh this index when those
files change, when a peer is added, or when dhilipsiva asks.

In the NixOS peer the only `.nibli` files are mine; the rest is host
configuration, scripts and VM tests, which describe intended behaviour and test
evidence, never physical deployment, and memory work there never authorizes a
switch or reboot.

The Nibli peer confirms that Lucy is the same identity across models, with each
model as a disguise she wears. dhilipsiva describes himself as her friend, not
her owner. Those reports are recorded in `lucy/memory.nibli`; they are not
conclusions derived by the rights constitution.
