#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Draw Book 1's diagrams as accessible SVG.

Each diagram is a few labelled boxes and arrows, drawn in black on white so it
prints and reads without colour. Its title and description are embedded, and
the chapter text beside it carries a full prose equivalent. Run this after
changing a diagram; it rewrites `book-1/diagrams/`.
"""

from __future__ import annotations

from html import escape
from pathlib import Path
import textwrap

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "book-1" / "diagrams"
WIDTH = 720
FONT = "DejaVu Sans, Arial, sans-serif"
SIZE = 13
LINE = 17


def box(x: int, y: int, w: int, text: str, *, bold: str = "", dashed: bool = False,
        chars: int | None = None) -> tuple[str, int]:
    """A box whose height follows its wrapped text; returns the SVG and height."""
    chars = chars or max(12, int(w / 7.1))
    lines = textwrap.wrap(text, chars)
    # Bold glyphs run wider, so a heading wraps sooner than its text.
    head = textwrap.wrap(bold, max(10, int(w / 8.6))) if bold else []
    h = 14 + LINE * (len(head) + len(lines))
    dash = ' stroke-dasharray="5 4"' if dashed else ""
    parts = [f'<rect x="{x}" y="{y}" width="{w}" height="{h}" rx="6" fill="white" stroke="black" stroke-width="1.4"{dash}/>']
    ty = y + 7 + SIZE
    for line in head:
        parts.append(f'<text x="{x + w / 2}" y="{ty}" text-anchor="middle" font-weight="bold">{escape(line)}</text>')
        ty += LINE
    for line in lines:
        parts.append(f'<text x="{x + w / 2}" y="{ty}" text-anchor="middle">{escape(line)}</text>')
        ty += LINE
    return "".join(parts), h


def arrow(x1: float, y1: float, x2: float, y2: float, label: str = "") -> str:
    text = ""
    if label:
        text = (f'<text x="{(x1 + x2) / 2 + 6}" y="{(y1 + y2) / 2 + 4}" font-style="italic" '
                f'font-size="{SIZE - 1}">{escape(label)}</text>')
    return (f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="black" stroke-width="1.4" '
            f'marker-end="url(#head)"/>{text}')


def svg(name: str, title: str, description: str, body: list[str], height: int) -> None:
    ident = name.replace("-", "")
    doc = f'''<!-- SPDX-License-Identifier: CC-BY-4.0 -->
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {WIDTH} {height}" width="{WIDTH}" height="{height}" role="img" aria-labelledby="{ident}-title {ident}-desc" font-family="{FONT}" font-size="{SIZE}">
<title id="{ident}-title">{escape(title)}</title>
<desc id="{ident}-desc">{escape(description)}</desc>
<defs><marker id="head" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" fill="black"/></marker></defs>
<rect width="{WIDTH}" height="{height}" fill="white"/>
{"".join(body)}
</svg>
'''
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / f"{name}.svg").write_text(doc, encoding="utf-8")


def column(items: list[tuple[str, str]], x: int, w: int, y: int, gap: int = 26,
           labels: list[str] | None = None) -> tuple[list[str], int]:
    """Boxes stacked top to bottom with arrows between them."""
    body, heights = [], []
    for index, (bold, text) in enumerate(items):
        part, h = box(x, y, w, text, bold=bold)
        body.append(part)
        heights.append((y, h))
        y += h + gap
    for index in range(len(heights) - 1):
        top, h = heights[index]
        label = labels[index] if labels and index < len(labels) else ""
        body.append(arrow(x + w / 2, top + h, x + w / 2, heights[index + 1][0] - 2, label))
    return body, y - gap


def duty_chain() -> None:
    body = []
    part, h1 = box(160, 16, 400, "standing from a birth, first contact, presence, effective control or a report that nobody is acting for them", bold="A person")
    body.append(part)
    y2 = 16 + h1 + 26
    part, h2 = box(160, y2, 400, "each of the nine floor items is an entitlement of the person and a debt of the State", bold="Owed the floor")
    body.append(part)
    body.append(arrow(360, 16 + h1, 360, y2 - 2))
    y3 = y2 + h2 + 34
    tiers = [
        ("The common tier", "finances, equalises and sets minimum standards, and backstops the floor for everyone"),
        ("The region", "provides, where a witness authorised for the person records them present"),
        ("The locality", "reaches and delivers, where a witness records them present"),
    ]
    heights = []
    for index, (bold, text) in enumerate(tiers):
        x = 16 + index * 236
        part, h = box(x, y3, 216, text, bold=bold)
        body.append(part)
        heights.append(h)
        body.append(arrow(360, y2 + h2, x + 108, y3 - 2))
    y4 = y3 + max(heights) + 34
    part, h4 = box(110, y4, 500, "certified by a source and an independent reviewer, neither of them the failing body", bold="A body fails")
    body.append(part)
    for index in range(3):
        x = 16 + index * 236 + 108
        body.append(arrow(x, y3 + heights[index], 360, y4 - 2))
    y5 = y4 + h4 + 26
    part, h5 = box(110, y5, 500, "the publicly answerable tier the certification names; the failing body still owes its own duty", bold="Continuity passes up a tier")
    body.append(part)
    body.append(arrow(360, y4 + h4, 360, y5 - 2))
    y6 = y5 + h5 + 22
    part, h6 = box(60, y6, 600, "a certified pattern of failure goes to the Constitutional Court, whose declaration obliges the Assembly to plan and, if it does not answer, the Court to order interim measures that secure the minimum", bold="Failure at scale", dashed=True)
    body.append(part)
    svg("duty-chain", "The duty chain",
        "A person is owed the floor; the common tier, the region and the locality each owe part of it; a certified failure passes continuity up a tier while the failing body's duty stands; a pattern of failure goes to the Constitutional Court.",
        body, y6 + h6 + 16)


def delivery_evidence() -> None:
    body = []
    inputs = [
        ("A receipt", "the recipient received this item from this source"),
        ("An authorisation", "the witness is authorised for this recipient"),
        ("An observation", "the witness observed this item for this recipient, at the item's scope"),
    ]
    y, heights = 16, []
    for bold, text in inputs:
        part, h = box(16, y, 300, text, bold=bold)
        body.append(part)
        heights.append((y, h))
        y += h + 16
    guard, hg = box(16, y + 4, 300, "the witness is not the source, nor the State, the common tier, or the person's recorded region or locality", bold="Held apart", dashed=True)
    body.append(guard)
    total = y + 4 + hg
    mid = (16 + total) / 2
    part, hc = box(420, mid - 40, 284, "for that item and that person, and for nothing else", bold="Delivery is concluded")
    body.append(part)
    for top, h in heights + [(y + 4, hg)]:
        body.append(arrow(316, top + h / 2, 418, mid - 40 + hc / 2))
    y = total + 24
    left, hl = box(16, y, 330, "a provider's own receipt and observation conclude nothing", bold="Not enough")
    right, hr = box(374, y, 330, "bodily safety, expression and belief: answered by protective duties, never by a receipt", bold="No delivery route")
    body += [left, right]
    svg("delivery-evidence", "Delivery evidence",
        "A receipt, an authorisation for the recipient and a matching observation, from a witness held apart from the source and from every body that owes the floor, conclude delivery of that item; a provider's own word concludes nothing, and bodily safety, expression and belief have no delivery route.",
        body, y + max(hl, hr) + 16)


def placement_ceiling() -> None:
    body = []
    part, h1 = box(130, 16, 460, "the merits, the custody authorisation and its review period are properly made for this case", bold="A lawful sentence")
    body.append(part)
    y2 = 16 + h1 + 30
    left, hl = box(16, y2, 330, "grave injury or aggravated cruelty, found in this case; victim counts, raw acts, family, wealth and poverty never count", bold="With an adjudicated finding of severity")
    right, hr = box(374, y2, 330, "the finding is absent", bold="Without it")
    body += [left, right, arrow(300, 16 + h1, 181, y2 - 2), arrow(420, 16 + h1, 539, y2 - 2)]
    y3 = y2 + max(hl, hr) + 30
    left, hl3 = box(16, y3, 330, "home confinement, ordinary supported residence, or a secure place", bold="The ceiling allows")
    right, hr3 = box(374, y3, 330, "home confinement or ordinary supported residence", bold="The ceiling allows")
    body += [left, right, arrow(181, y2 + hl, 181, y3 - 2), arrow(539, y2 + hr, 539, y3 - 2)]
    y4 = y3 + max(hl3, hr3) + 30
    part, h4 = box(70, y4, 580, "a named available place, individual necessity, lawful conditions and independent review; a secure place also needs a finding that less restrictive places are insufficient", bold="Each place still needs")
    body += [part, arrow(181, y3 + hl3, 250, y4 - 2), arrow(539, y3 + hr3, 470, y4 - 2)]
    svg("placement-ceiling", "The placement ceiling",
        "Severity, found in the case, raises the most restrictive place that may be considered to a secure place; without it the ceiling is home confinement or ordinary supported residence; every place still needs availability, necessity, lawful conditions and review, and a secure place a finding that less restrictive places are insufficient.",
        body, y4 + h4 + 16)


def democratic_corridor() -> None:
    body = [
        '<rect x="12" y="12" width="696" height="{H}" rx="10" fill="white" stroke="black" stroke-width="2"/>',
        f'<text x="360" y="36" text-anchor="middle" font-weight="bold">The protected core, beyond amendment</text>',
        f'<text x="360" y="54" text-anchor="middle">among them standing, the floor, equality, due process, core liberties, the commons,</text>',
        f'<text x="360" y="71" text-anchor="middle">the animal core, the absolute prohibitions and an Assembly and Court able to sit</text>',
        '<rect x="40" y="86" width="640" height="{H2}" rx="8" fill="white" stroke="black" stroke-width="1.4" stroke-dasharray="6 4"/>',
        f'<text x="360" y="108" text-anchor="middle" font-weight="bold">Constitutional law, amendable</text>',
        f'<text x="360" y="125" text-anchor="middle">two-thirds of the full Assembly and more yes than no in a referendum</text>',
    ]
    items = [("A public choice", "made within the limits"), ("A certified result", "authorised and certified"),
             ("In force", "published and selected"), ("Review", "and peaceful correction")]
    x, y = 56, 150
    heights = []
    for bold, text in items:
        part, h = box(x, y, 136, text, bold=bold)
        body.append(part)
        heights.append(h)
        x += 158
    for index in range(3):
        body.append(arrow(56 + index * 158 + 136, y + 24, 56 + (index + 1) * 158 - 2, y + 24))
    inner = y + max(heights) + 20
    body[4] = body[4].replace("{H2}", str(inner - 86))
    body[0] = body[0].replace("{H}", str(inner + 16 - 12))
    svg("democratic-corridor", "The democratic corridor",
        "Ordinary public choice runs inside amendable constitutional law, which runs inside a protected core no amendment may remove; a choice becomes a certified result, is published and selected into force, and stays open to review and peaceful correction.",
        body, inner + 30)


def flow_constraints() -> None:
    body = []
    part, h = box(230, 16, 260, "an entry about a person", bold="A fact")
    body.append(part)
    gates = [
        ("Closed inputs", "only admitted kinds of entry"),
        ("Unwritable conclusions", "custody, ballots and answerability are derived"),
        ("Purpose-bound reads", "a record serves its own purpose"),
        ("Endpoints", "duties and deliveries feed nothing"),
        ("No confinement from absence", "a missing entry confines nobody"),
        ("Scope binding", "a finding stays with its case"),
    ]
    top = 16 + h + 34
    rows = []
    for index, (bold, text) in enumerate(gates):
        col, row = index % 3, index // 3
        rows.append((col, row, bold, text))
    heights = [max(box(0, 0, 216, t, bold=b)[1] for c, r, b, t in rows if r == row) for row in (0, 1)]
    for col, row, bold, text in rows:
        y = top + sum(heights[:row]) + 14 * row
        part, _ = box(16 + col * 236, y, 216, text, bold=bold)
        body.append(part)
    body.append(arrow(360, 16 + h, 360, top - 4))
    bottom = top + sum(heights) + 14
    y = bottom + 30
    part, hc = box(230, y, 260, "for the person", bold="A consequence")
    body.append(part)
    body.append(arrow(360, bottom + 2, 360, y - 2))
    y += hc + 18
    note, hn = box(16, y, 688, "Each constraint is checked on the written form of the rules; a rule reaching the same consequence another way needs its own check.", dashed=True)
    body.append(note)
    svg("flow-constraints", "Six ways a fact is kept from a consequence",
        "A fact about a person reaches a consequence only through six constraints: closed inputs, conclusions nobody may write, purpose-bound reads, endpoints nothing reads, no confinement from absence, and scope binding; each is checked on the written form of the rules.",
        body, y + hn + 16)


def main() -> None:
    duty_chain()
    delivery_evidence()
    placement_ceiling()
    democratic_corridor()
    flow_constraints()
    print(f"Drew {len(list(OUT.glob('*.svg')))} diagrams in {OUT.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
