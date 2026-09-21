<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# The Rights Nobody Has to Earn

**Book proposal · dhilipsiva · political nonfiction / constitutional design**

**Approximately 58,000 words · complete, internally reviewed manuscript**

This proposal seeks an editorial and print partnership for an openly licensed
book. It is prepared for a submission decision, without being addressed or sent
to a particular publisher. The [public manuscript](../book-1/README.md) and
[repository](https://github.com/dhilipsiva/rights-nobody-has-to-earn) provide the
text, its formal source and its evidence trail.

## Synopsis

A child's recorded birth is the book's starting point. The record names no
parent or provider and contains no request for help. What is the child owed,
who must provide it, and what follows when it does not arrive?
*The Rights Nobody Has to Earn* develops a constitutional answer: standing and a
floor of essentials belong to every person without a qualifying condition.

The book follows that commitment into food, shelter, care and learning, then
into the ordinary life it is meant to make possible. It describes freedom of
belief, speech, association and inquiry; earning above the floor; contribution
without purchased standing; family roles; and the claims of people who move.
Short cases distinguish an entitlement from delivery, a provider's receipt
from evidence, and a recorded duty from a remedy that reaches its recipient.

The later chapters turn to public power: elections, institutional authority,
records, emergencies, accessible justice and changing the rules. They examine
what happens when that power confines a person, mistakes a claim or exceeds
its limits. The governing question is how a society can owe essentials
unconditionally while making coercion specific, bounded and answerable.

The closing argument addresses valuation, rotation, coercion, capture and the
state. Historical evidence and objections test the choices behind the rules.
An optional method explains the executable model and the limits of its checks.
Readers can follow the design without learning its formal language, while
those who want to challenge a consequence can inspect and run the corresponding
case. This is Book 1's destination: constitutional duties and remedies. The
planned Book 2, *What It Would Take*, concerns operation and transition; it
remains inactive until Book 1's release decision.

## Readers and contribution

The primary reader is interested in rights, public institutions and alternatives
to making subsistence conditional on employment or approved conduct. The book
also offers constitutional designers, policy researchers and technically
curious readers a common object to examine. No programming knowledge is
required for the main reading sequence. It is a proposal for discussion and
criticism, rather than a guide to an existing jurisdiction or an account of
a society already operating these arrangements.

Its particular contribution is the combination of a sustained, ordinary-language
constitutional design with public executable cases. A reader can ask what a
rule entails for a named person, what evidence the conclusion depends on, and
which apparently similar case it refuses. The prose supplies reasons and
consequences; the formal model supplies conditional results that can be
challenged against their stated inputs. Neither substitutes for evidence that
an institution can deliver its promises. The book makes no claim to have
invented unconditional rights, institutional alternatives or formal reasoning
about law.

## Relationship to existing writing

These are points of intellectual comparison, not sales forecasts or endorsements.

- John Rawls's *A Theory of Justice* develops justice as fairness through a
  social-contract account of basic rights and liberties. This book shares the
  concern with the terms of a society's institutions, but presents one concrete
  proposed constitution through cases rather than deriving Rawls's principles.
  [Harvard University Press / Belknap's description of the revised edition](https://www.jstor.org/stable/j.ctvkjb25m).
- Erik Olin Wright's *Envisioning Real Utopias* examines concrete emancipatory
  alternatives to capitalism. Readers interested in institutional alternatives
  may find a related question here; this book's narrower object is a single
  constitutional destination, with operation and transition assigned to a
  separate volume. [Verso's description](https://www.versobooks.com/products/2143-envisioning-real-utopias).
- Elinor Ostrom's *Governing the Commons* uses empirical institutional analysis
  to examine successful and unsuccessful arrangements for common-pool
  resources. Its attention to institutional conditions is a useful comparison.
  This manuscript is a normative design with formal examples; it does not
  offer equivalent field evidence for its own arrangements.
  [Cambridge University Press's description](https://www.cambridge.org/core/books/governing-the-commons/7AB7AE11BADA84409C34815CC288CD79).

## Contents

The [opening note's annotated contents](../book-1/00-opening-note.md#annotated-contents)
give the longer chapter descriptions. The sequence below matches
[the current contents manifest](../book-1/contents.json).

Epigraph; A Note Before the Design — both unnumbered.

**Part I — Who counts, and what they are owed**

1. The Child With Nobody
2. Who Counts
3. What Counts as Evidence
4. What You Are Owed
5. Whether It Arrived
6. When There Is Genuinely Not Enough
7. Who Owes, and What Follows

**Part II — The life the design leaves alone**

8. What Nobody Has to Ask Permission For
9. Earning Above the Floor
10. Contribution
11. What Money Cannot Buy
12. The Same Route for Everyone
13. A Place in Which Life Remains Possible
14. Holding a Role in Somebody's Life
15. Arriving and Belonging

**Part III — The public power that serves it**

16. Public Answerability, and Why It Is Never Revoked
17. How Public Power Is Built
18. The Vote Conviction Does Not Take
19. What May Be Kept About You
20. A Crisis Does Not Suspend the Republic
21. A Way to Be Heard
22. Changing the Rules
23. Who Holds the Pen

**Part IV — What the design does to a person, and how it catches itself**

24. The Shield
25. Voiding
26. The Limits of a Finding
27. A Prisoner Is a Person
28. Where People Are Put
29. The One Thing Taken
30. When the System Notices It Broke

**Part V — Outside the graph**

31. The Five Joints

The Method — optional and unnumbered.

## Representative sample

The selection contains five complete chapters, approximately 14,000 words.
Their original numbering and citations are retained. It includes both short
derived chapters and the longer closing argument so an editor can assess the
book's two principal modes. The selection moves from standing to delivery,
ordinary freedom and access to justice, then gives the opposing arguments
and evidence in full. Chapter 31 occupies about 71% of the sample, compared
with about 17% of the manuscript. Keeping it complete lets an editor assess
the reasoning and its qualifications together. This selection can be adapted
to a particular publisher's requirements before any submission.

| Chapter | What the selection shows |
|---|---|
| [1. The Child With Nobody](../book-1/01-the-child-with-nobody.md) | Unconditional standing tested from a minimal birth record. |
| [5. Whether It Arrived](../book-1/05-whether-it-arrived.md) | The difference between an entitlement, a receipt and established delivery. |
| [8. What Nobody Has to Ask Permission For](../book-1/08-what-nobody-has-to-ask-permission-for.md) | Ordinary freedom and the limits on public interference. |
| [21. A Way to Be Heard](../book-1/21-a-way-to-be-heard.md) | Accessible claims, institutional duties and the conditions for relief. |
| [31. The Five Joints](../book-1/31-the-five-joints.md) | The extended argument, historical sources, objections and costs. |

To assemble the sample from this checkout, follow the
[browser setup instructions](../book-1/README.md#read-or-assemble-the-book), then run:

```bash
uv run tools/build_book.py --sample
```

This produces `book-1-sample.html`, `book-1-sample.epub` and
`book-1-sample.pdf` in `output/book-1/`; the current PDF is 35 pages.
Use `--no-pdf` for HTML and EPUB alone. The sample is labelled as selected
chapters. Links outside the selection open the public manuscript or formal
source on `main`, which can change; they do not bind a submitted copy to an
immutable edition. The same builder without `--sample` assembles the full
manuscript. Generated copies are local artifacts, rebuilt from the text being
considered for submission.

## Completion and limits

As of 2026-09-21, all 31 numbered chapters, the epigraph, opening note and
method are drafted: 57,909 words across 34 ordered inputs. The 30 derived
chapters contain 40,481 words, or 69.90%; the sample contains 13,633.
These counts use whitespace-separated rendered manuscript text, including
notes, before the generated cover and contents. The full manuscript and
sample are available locally as HTML, EPUB and PDF.

The latest completed substantive run passed 88,815 pins across
16,137 cases in 1,188.54 seconds, with complete contradiction checks and no
findings. No active known-defect expectations remain. Explicitly weakened
counterfactuals still demonstrate the harms their altered rules permit; those
results do not describe the enacted model. The five-minute verification target
is not met.

The [September 21 review](../reviews/2026-09-21-final-manuscript-review.md)
rates the manuscript 9/10 after reading all ordered inputs and correcting a
shield-scope overstatement. It includes chapter assessments, remaining
editorial limits and the scope of its checks. The [revision backlog](../TODO.md)
is complete; the submission decision remains separate.
The rebuilt full PDF has 169 pages; the five-chapter sample has 35.
Both EPUBs pass EPUBCheck with zero errors or warnings. Inspection found
no missing checked text or invalid internal PDF destinations; browser checks
cover the full HTML and all packaged EPUB documents at 360px and 1280px.
These are bounded formal and rendering checks within the same AI-assisted
project, not independent editorial endorsement.

The [method](../book-1/method.md) discloses AI assistance and explains what
the checks establish. No independent expert endorsement, external reader
testing or successful operation of the proposed society is claimed. Real-world
inputs, honest institutions, resources and successful remedies cannot be
certified by a contradiction-free model.

## Open contribution, rights and the proposed partnership

Readers can propose corrections, evidence, objections and co-authored additions
through the public repository. The [contribution guide](../CONTRIBUTING.md)
describes review, attribution and integration into the maintained edition.
Submitting a change does not automatically make it part of that edition.
Independent adaptations remain possible under the applicable licences.

The new book prose is **CC BY 4.0**: commercial republication and adaptation
are allowed with the licence's required attribution and notices. Existing
grants cannot be withdrawn to create exclusive control over already licensed
text. The constitution and registry claims retain **CC0** terms; code is
**MIT OR Apache-2.0**; bundled fonts and upstream data retain their own terms.
Legacy public-domain dedications remain in place. See the
[licensing policy](../LICENSING.md) and
[CC BY 4.0 terms](https://creativecommons.org/licenses/by/4.0/).

The desired partnership would provide editorial challenge, copyediting,
typesetting, print production and distribution for a clearly identified
edition while preserving the open manuscript and public correction process.
The publisher and author would need to agree how proposed changes reach that
edition, how contributors are credited, and what services and rights the
agreement covers. No publisher's acceptance of open editing is assumed from
its support for open access. Before adapting this proposal to a named press,
check its current official requirements, including sample length, author
information, rights and AI-disclosure policies. Contact, submission and any
contract are separate decisions; none has taken place through this package.

This proposal is licensed under [CC BY 4.0](../book-1/LICENSE-CC-BY).
