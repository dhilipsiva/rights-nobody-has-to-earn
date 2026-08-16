# Counterfactual fixtures

Copies of `../constitution.nibli`, each differing from it in **exactly one deliberate way**.
They exist because of a limitation the tracker records: derivation is monotone and probe
facts load *on top* of the knowledge base, so **no probe can test a restriction**. Every
"if we removed X" claim in the book and the tracker is an argument until it is run against
a file where the change is actually made.

Three classes, and `verify.sh` checks each fixture's diff shape as its identity:

- **A line deleted** (1 removed, 0 added) — `no-person-line`, `no-public-court`,
  `no-choose-boss`, `no-first-contact-standing`. What the world loses without the line.
- **A line changed** (1 removed, 1 added) — `no-dead-conjuncts`: Article 4's multi-sig with
  its `~broken`/`~match(·, CarriedVoid)` signer checks stripped. It has **no paired pin file on purpose**:
  `verify.sh` runs chapters 4 and 5's own pin files against it, and their passing unchanged
  is the standing proof those conjuncts decide nothing today. The proof strengthens by
  itself as those suites grow.
- **A line added** (0 removed, 1 added) — `unguarded-pen`: the constitution *plus* a
  credential route that forgets the guards. A postulated future, not a deletion — its pins
  show Article 4's kept conjuncts are the only thing standing between that one line and a
  carried-void signature counting. See the v0.9 note in Article 4's header.
  `undelivered-marker` is the same class: the constitution *plus* the floor-delivery marker
  the 2026-08-02 ruling refused. Its pins show the marker firing on the voided, the
  confined and the never-accused alike — the measured reason it stays unbuilt while the
  record holds no arrival facts. Regenerate like unguarded-pen: copy, then append
  `all $x: owe(State, Eats, $x) & ~eats($x) -> err($x, Undelivered).` with no leading
  blank line.

Regenerate any of them with:

```
K=new-book-plans/constitution.nibli
CF=new-book-plans/counterfactual
grep -vFx 'all $anyone: prisoner($anyone) -> person($anyone).' $K > $CF/no-person-line.nibli
grep -vFx 'public(Court).'                                     $K > $CF/no-public-court.nibli
grep -vFx 'choose(Electorate, Boss).'                          $K > $CF/no-choose-boss.nibli
grep -vFx 'all $subject: at($subject, FirstContact) & ~public($subject) -> person($subject).' \
  $K > $CF/no-first-contact-standing.nibli
```

**The `$CF/` prefixes are load-bearing and were missing until 2026-07-30.** `$K` is
written relative to the repo root, so the command has to be run from there — and without
the prefix the three files landed in the repo root while the real fixtures sat untouched
beside this README. Anyone following the procedure would have believed they had
regenerated and would have left three stale fixtures in place, which is the exact failure
the `-vFx` note below describes, reached by a different route. That makes twice this
file's own command has been wrong; check it against a real run before trusting it.

**Use `-vFx`, and do not "fix" it back to a regex.** The version of this command that
stood here until v0.5 was `grep -v '^all \$anyone: …'`, and inside single quotes `\$` is a
literal backslash-dollar, so it matched nothing and wrote out a byte-identical copy of the
constitution. Anyone following the documented procedure destroyed the fixture, and the
three pins still passed, because a fixture that is a copy of the real file answers every
question the real file answers. `-F` (fixed string) and `-x` (whole line) cannot be
misread that way.

Regenerate the changed-line and added-line fixtures with:

```
python3 - <<'EOF'
import pathlib
s = pathlib.Path('new-book-plans/constitution.nibli').read_text(encoding='utf-8')
old = " & ~broken($a) & ~broken($b) & ~match($a, CarriedVoid) & ~match($b, CarriedVoid) -> false($audited)."
assert s.count(old) == 1, f"expected exactly one occurrence, found {s.count(old)}"
pathlib.Path('new-book-plans/counterfactual/no-dead-conjuncts.nibli').write_text(
    s.replace(old, " -> false($audited)."), encoding='utf-8')
EOF
cp new-book-plans/constitution.nibli new-book-plans/counterfactual/unguarded-pen.nibli
printf 'all $a: choose(Electorate, $a) -> permits(Review, $a).\n' \
  >> new-book-plans/counterfactual/unguarded-pen.nibli
```

