<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Pilot Private Custody and Public Record Instructions

> **Status: draft template; no custodian is appointed and no record, freeze,
> attestation, or evidence exists.** A checker can validate bindings but cannot
> establish that an external custodian exists or tells the truth.

## 1. Governing separation

The named reviewer/custodian maintains two deliberately unequal stores:

| Private custody only | Public repository allowed |
| --- | --- |
| participant, session, coder, facilitator, adjudicator, reviewer, and custodian names, pseudonyms, identifiers, and identity mappings | opaque study and attempt IDs that disclose no identity |
| contact, recruitment, screening, demographic, accessibility, compensation, and payment records | coded target and pilot-observation outcomes for admissible sessions only |
| raw audio, video, transcripts, notes, quotations, excerpts, and diagnostic free text | artifact and nonce-protected commitment digests |
| consent, withdrawal, safety, incident, and ethics-review records | coded deviations with no narrative or identity material |
| independent coder records, adjudication excerpts, and role-conflict checks | custody attestations containing no identity material |
| pre-registration payload, as-run instrument/rubric, tested snapshot, admissibility sources, control key, and decision-packet source material | the reviewed machine fields allowed by `reader-evidence.json` |

Public allowance is not a publication instruction. Publish only what the
canonical schema requires for the current legal transition. Do not add raw or
free-text material because it seems anonymised; contextual re-identification
remains possible.

## 2. Identifier and access discipline

Generate opaque study, attempt, session, record, deviation, receipt, and
attestation IDs outside the repository. Keep the identity map in a separately
encrypted store with access restricted to the minimum custody personnel. Do
not encode initials, dates of birth, location, recruitment channel, sequence,
or another identifying fact in a public ID.

Give coders only opaque response records. Give the adjudicator the two locked
coder records and the minimum excerpts needed to apply the rubric. The
facilitator does not receive codes. The author receives the frozen decision
packet and sensitivity brief without identity or raw response material.

Record every read, write, export, deletion, reveal, and custody transfer in the
private audit trail. Pre-register retention and deletion rather than deciding
them after results are known.

## 3. Freeze payload and attestation

Before recruitment, construct the exact pilot pre-registration payload from
the bound protocol, instrument, rubric, sample rule, disclosure set, ethics
terms, provisional control rule, tested-snapshot manifest, prerequisite
references, fixed protocol digest, predecessor attempt digest if any, and
prior history head.

The external freeze record carries:

- an opaque binding ID and custody reference;
- binding type and canonical UTC freeze time;
- the exact bound-payload SHA-256;
- the computed `attested_payload_sha256` required by the canonical contract;
- the attestation artifact SHA-256; and
- the custodian's private identity and signed source record outside Git.

A generic statement that “materials were frozen” is invalid. The public
attestation binds bytes and scope, not identity. The frozen `not-run` attempt
must be committed before recruitment or collection. A change to any bound byte
voids that attempt; preserve it and create a successor rather than overwriting
the old payload.

## 4. Run custody records

For each attempted session, custody privately links the identity, consent,
snapshot, raw record, facilitator record, compensation, independent coder
records, adjudication, admissibility, deviations, and final coded record.
Public session-record attestations bind only the opaque study ID, record
commitment, identity-free scope, external reference, and digests allowed by the
canonical schema.

Whenever run evidence exists, record exactly one `study-freshness` custody
attestation for that pilot study. For the pilot, it attests that the run records
were collected after the frozen pre-registration, belong to the named opaque
study, and were not imported from a prior attempt. It is not the holdout's
participant-freshness claim. A missing, duplicate, wrong-study, or false-valued
freshness record invalidates completion.

Every public coded deviation links to an identity-free custody attestation.
The private record states what happened and its adjudicated effect; the public
record carries only the frozen deviation code, impact class, opaque custody
link, and digests. Never remove an inconvenient deviation.

## 5. Completion receipt and decision packet

At a valid completion time in canonical UTC, generate the pilot receipt from
the exact pre-registration, snapshot, instrument, rubric, coded session
records, deviations, control transcript, coder record commitment, custody
records, and session classifications. The receipt binds all custody
attestation digests in canonical order.

Only after completion, freeze the decision packet under a second external
binding. It binds:

- the pilot pre-registration and tested snapshot;
- admitted coded evidence and the complete exclusion list;
- coder disagreements, including unresolved items;
- all deviations;
- the revised instrument and rubric material;
- the seeded-control transcript and result; and
- the packet's exact digest and freeze time.

Freeze the sensitivity brief separately or bind its exact artifact digest as
the canonical contract requires. The brief compares defensible post-pilot
choices without holdout evidence. It contains no participant identity or raw
response.

## 6. Privacy-minimal publication review

Before any commit, issue, message, or hand-off involving study records, two
people independently check that the candidate public material contains only
allowed kinds. Search for names, pseudonyms, emails, phone numbers, locations,
raw quotations, free text, consent language with signatures, accessibility
details, demographic combinations, payment identifiers, local private paths,
and identity-bearing metadata.

Hashing a prohibited record does not make the record publishable. Publish the
required digest or commitment, never the private preimage. Do not put secrets
or identity data in Git and then delete them; version history preserves them.
If prohibited data reaches Git, stop publication and follow the private
incident procedure before proceeding.

## 7. Truth and claim boundary

Custody attestations are evidence-bearing external statements, not formal
proof of their own truth. Repository checks validate shape, chronology,
digests, uniqueness, and binding. They do not authenticate people, consent,
freshness, compensation, accessibility, or the external store.

Ratification, route availability, and holdout results remain separate. No
custody record upgrades FS-CLM-37 or Gate C by itself.
