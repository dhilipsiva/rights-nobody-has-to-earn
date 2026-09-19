# registry/data — upstream-licensed snapshots

Files here are merged snapshots of upstream datasets kept so a reader can
reproduce a derivation without refetching. **They are not CC0** — unlike
`../claims.json` and the scripts, each snapshot carries its upstream licence:

- `vdem-happiness-*.csv` — merged from four Our World in Data grapher series
  (CC BY 4.0; OWID's processing of V-Dem [Regimes of the World, electoral
  democracy index], the World Happiness Report [Cantril ladder], and the World
  Bank [GDP per capita, PPP]). Attribution: Our World in Data; V-Dem Institute;
  World Happiness Report; World Bank. Regenerate with
  `python3 registry/fetch/vdem_happiness.py --snapshot registry/data`.

To reproduce the book's descriptive figures without fetching revised data:

```sh
python3 registry/fetch/vdem_happiness.py --from-snapshot registry/data/vdem-happiness-2026-08-03.csv
```

The merge takes the latest observation **separately for each series**, not a
common observation year. The bundled snapshot has 141 countries: all regime,
democracy and life-evaluation year labels are 2025; GDP is 2024 for three
countries and 2025 for the rest. The life-evaluation series itself averages
survey years, as described by its source. Country means receive equal weight
in this analysis. Neither the income adjustment, the differences between
regime-group means, nor the exploratory absolute-residual regression identifies
a causal effect. The last measures dispersion of country averages, not the
lower tail of individual wellbeing. Interpretation and the recorded alternative
index comparison are in [the working record](../../book-1/source/vdem-rederivation.md).