The assert is not decoration: this file's commands have been wrong twice, both times by
matching nothing and silently writing a byte-identical copy, and an assert on the
occurrence count is what makes that loud. The `printf` must not lead with a blank line —
that reads as a second added line and fails the shape check.

`diff` each against the constitution and check the shape its class requires. **Regenerate
after every constitution edit** — a stale fixture proves something about a file that no
longer exists, which is the failure mode these were built to answer. `verify.sh` now
enforces the one-line property before it runs the pins; it did not until v0.5, because the
guard assigned its result to a variable it never read.

| Fixture | Line removed | What it proves |
|---|---|---|
| `no-person-line.nibli` | `prisoner -> person` | Chapter 7's headline result. With the line, a heresy law is refused; without it the same law **loads**, and the whole population becomes imprisonable for belief. The clause that keeps prisoners human and the clause that keeps everyone's rights unconditional are the same clause. |
| `no-public-court.nibli` | `public(Court).` | The deletion axis of the fact-write trust base. One deleted line and `authority(Court)` goes FALSE, taking Sly's shield with it — `prisoner(Sly)` flips FALSE→TRUE. |
| `no-choose-boss.nibli` | `choose(Electorate, Boss).` | The same harm by the other route into standing. `authority(Boss)` goes FALSE and **Rebel — the file's own honest whistleblower — is jailed**, which is the whole of chapter 2's argument, undone by deleting one fact. |
| `no-first-contact-standing.nibli` | first-contact standing rule | An unregistered first-contact handle no longer acquires standing, the floor, or the State's debt; the other standing roots remain controls. |

Each has a paired `*.pins.nibli` asserting the flipped verdicts. Those pin files are
**expected to pass against their own fixture**, not against the constitution: they encode
what the world looks like once the line is gone. Run one with

```
nibli-pin --kb new-book-plans/counterfactual/no-person-line.nibli \
          new-book-plans/counterfactual/no-person-line.pins.nibli
```

`verify.sh` runs all three.

## Liberty and ecological fixtures

The `no-environmental-right` fixture deletes the present-person environmental-conditions rule while retaining environmental information and the material floor.
The `no-class9-climate-axis` fixture deletes only the climate-axis ceiling rule while retaining the clean-air axis and the material-floor boundary.
Both are one-line deletion fixtures and are regenerated from `constitution.nibli` after every constitutional edit.

## Substantive-equality fixtures

The `no-direct-equality`, `no-equality-data-wall`, and
`no-positive-measure-end` fixtures each delete exactly one independently
registered equality effect. Their controls keep an adjacent equality effect and
the common status baseline live, so a green result cannot come from losing the
whole family. Regenerate them from `constitution.nibli` after every source edit
by deleting, respectively:

- `all $x: person($x) -> prevents($x, DirectDiscrimination).`
- `all $x: person($x) -> prevents($x, EqualityDiagnosticRecordReuse).`
- `all $x: person($x) -> prevents($x, ExpiredPositiveMeasureContinuation).`

## Family and life-course fixtures

The family and life-course suite adds four one-line deletion fixtures:

- `no-automatic-adulthood` removes the automatic general-adulthood protection while retaining the ballot control;
- `no-family-confinement-wall` removes only the explicit family-status placement wall;
- `no-missing-kinship-independence` removes only the affirmative-independence boundary; and
- `no-pregnancy-authority` removes only pregnancy continuation and termination authority while retaining the fetal-override refusal.

They are regenerated from `constitution.nibli` after every constitutional edit exactly like the equality deletion fixtures.
