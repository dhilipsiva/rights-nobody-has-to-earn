# The V-Dem re-derivation — working record (2026-08-03)

Executes the 2026-08-02 ruling: the democracy/happiness analysis stands on V-Dem,
re-derived, so Part V's worked example is re-runnable by a stranger. Derivation
script: `registry/fetch/vdem_happiness.py` (stdlib only; fetches four OWID grapher
series — CC BY, OWID's processing of V-Dem, the World Happiness Report and the
World Bank — merges latest common year ≥ 2022 by ISO3). Snapshot:
`registry/data/vdem-happiness-2026-08-03.csv`. This file is the successor to
`demo-happy.txt`'s role: the working notes behind the registry entries.

## Primary derivation (N = 141)

- raw: r(polyarchy, ladder) = +0.513, ρ = +0.552, R² = 0.263
- income control: partial r(polyarchy, ladder | log GDP) = **+0.197**
  (the EIU-era figure was 0.195 — the narrowing reproduces almost exactly);
  partial r(log GDP, ladder | polyarchy) = +0.727
- Regimes of the World table (n / mean ladder / sd):
  closed autocracy 22 / 5.10 / 1.25; electoral autocracy 46 / 5.13 / 0.99;
  electoral democracy 45 / 5.72 / 0.95; liberal democracy 28 / 6.81 / 0.48
- step sizes: **+0.02, +0.59, +1.09** — the EIU pattern (+0.16/+0.73/+1.01)
  reproduces: the bottom step buys approximately nothing, the top step is the
  largest. The anti-gradualist reading is robust across instruments.
- floor claim (|residual of ladder~polyarchy| regressed on polyarchy):
  alone b = −0.655, t = −3.37, p = 0.001;
  with log GDP: **b_poly = −0.490, t = −2.17, p = 0.032** while
  b_lgdp = −0.073, t = −1.42, p = 0.158.
  **On V-Dem the floor claim survives the income control, narrowed.**

## The isolation test (same 138 countries, same ladder, same real log GDP)

EIU 2025 index (from the local working CSV; these figures stay OUT of the CC0
registry per the licensing ruling — they live only in these CC-BY working notes):

- raw r = +0.594; partial r | log GDP = +0.165
- floor alone: b = −0.076, t = −3.67, p = 0.0003
- floor with log GDP: b_D = −0.046, t = −1.71, **p = 0.089**; b_G p = 0.090

V-Dem polyarchy on the identical sample: floor with log GDP b_D = −0.497,
t = −2.16, **p = 0.032**.

**Conclusion: the flip is the instrument, not the sample.** On identical data,
EIU's index loses the dispersion-compression effect under the income control
while V-Dem's polyarchy keeps it. Second finding: the original analysis's stark
"income kills it" (democracy p = 0.37, GDP p = 0.011) also depended on using the
WHR *explained-by* GDP contribution as the control; with real log GDP the EIU
picture is marginal on both coefficients rather than decisively income-driven.

## What Part V's worked example now says

The planned arc was: survives raw → narrows sharply under income control → the
floor version fails outright. The honest arc after re-derivation:

1. **Survives raw** — on either instrument.
2. **Survives, narrowed** — the income control collapses the association to
   ~0.17–0.20 on either instrument; what compresses wellbeing across countries
   is mostly material provision. Robust.
3. **The floor version: fails as stated, for a new reason** — its verdict
   depends on which democracy index you measure with (dies-or-marginal on EIU,
   survives narrowed on polyarchy) and on which income proxy you control with.
   A claim whose truth tracks the instrument rather than the world is not
   citable as support, and *that* — not a clean refutation — is what the
   discipline caught. The example is stronger for it: the method demonstrated
   is "we would have loved this claim, and we cannot have it, and here is
   exactly why."

Caveats that travel: the |residual| regression is a crude heteroskedasticity
test (kept deliberately — it mirrors the original analysis exactly); no HC
standard errors; polyarchy is continuous 0–1 while EIU is 0–10, irrelevant to
t/p; sample is the OWID latest-common-year merge, not the WHR country list.

One flag for the author: `democracy_vs_happiness_144.csv` (repo root, committed
before the licensing decision, CC0 under the root LICENSE) contains EIU 2025
index values. Whether an EIU-derived column may sit in a CC0-dedicated repo file
is worth a look on the same grounds as the registry ruling.
