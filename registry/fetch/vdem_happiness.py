#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Re-derive the democracy/happiness analysis on V-Dem (ruled 2026-08-02).

Fetches four OWID grapher series (CC BY; OWID processing of V-Dem, the World
Happiness Report, and the World Bank), merges each series' latest observation
by ISO3 code, and derives Part V's descriptive statistics. Observation years
may differ within a country; these are not estimates of regime transitions.

  - raw association: Pearson r, Spearman rho, R^2 (polyarchy x ladder)
  - the income control: partial r (polyarchy, ladder | log GDP), and
    r (log GDP, ladder | polyarchy)
  - the RoW regime table (closed autocracy .. liberal democracy):
    n / mean ladder / sd, and the step sizes between adjacent categories
  - exploratory cross-country dispersion: |residual| ~ polyarchy, then
    |residual| ~ polyarchy + log GDP (slopes, t, p)

Everything is stdlib; p-values via the incomplete-beta t CDF. Usage:

  vdem_happiness.py                 fetch live, print the derivation
  vdem_happiness.py --snapshot DIR  also write the merged CSV to DIR
  vdem_happiness.py --offline DIR   read previously fetched CSVs from DIR
  vdem_happiness.py --from-snapshot FILE  reproduce a merged snapshot, no network
"""
import argparse
import csv
import io
import math
import urllib.request
from datetime import date

GRAPHERS = {
    "regime": "political-regime",            # V-Dem Regimes of the World, 0-3
    "poly": "electoral-democracy-index",     # V-Dem v2x_polyarchy, 0-1
    "ladder": "happiness-cantril-ladder",    # WHR Cantril ladder
    "gdp": "gdp-per-capita-worldbank",       # World Bank PPP per capita
}
ROW_NAMES = ["closed autocracy", "electoral autocracy",
             "electoral democracy", "liberal democracy"]


def fetch(name, offline=None):
    if offline:
        with open(f"{offline}/{GRAPHERS[name]}.csv", encoding="utf-8") as f:
            text = f.read()
    else:
        url = f"https://ourworldindata.org/grapher/{GRAPHERS[name]}.csv"
        with urllib.request.urlopen(url, timeout=120) as r:
            text = r.read().decode("utf-8")
    rows = list(csv.reader(io.StringIO(text)))
    head, body = rows[0], rows[1:]
    vi = 3  # value column follows Entity,Code,Year
    out = {}
    for row in body:
        code, year, val = row[1], row[2], row[vi]
        if not code or code.startswith("OWID") or not val:
            continue
        try:
            y, v = int(year), float(val)
        except ValueError:
            continue
        cur = out.get(code)
        if cur is None or y > cur[0]:
            out[code] = (y, v)
    return out


def mean(xs):
    return sum(xs) / len(xs)


def sd(xs):
    m = mean(xs)
    return math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))


def pearson(xs, ys):
    mx, my = mean(xs), mean(ys)
    num = sum((x - mx) * (y - my) for x, y in zip(xs, ys))
    den = math.sqrt(sum((x - mx) ** 2 for x in xs) *
                    sum((y - my) ** 2 for y in ys))
    return num / den


def ranks(xs):
    order = sorted(range(len(xs)), key=lambda i: xs[i])
    r = [0.0] * len(xs)
    i = 0
    while i < len(order):
        j = i
        while j + 1 < len(order) and xs[order[j + 1]] == xs[order[i]]:
            j += 1
        avg = (i + j) / 2 + 1
        for k in range(i, j + 1):
            r[order[k]] = avg
        i = j + 1
    return r


def spearman(xs, ys):
    return pearson(ranks(xs), ranks(ys))


def partial_r(rxy, rxz, ryz):
    return (rxy - rxz * ryz) / math.sqrt((1 - rxz ** 2) * (1 - ryz ** 2))


def betacf(a, b, x):
    MAXIT, EPS, FPMIN = 200, 3e-9, 1e-30
    qab, qap, qam = a + b, a + 1, a - 1
    c, d = 1.0, 1.0 - qab * x / qap
    if abs(d) < FPMIN:
        d = FPMIN
    d = 1.0 / d
    h = d
    for m in range(1, MAXIT + 1):
        m2 = 2 * m
        aa = m * (b - m) * x / ((qam + m2) * (a + m2))
        d = 1.0 + aa * d
        if abs(d) < FPMIN:
            d = FPMIN
        c = 1.0 + aa / c
        if abs(c) < FPMIN:
            c = FPMIN
        d = 1.0 / d
        h *= d * c
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))
        d = 1.0 + aa * d
        if abs(d) < FPMIN:
            d = FPMIN
        c = 1.0 + aa / c
        if abs(c) < FPMIN:
            c = FPMIN
        d = 1.0 / d
        delta = d * c
        h *= delta
        if abs(delta - 1.0) < EPS:
            break
    return h


def betai(a, b, x):
    if x in (0.0, 1.0):
        return x
    lbeta = (math.lgamma(a + b) - math.lgamma(a) - math.lgamma(b)
             + a * math.log(x) + b * math.log(1.0 - x))
    front = math.exp(lbeta)
    if x < (a + 1) / (a + b + 2):
        return front * betacf(a, b, x) / a
    return 1.0 - front * betacf(b, a, 1.0 - x) / b


def t_pvalue(t, df):
    return betai(df / 2, 0.5, df / (df + t * t))


def ols(y, cols):
    """OLS with intercept. cols: list of predictor vectors.
    Returns (betas incl. intercept, t-stats, p-values, residuals)."""
    n, k = len(y), len(cols) + 1
    X = [[1.0] + [c[i] for c in cols] for i in range(n)]
    XtX = [[sum(X[i][a] * X[i][b] for i in range(n)) for b in range(k)]
           for a in range(k)]
    Xty = [sum(X[i][a] * y[i] for i in range(n)) for a in range(k)]
    # Gauss-Jordan inversion
    aug = [row[:] + [1.0 if i == j else 0.0 for j in range(k)]
           for i, row in enumerate(XtX)]
    for col in range(k):
        piv = max(range(col, k), key=lambda r: abs(aug[r][col]))
        aug[col], aug[piv] = aug[piv], aug[col]
        pv = aug[col][col]
        aug[col] = [v / pv for v in aug[col]]
        for r in range(k):
            if r != col:
                f = aug[r][col]
                aug[r] = [v - f * w for v, w in zip(aug[r], aug[col])]
    inv = [row[k:] for row in aug]
    beta = [sum(inv[a][b] * Xty[b] for b in range(k)) for a in range(k)]
    resid = [y[i] - sum(X[i][a] * beta[a] for a in range(k)) for i in range(n)]
    df = n - k
    s2 = sum(r * r for r in resid) / df
    se = [math.sqrt(s2 * inv[a][a]) for a in range(k)]
    tstat = [beta[a] / se[a] for a in range(k)]
    pval = [t_pvalue(abs(t), df) for t in tstat]
    return beta, tstat, pval, resid


def merge_latest(offline=None):
    data = {k: fetch(k, offline) for k in GRAPHERS}
    codes = sorted(set(data["regime"]) & set(data["poly"])
                   & set(data["ladder"]) & set(data["gdp"]))
    merged = []
    for c in codes:
        ry, regime = data["regime"][c]
        py, poly = data["poly"][c]
        ly, ladder = data["ladder"][c]
        gy, gdp = data["gdp"][c]
        if min(ry, py, ly, gy) < 2022:  # stale tail — keep the merge honest
            continue
        merged.append((c, regime, poly, ladder, math.log(gdp),
                       ry, py, ly, gy))
    return merged


def read_snapshot(path):
    """Read the merged, upstream-licensed CSV distributed with the book."""
    with open(path, newline="", encoding="utf-8") as stream:
        return [
            (row["iso3"], float(row["row_regime"]), float(row["polyarchy"]),
             float(row["cantril_ladder"]), float(row["log_gdp_pc"]),
             int(row["regime_year"]), int(row["poly_year"]),
             int(row["ladder_year"]), int(row["gdp_year"]))
            for row in csv.DictReader(stream)
        ]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group()
    source.add_argument("--offline", metavar="DIR",
                        help="directory holding the four raw OWID CSVs")
    source.add_argument("--from-snapshot", metavar="FILE",
                        help="merged CSV to analyse without fetching")
    parser.add_argument("--snapshot", metavar="DIR",
                        help="write the merged observations to this directory")
    args = parser.parse_args()
    merged = (read_snapshot(args.from_snapshot) if args.from_snapshot
              else merge_latest(args.offline))
    n = len(merged)
    regime = [m[1] for m in merged]
    poly = [m[2] for m in merged]
    ladder = [m[3] for m in merged]
    lgdp = [m[4] for m in merged]

    print(f"N = {n} countries (latest observation per series, merged on ISO3)")
    for index, name in enumerate(("regime", "polyarchy", "ladder", "GDP"), 5):
        years = [row[index] for row in merged]
        print(f"  {name} observation years: {min(years)}..{max(years)}")
    mismatched = sum(len(set(row[5:])) > 1 for row in merged)
    print(f"  countries with differing observation years: {mismatched}")
    print("Descriptive cross-section; no causal or transition effect is estimated.")
    r = pearson(poly, ladder)
    rho = spearman(poly, ladder)
    print(f"raw:   r(polyarchy, ladder)   = {r:+.4f}   rho = {rho:+.4f}   "
          f"R^2 = {r*r:.3f}")
    r_pg = pearson(poly, lgdp)
    r_lg = pearson(ladder, lgdp)
    print(f"       r(polyarchy, log gdp)  = {r_pg:+.4f}   "
          f"r(ladder, log gdp) = {r_lg:+.4f}")
    pr = partial_r(r, r_pg, r_lg)
    pg = partial_r(r_lg, r_pg, r)
    print(f"income control: partial r(poly, ladder | log gdp) = {pr:+.4f}")
    print(f"                partial r(log gdp, ladder | poly) = {pg:+.4f}")

    print("\nRegimes of the World (latest year):")
    steps = []
    prev = None
    for cat in range(4):
        vals = [l for g, l in zip(regime, ladder) if g == cat]
        m = mean(vals)
        print(f"  {cat} {ROW_NAMES[cat]:<20} n={len(vals):>3}  "
              f"mean={m:.2f}  sd={sd(vals):.2f}")
        if prev is not None:
            steps.append(m - prev)
        prev = m
    print("  differences between category means: " +
          "  ".join(f"{s:+.2f}" for s in steps))

    print("\nExploratory dispersion of country means (not a lower-tail test):")
    _, _, _, resid = ols(ladder, [poly])
    absr = [abs(x) for x in resid]
    b1, t1, p1, _ = ols(absr, [poly])
    print(f"  |resid| ~ polyarchy:            b = {b1[1]:+.4f}  "
          f"t = {t1[1]:+.2f}  p = {p1[1]:.4f}")
    b2, t2, p2, _ = ols(absr, [poly, lgdp])
    print(f"  |resid| ~ polyarchy + log gdp:  b_poly = {b2[1]:+.4f}  "
          f"t = {t2[1]:+.2f}  p = {p2[1]:.4f}")
    print(f"                                  b_lgdp = {b2[2]:+.4f}  "
          f"t = {t2[2]:+.2f}  p = {p2[2]:.4f}")

    if args.snapshot:
        path = f"{args.snapshot}/vdem-happiness-{date.today().isoformat()}.csv"
        with open(path, "w", newline="", encoding="utf-8") as f:
            w = csv.writer(f)
            w.writerow(["iso3", "row_regime", "polyarchy", "cantril_ladder",
                        "log_gdp_pc", "regime_year", "poly_year",
                        "ladder_year", "gdp_year"])
            for m in merged:
                w.writerow(m)
        print(f"\nsnapshot written: {path}")


if __name__ == "__main__":
    main()
