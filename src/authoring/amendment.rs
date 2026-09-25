// SPDX-License-Identifier: MIT OR Apache-2.0

//! Exact-change and effective-version interfaces, explicitly authored.
//! Bytes, authentication, publication and selection remain host responsibilities.

use super::{Base, Edit, Export, state_form};
use crate::{cli::Error, context::Context};
use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};

const BEGIN: &str = "# <AMENDMENT-ENACTMENT-RULES-BEGIN>";
const END: &str = "# <AMENDMENT-ENACTMENT-RULES-END>";
const PATH: &str = "book-1/source/constitution.nibli";
const INDEPENDENT: &str = "~($binder = $review)";

const FIELDS: &[(&str, &str)] = &[
    ("$base", "AmendmentBaseScope"),
    ("$base", "AmendmentObservedEffectiveBaseScope"),
    ("$candidate", "AmendmentCandidateScope"),
    ("$transition", "AmendmentTransitionScope"),
    ("$proposal", "AmendmentProposalScope"),
    ("$consent_record", "AmendmentConsentRecordScope"),
    ("$consent_result", "AmendmentConsentResultScope"),
    ("$certificate", "AmendmentConsentCertificateScope"),
    ("$jurisdiction", "AmendmentJurisdictionScope"),
    ("$legal_scope", "AmendmentLegalScope"),
    ("$effects", "AmendmentEffectReviewScope"),
    ("$operator", "AmendmentOperatorScope"),
    ("$reader", "AmendmentChallengeReaderScope"),
    ("$alternate", "AmendmentAlternateReaderScope"),
    ("$challenge", "AmendmentChallengeScope"),
    ("$correction", "AmendmentCorrectionScope"),
    ("$remedy", "AmendmentRemedyScope"),
    ("$end", "AmendmentReviewEndScope"),
    (
        "ExactBaseAndCandidateBytesBound",
        "AmendmentByteIdentityScope",
    ),
    (
        "BoundedEffectsMatchReviewedCandidate",
        "AmendmentEffectDispositionScope",
    ),
    (
        "PositiveCorridorCompatibility",
        "AmendmentCompatibilityScope",
    ),
    (
        "PreservesDirectCrediblySentientAnimalProtectedSubjectStatus",
        "AmendmentAnimalSubjectCoreScope",
    ),
    (
        "PreservesNonWaivableSevereAvoidableAnimalSufferingProhibition",
        "AmendmentAnimalSufferingCoreScope",
    ),
    (
        "PreservesNonWaivableDispensableAnimalKillingProhibition",
        "AmendmentAnimalKillingCoreScope",
    ),
    ("$vocabulary", "AmendmentVocabularyDispositionScope"),
    ("CurrentReconciledReview", "AmendmentCurrentReviewScope"),
    (
        "FreshUnconsumedTransition",
        "AmendmentReplayDispositionScope",
    ),
    (
        "IndependentChallengeAndFinalCourtReview",
        "AmendmentFinalReviewScope",
    ),
    (
        "NoPersonalConsequenceFromSourceStatus",
        "AmendmentConsequenceLimitScope",
    ),
];

fn variables(text: &str) -> BTreeSet<String> {
    Regex::new(r"\$[a-z][a-z0-9_]*")
        .unwrap()
        .find_iter(text)
        .map(|m| m.as_str().to_owned())
        .collect()
}

fn rule(body: &[String], head: &str) -> String {
    let quantifiers: String = variables(&format!("{} {head}", body.join(" ")))
        .iter()
        .map(|v| format!("all {v}: "))
        .collect();
    format!("{quantifiers}{} -> {head}.", body.join(" & "))
}

fn atoms(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

fn observed(body: &mut Vec<String>, actors: &[&str], value: &str, scope: &str) {
    for actor in actors {
        body.push(format!("observe({actor}, $record, {value}, {scope})"));
    }
}

fn candidate_body() -> Vec<String> {
    let mut body = atoms(&[
        "authorized($binder, AmendmentSourceBindingAuthority, $record)",
        "authorized($review, IndependentAmendmentEffectReviewAuthority, $record)",
        "authorized($reader, AmendmentChallengeReaderAuthority, $record)",
        "authorized($alternate, AmendmentAlternateReaderAuthority, $record)",
        INDEPENDENT,
        "~related($record, AmendmentRecordAmbiguity)",
        "~contradict($record, AmendmentSourceAuthorization)",
        "~($base = $candidate)",
        "member($vocabulary, AmendmentVocabularyDisposition)",
    ]);
    for (i, left) in ["$binder", "$review", "$operator", "$reader", "$alternate"]
        .iter()
        .enumerate()
    {
        for right in ["$binder", "$review", "$operator", "$reader", "$alternate"]
            .iter()
            .skip(i + 1)
        {
            let distinct = format!("~({left} = {right})");
            if !body.contains(&distinct) {
                body.push(distinct);
            }
        }
    }
    for (value, scope) in FIELDS {
        observed(&mut body, &["$binder", "$review"], value, scope);
    }
    body.extend(atoms(&[
        "complete($consent_result, FSPOW_037, $consent_record)",
        "authority(FSBOD_02, FSPOW_037, $consent_record)",
        "authorized($consent_source, StateFormSourceAuthority, $consent_record)",
        "observe($consent_source, $consent_record, $base, SourceVersionScope)",
        "observe($consent_source, $consent_record, $consent_result, ResultScope)",
        "observe($consent_source, $consent_record, $jurisdiction, JurisdictionScope)",
        "observe($consent_source, $consent_record, $legal_scope, AuthorityScope)",
        "observe($consent_source, $consent_result, $proposal, AmendmentProposalScope)",
        "observe($consent_source, $consent_result, $certificate, ResultCertificateScope)",
        "observe($consent_source, $consent_result, $base, AmendmentBaseScope)",
        "observe($consent_source, $consent_result, $candidate, AmendmentCandidateScope)",
        "~related($review, $consent_record, AmendmentConsentOperator)",
        "~($review = FSBOD_01)",
        "~($review = FSBOD_02)",
        "~($review = FSBOD_03)",
    ]));
    body
}

/// Item 71: the effect reviewer's reading of the core does not block on its
/// own. Where the reviewer records no positive compatibility, an independent
/// final reviewer's decision that the candidate is compatible stands in for it;
/// every other field of the reviewed record is still required from both
/// attesters. The final reviewer is none of the record's own actors and none of
/// the political bodies.
fn final_review_body() -> Vec<String> {
    let positive = "observe($review, $record, PositiveCorridorCompatibility, AmendmentCompatibilityScope)";
    let mut body = candidate_body();
    let before = body.len();
    body.retain(|atom| atom != positive);
    assert_eq!(body.len() + 1, before, "the reviewer's compatibility atom");
    body.extend(atoms(&[
        "authorized($final, AmendmentFinalCompatibilityAuthority, $record)",
        "observe($final, $record, FinalReviewFoundCandidateCompatible, AmendmentFinalCompatibilityScope)",
        "~($final = $binder)",
        "~($final = $review)",
        "~($final = $operator)",
        "~($final = FSBOD_01)",
        "~($final = FSBOD_02)",
        "~($final = FSBOD_03)",
        "~related($final, $consent_record, AmendmentConsentOperator)",
    ]));
    body
}

/// The closed list of corridor provisions a reasoned incompatibility may name:
/// the protected core an amendment may not lower. A reading that names anything
/// else refuses nothing.
const CORRIDOR_PROVISIONS: [&str; 14] = [
    "CorridorUniversalStanding",
    "CorridorMaterialFloor",
    "CorridorEqualProtection",
    "CorridorDueProcess",
    "CorridorCoreLiberties",
    "CorridorCommonsConstraints",
    "CorridorAnimalProtectedSubjectStatus",
    "CorridorSevereAvoidableAnimalSufferingProhibition",
    "CorridorDispensableAnimalKillingProhibition",
    "CorridorCategoricalRefusalsOnForce",
    "CorridorNonRefoulementAndCollectiveExpulsionBan",
    "CorridorPromptIndependentReviewOfDetention",
    "CorridorEffectiveRemedy",
    "CorridorAssemblyAndCourtCapacityToSit",
];

fn publication_body() -> Vec<String> {
    atoms(&[
        "complete($record, AmendmentCertifiedCandidate, $candidate)",
        "authorized($publisher, AmendmentPublicationAuthority, $record)",
        "authorized($review, IndependentAmendmentEffectReviewAuthority, $record)",
        "observe($publisher, $record, $candidate, AmendmentPublishedCandidateScope)",
        "observe($review, $record, $candidate, AmendmentPublishedCandidateScope)",
        "observe($publisher, $record, PublishedExactCandidateBytes, AmendmentPublicationDispositionScope)",
        "observe($review, $record, PublishedExactCandidateBytes, AmendmentPublicationDispositionScope)",
        "~($publisher = $review)",
        "~related($record, AmendmentPublicationConflict)",
    ])
}

fn selection_body() -> Vec<String> {
    let mut body = atoms(&[
        "complete($record, AmendmentPublishedCandidate, $candidate)",
        "authorized($selector, AmendmentEffectiveSelectionAuthority, $record)",
        "authorized($review, IndependentAmendmentEffectReviewAuthority, $record)",
        "observe($review, $record, $base, AmendmentBaseScope)",
        "observe($review, $record, $jurisdiction, AmendmentJurisdictionScope)",
        "~($selector = $review)",
        "~related($record, AmendmentSelectionConflict)",
    ]);
    for (value, scope) in [
        ("$base", "AmendmentSelectedPredecessorScope"),
        ("$candidate", "AmendmentSelectedCandidateScope"),
        ("$jurisdiction", "AmendmentSelectedJurisdictionScope"),
        ("$slot", "AmendmentEffectiveSlotScope"),
        ("$generation", "AmendmentSelectionGenerationScope"),
        ("CurrentUniqueSelection", "AmendmentSelectionStatusScope"),
        (
            "AtomicPredecessorComparisonPassed",
            "AmendmentSelectionComparisonScope",
        ),
    ] {
        observed(&mut body, &["$selector", "$review"], value, scope);
    }
    body
}

fn defect_body() -> Vec<String> {
    let mut body = atoms(&[
        "authorized($binder, AmendmentSourceBindingAuthority, $record)",
        "authorized($review, IndependentAmendmentEffectReviewAuthority, $record)",
        INDEPENDENT,
        "authorized($reader, AmendmentChallengeReaderAuthority, $record)",
        "~($reader = $binder)",
        "~($reader = $review)",
        "~($reader = $operator)",
    ]);
    for (value, scope) in [
        ("$candidate", "AmendmentCandidateScope"),
        ("$operator", "AmendmentOperatorScope"),
        ("$reader", "AmendmentChallengeReaderScope"),
        ("$end", "AmendmentReviewEndScope"),
        (
            "CurrentIndependentSourceDefect",
            "AmendmentDefectDispositionScope",
        ),
        (
            "PreserveEvidenceCorrectUseAndPermitFinalReview",
            "AmendmentDefectRemedyScope",
        ),
    ] {
        observed(&mut body, &["$binder", "$review"], value, scope);
    }
    body
}

fn nonresponse_body() -> Vec<String> {
    atoms(&[
        "authorized($witness, AmendmentNonresponseEvidenceAuthority, $request)",
        "authorized($review, IndependentAmendmentNonresponseReviewAuthority, $request)",
        "authorized($reader, AmendmentChallengeReaderAuthority, $request)",
        "authorized($alternate, AmendmentAlternateReaderAuthority, $request)",
        "challenge($requester, $reader, $request)",
        "observe($witness, $request, $reader, AmendmentNonrespondingReaderScope)",
        "observe($review, $request, $reader, AmendmentNonrespondingReaderScope)",
        "observe($witness, $request, $alternate, AmendmentAlternateReaderScope)",
        "observe($review, $request, $alternate, AmendmentAlternateReaderScope)",
        "observe($witness, $request, $target, AmendmentChallengeTargetScope)",
        "observe($review, $request, $target, AmendmentChallengeTargetScope)",
        "observe($witness, $request, $deadline, AmendmentNonresponseDeadlineScope)",
        "observe($review, $request, $deadline, AmendmentNonresponseDeadlineScope)",
        "observe($witness, $request, NoticeOpportunityAndDeadlineFailureEstablished, AmendmentNonresponseDispositionScope)",
        "observe($review, $request, NoticeOpportunityAndDeadlineFailureEstablished, AmendmentNonresponseDispositionScope)",
        "observe($witness, $request, CurrentIndependentNonresponseFinding, AmendmentNonresponseCurrentScope)",
        "observe($review, $request, CurrentIndependentNonresponseFinding, AmendmentNonresponseCurrentScope)",
        "~($witness = $review)",
        "~($reader = $alternate)",
        "~($witness = $reader)",
        "~($review = $reader)",
        "~($witness = $alternate)",
        "~($review = $alternate)",
        "~($requester = $reader)",
        "~($requester = $alternate)",
        "~($requester = $witness)",
        "~($requester = $review)",
    ])
}

fn rules() -> Vec<String> {
    let mut rules = Vec::new();
    for value in [
        "UnchangedEvidenceVocabulary",
        "ExplicitlyReviewedVocabularyChange",
    ] {
        rules.push(format!("all $writer: all $record: observe($writer, $record, {value}, AmendmentVocabularyDispositionScope) -> member({value}, AmendmentVocabularyDisposition)."));
    }
    // Item 71: a contrary compatibility reading is answered by the final review
    // or, when reasoned against a named corridor provision, refuses the
    // candidate; it no longer makes the whole record ambiguous.
    for scope in FIELDS
        .iter()
        .map(|(_, s)| *s)
        .filter(|s| *s != "AmendmentCompatibilityScope")
        .collect::<BTreeSet<_>>()
    {
        rules.push(format!("all $writer: all $record: all $value: observe($writer, $record, $value, {scope}) -> member({scope}, AmendmentSingleValueScope)."));
    }
    for provision in CORRIDOR_PROVISIONS {
        rules.push(format!("all $writer: all $record: observe($writer, $record, {provision}, AmendmentIncompatibleCoreProvisionScope) -> member({provision}, AmendmentCorridorProvisionVocabulary)."));
    }
    // The final reviewer holds neither attester role on the record it decides.
    rules.push("all $final: all $record: authorized($final, AmendmentFinalCompatibilityAuthority, $record) & observe($final, $record, FinalReviewFoundCandidateCompatible, AmendmentFinalCompatibilityScope) & ~authorized($final, AmendmentSourceBindingAuthority, $record) & ~authorized($final, IndependentAmendmentEffectReviewAuthority, $record) & ~($final = FSBOD_01) & ~($final = FSBOD_02) & ~($final = FSBOD_03) -> related($record, AmendmentFinalCompatibilityFound).".into());
    rules.push("all $writer: all $record: all $provision: all $change: all $reasons: related($writer, $record, AmendmentRecordAttester) & observe($writer, $record, ReasonedCoreIncompatibility, AmendmentCompatibilityScope) & observe($writer, $record, $provision, AmendmentIncompatibleCoreProvisionScope) & member($provision, AmendmentCorridorProvisionVocabulary) & observe($writer, $record, $change, AmendmentIncompatibleCandidateChangeScope) & observe($writer, $record, $reasons, AmendmentIncompatibilityReasonsScope) & ~related($record, AmendmentFinalCompatibilityFound) -> contradict($record, AmendmentSourceAuthorization).".into());
    rules.push("all $proponent: all $final: all $request: all $record: challenge($proponent, $final, $request) & authorized($final, AmendmentFinalCompatibilityAuthority, $record) & observe($proponent, $request, $record, AmendmentChallengedRecordScope) & ~($proponent = $final) -> obliged($final, DecideTheCandidatesCompatibilityWithTheCore, $record).".into());
    for role in [
        "AmendmentSourceBindingAuthority",
        "IndependentAmendmentEffectReviewAuthority",
    ] {
        rules.push(format!("all $writer: all $record: authorized($writer, {role}, $record) -> related($writer, $record, AmendmentRecordAttester)."));
    }
    // One attester recording two compatibility readings still makes the record
    // ambiguous; two attesters disagreeing is answered by the rules above.
    rules.push("all $a: all $record: all $x: all $y: related($a, $record, AmendmentRecordAttester) & observe($a, $record, $x, AmendmentCompatibilityScope) & observe($a, $record, $y, AmendmentCompatibilityScope) & ~($x = $y) -> related($record, AmendmentRecordAmbiguity).".into());
    rules.push("all $a: all $b: all $record: all $scope: all $x: all $y: related($a, $record, AmendmentRecordAttester) & related($b, $record, AmendmentRecordAttester) & member($scope, AmendmentSingleValueScope) & observe($a, $record, $x, $scope) & observe($b, $record, $y, $scope) & ~($x = $y) -> related($record, AmendmentRecordAmbiguity).".into());
    // A completed upstream result is not a licence to append a different
    // proposal/version to one writer and splice it into the downstream join.
    for role in [
        "StateFormSourceAuthority",
        "StateFormEvidenceAuthority",
        "IndependentStateFormReviewAuthority",
    ] {
        rules.push(format!("all $writer: all $consent: authorized($writer, {role}, $consent) -> related($writer, $consent, AmendmentConsentAttester)."));
    }
    for role in [
        "StateFormSourceAuthority",
        "StateFormEvidenceAuthority",
        "DecisionAdministrationAuthority",
        "IndependentCompletenessAssuranceAuthority",
        "ResultServiceAuthority",
        "InstitutionalExecutionAuthority",
    ] {
        rules.push(format!("all $writer: all $consent: authorized($writer, {role}, $consent) -> related($writer, $consent, AmendmentConsentOperator)."));
    }
    for (subject, scope) in [
        ("$consent", "SourceVersionScope"),
        ("$consent", "ResultScope"),
        ("$consent", "JurisdictionScope"),
        ("$consent", "AuthorityScope"),
        ("$result", "AmendmentProposalScope"),
        ("$result", "ResultCertificateScope"),
        ("$result", "AmendmentBaseScope"),
        ("$result", "AmendmentCandidateScope"),
    ] {
        rules.push(format!("all $binder: all $record: all $consent: all $result: all $a: all $b: all $x: all $y: authorized($binder, AmendmentSourceBindingAuthority, $record) & observe($binder, $record, $consent, AmendmentConsentRecordScope) & observe($binder, $record, $result, AmendmentConsentResultScope) & related($a, $consent, AmendmentConsentAttester) & related($b, $consent, AmendmentConsentAttester) & observe($a, {subject}, $x, {scope}) & observe($b, {subject}, $y, {scope}) & ~($x = $y) -> related($record, AmendmentRecordAmbiguity)."));
    }
    rules.push("all $a: all $b: all $first: all $second: all $transition: authorized($a, AmendmentSourceBindingAuthority, $first) & authorized($b, AmendmentSourceBindingAuthority, $second) & observe($a, $first, $transition, AmendmentTransitionScope) & observe($b, $second, $transition, AmendmentTransitionScope) & ~($first = $second) -> related($first, AmendmentRecordAmbiguity).".into());
    for (body, heads) in [
        (
            candidate_body(),
            vec![
                "complete($record, AmendmentCertifiedCandidate, $candidate)",
                "obliged($operator, PublishExactAmendmentCandidate, $record)",
            ],
        ),
        (
            final_review_body(),
            vec![
                "complete($record, AmendmentCertifiedCandidate, $candidate)",
                "obliged($operator, PublishExactAmendmentCandidate, $record)",
            ],
        ),
        (
            publication_body(),
            vec!["complete($record, AmendmentPublishedCandidate, $candidate)"],
        ),
        (
            selection_body(),
            vec!["complete($record, AmendmentEffectiveVersion, $candidate)"],
        ),
        (
            defect_body(),
            vec![
                "contradict($record, AmendmentSourceAuthorization)",
                "obliged($operator, PreserveAmendmentHistoryAndCorrectAffectedUse, $record)",
                "obliged($reader, ReviewAmendmentSourceDispute, $record)",
            ],
        ),
        (
            nonresponse_body(),
            vec![
                "obliged($alternate, ReviewAmendmentSourceDispute, $request)",
                "complete($request, AmendmentNonresponseEscalation, $alternate)",
            ],
        ),
    ] {
        for head in heads {
            rules.push(rule(&body, head));
        }
    }
    rules.push("all $requester: all $reader: all $request: challenge($requester, $reader, $request) & authorized($reader, AmendmentChallengeReaderAuthority, $request) & ~($requester = $reader) -> obliged($reader, ReviewAmendmentSourceDispute, $request).".into());
    // These conflict witnesses depend on supplied evidence, never on a conclusion
    // that consumes their absence. Conflicting selections fail closed together.
    for (role, scopes, marker) in [
        (
            "AmendmentPublicationAuthority",
            &[
                "AmendmentPublishedCandidateScope",
                "AmendmentPublicationDispositionScope",
            ][..],
            "AmendmentPublicationConflict",
        ),
        (
            "AmendmentEffectiveSelectionAuthority",
            &[
                "AmendmentSelectedPredecessorScope",
                "AmendmentSelectedCandidateScope",
                "AmendmentSelectedJurisdictionScope",
                "AmendmentEffectiveSlotScope",
                "AmendmentSelectionGenerationScope",
                "AmendmentSelectionStatusScope",
                "AmendmentSelectionComparisonScope",
            ][..],
            "AmendmentSelectionConflict",
        ),
    ] {
        for scope in scopes {
            rules.push(format!("all $a: all $b: all $record: all $x: all $y: authorized($a, {role}, $record) & authorized($b, IndependentAmendmentEffectReviewAuthority, $record) & observe($a, $record, $x, {scope}) & observe($b, $record, $y, {scope}) & ~($x = $y) -> related($record, {marker})."));
        }
    }
    rules.push("all $a: all $b: all $first: all $second: all $slot: all $generation: all $jurisdiction: authorized($a, AmendmentEffectiveSelectionAuthority, $first) & authorized($b, AmendmentEffectiveSelectionAuthority, $second) & observe($a, $first, $slot, AmendmentEffectiveSlotScope) & observe($b, $second, $slot, AmendmentEffectiveSlotScope) & observe($a, $first, $generation, AmendmentSelectionGenerationScope) & observe($b, $second, $generation, AmendmentSelectionGenerationScope) & observe($a, $first, $jurisdiction, AmendmentSelectedJurisdictionScope) & observe($b, $second, $jurisdiction, AmendmentSelectedJurisdictionScope) & observe($a, $first, CurrentUniqueSelection, AmendmentSelectionStatusScope) & observe($b, $second, CurrentUniqueSelection, AmendmentSelectionStatusScope) & ~($first = $second) -> related($first, AmendmentSelectionConflict).".into());
    rules
}

fn ground(text: &str, values: &BTreeMap<String, String>) -> String {
    Regex::new(r"\$[a-z][a-z0-9_]*")
        .unwrap()
        .replace_all(text, |c: &regex::Captures<'_>| values[&c[0]].clone())
        .into_owned()
}

fn values() -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for var in variables(&rules().join("\n")) {
        let suffix: String = var[1..]
            .split('_')
            .map(|s| s[..1].to_uppercase() + &s[1..])
            .collect();
        values.insert(var, format!("Enact{suffix}"));
    }
    values.insert("$vocabulary".into(), "UnchangedEvidenceVocabulary".into());
    values
}

fn facts(body: &[String], values: &BTreeMap<String, String>) -> String {
    body.iter()
        .filter(|s| {
            s.starts_with("authorized(") || s.starts_with("observe(") || s.starts_with("challenge(")
        })
        .filter(|s| !s.contains("$consent_source"))
        .map(|s| format!("{}.\n", ground(s, values)))
        .collect()
}

fn pins(queries: &[(&str, &str)], values: &BTreeMap<String, String>) -> String {
    let mut text = format!(":expect-pins {}\n", queries.len());
    for (q, expected) in queries {
        text += &format!("? {}.\n# => {expected}\n", ground(q, values));
    }
    text
}

pub(crate) fn ecological_example(
    context: &Context,
) -> Result<(String, BTreeMap<String, String>), Error> {
    let (consent, mapping) = state_form::amendment_example(context, "ordinary_amendment")?;
    let mut v = values();
    for (name, upstream) in [
        ("$consent_record", "$record"),
        ("$consent_result", "$result"),
        ("$proposal", "$proposal"),
        ("$certificate", "$amendment_certificate"),
        ("$legal_scope", "$legal_scope"),
    ] {
        v.insert(name.into(), mapping[upstream].clone());
    }
    Ok((format!("{consent}{}", facts(&candidate_body(), &v)), v))
}

pub(crate) fn ecological_corridor_counterfactual() -> Edit {
    let body = candidate_body();
    let head = "complete($record, AmendmentCertifiedCandidate, $candidate)";
    let weakened = body
        .iter()
        .filter(|atom| atom.as_str() != "~contradict($record, AmendmentSourceAuthorization)")
        .cloned()
        .collect::<Vec<_>>();
    Edit {
        before: rule(&body, head),
        after: rule(&weakened, head),
    }
}

pub(crate) fn ecological_effective_example(
    context: &Context,
) -> Result<(String, BTreeMap<String, String>), Error> {
    let (fixture, values) = ecological_example(context)?;
    Ok((
        fixture + &facts(&publication_body(), &values) + &facts(&selection_body(), &values),
        values,
    ))
}

pub(crate) fn generate(context: &Context, export: &mut Export) -> Result<(), Error> {
    let authored = rules();
    let block = format!(
        "{BEGIN}\n# Generated explicitly by ./generate.sh amendment; no host action is inferred.\n{}\n{END}",
        authored.join("\n")
    );
    let before = context.read(PATH)?;
    let after = match (before.matches(BEGIN).count(), before.matches(END).count()) {
        (0, 0) => format!("{}\n\n{block}\n", before.trim_end()),
        (1, 1) => {
            let start = before.find(BEGIN).unwrap();
            let stop = before.find(END).unwrap() + END.len();
            if stop <= start {
                return Err(Error::new("reversed amendment markers"));
            }
            format!("{}{block}{}", &before[..start], &before[stop..])
        }
        _ => return Err(Error::new("ambiguous amendment rule block")),
    };
    std::fs::write(context.path(PATH), after)?;
    let (consent, mapping) = state_form::amendment_example(context, "ordinary_amendment")?;
    let mut v = values();
    for (name, upstream) in [
        ("$consent_record", "$record"),
        ("$consent_result", "$result"),
        ("$proposal", "$proposal"),
        ("$certificate", "$amendment_certificate"),
        ("$legal_scope", "$legal_scope"),
    ] {
        v.insert(name.into(), mapping[upstream].clone());
    }
    let body = candidate_body();
    let candidate = facts(&body, &v);
    let published = facts(&publication_body(), &v);
    let selected = facts(&selection_body(), &v);
    let complete = "complete($record, AmendmentCertifiedCandidate, $candidate)";
    let publication = "complete($record, AmendmentPublishedCandidate, $candidate)";
    let effective = "complete($record, AmendmentEffectiveVersion, $candidate)";
    let mut add = |name: &str, base: &str, fixture: &str, text: String| {
        export.add_case(
            context,
            &format!("amendment-enactment/{name}"),
            base,
            fixture,
            &[("expect", &text)],
            vec![],
            true,
        )
    };
    add(
        "certified",
        "live",
        &format!("{consent}{candidate}"),
        pins(
            &[
                (complete, "TRUE"),
                (publication, "FALSE"),
                (effective, "FALSE"),
            ],
            &v,
        ),
    )?;
    add(
        "published",
        "live",
        &format!("{consent}{candidate}{published}"),
        pins(
            &[
                (complete, "TRUE"),
                (publication, "TRUE"),
                (effective, "FALSE"),
            ],
            &v,
        ),
    )?;
    add(
        "effective",
        "live",
        &format!("{consent}{candidate}{published}{selected}"),
        pins(
            &[
                (complete, "TRUE"),
                (publication, "TRUE"),
                (effective, "TRUE"),
            ],
            &v,
        ),
    )?;
    for (_, scope) in FIELDS.iter().filter(|(value, _)| {
        !value.starts_with('$')
            || [
                "$base",
                "$candidate",
                "$certificate",
                "$effects",
                "$vocabulary",
            ]
            .contains(value)
    }) {
        let missing: String = candidate
            .lines()
            .filter(|s| !s.ends_with(&format!(", {scope}).")))
            .map(|s| format!("{s}\n"))
            .collect();
        add(
            &format!("without-{scope}"),
            "live",
            &format!("{consent}{missing}{published}{selected}"),
            pins(&[(complete, "FALSE"), (effective, "FALSE")], &v),
        )?;
    }
    for (name, old, new) in [
        (
            "stale-base",
            "EnactBase, AmendmentObservedEffectiveBaseScope",
            "StaleBase, AmendmentObservedEffectiveBaseScope",
        ),
        (
            "replay",
            "FreshUnconsumedTransition",
            "PreviouslyConsumedTransition",
        ),
        (
            "semantic-mismatch",
            "BoundedEffectsMatchReviewedCandidate",
            "CandidateEffectsDiffer",
        ),
        (
            "incompatible",
            "PositiveCorridorCompatibility",
            "CorridorIncompatible",
        ),
        (
            "unauthorized-vocabulary",
            "UnchangedEvidenceVocabulary",
            "UnreviewedVocabularyChange",
        ),
        (
            "wrong-proposal",
            mapping["$proposal"].as_str(),
            "DifferentProposal",
        ),
        (
            "wrong-certificate",
            mapping["$amendment_certificate"].as_str(),
            "DifferentCertificate",
        ),
        ("self-review", "EnactReview", "EnactBinder"),
    ] {
        if !candidate.contains(old) {
            return Err(Error::new(format!("unused amendment mutation {name}")));
        }
        add(
            name,
            "live",
            &format!("{consent}{}", candidate.replace(old, new)),
            pins(&[(complete, "FALSE")], &v),
        )?;
    }
    add(
        "reviewed-vocabulary",
        "live",
        &format!(
            "{consent}{}",
            candidate.replace(
                "UnchangedEvidenceVocabulary",
                "ExplicitlyReviewedVocabularyChange"
            )
        ),
        pins(&[(complete, "TRUE")], &v),
    )?;
    // Item 71: a contrary reading of the core, answered by the final review.
    let positive = "observe(EnactReview, EnactRecord, PositiveCorridorCompatibility, AmendmentCompatibilityScope).";
    if !candidate.contains(positive) {
        return Err(Error::new("the reviewer's compatibility fact"));
    }
    let unreasoned = candidate.replace(
        positive,
        "observe(EnactReview, EnactRecord, CoreReadExpansively, AmendmentCompatibilityScope).",
    );
    let reasoned = candidate.replace(
        positive,
        "observe(EnactReview, EnactRecord, ReasonedCoreIncompatibility, AmendmentCompatibilityScope).\n\
         observe(EnactReview, EnactRecord, CorridorMaterialFloor, AmendmentIncompatibleCoreProvisionScope).\n\
         observe(EnactReview, EnactRecord, CandidateRemovesTheFoodEntitlement, AmendmentIncompatibleCandidateChangeScope).\n\
         observe(EnactReview, EnactRecord, PublishedIncompatibilityReasons, AmendmentIncompatibilityReasonsScope).",
    );
    let final_review = |actor: &str| {
        format!(
            "authorized({actor}, AmendmentFinalCompatibilityAuthority, EnactRecord).\n\
             observe({actor}, EnactRecord, FinalReviewFoundCandidateCompatible, AmendmentFinalCompatibilityScope).\n"
        )
    };
    let refused = "contradict($record, AmendmentSourceAuthorization)";
    let ambiguous = "related($record, AmendmentRecordAmbiguity)";
    let found = "related($record, AmendmentFinalCompatibilityFound)";
    for (name, fixture, queries) in [
        (
            "unreasoned-reading-alone",
            unreasoned.clone(),
            vec![
                (complete, "FALSE"),
                (ambiguous, "FALSE"),
                (refused, "FALSE"),
            ],
        ),
        (
            "unreasoned-reading-answered-by-final-review",
            format!("{unreasoned}{}", final_review("EnactFinal")),
            vec![
                (found, "TRUE"),
                (complete, "TRUE"),
                (refused, "FALSE"),
            ],
        ),
        (
            "final-review-by-the-effect-reviewer",
            format!("{unreasoned}{}", final_review("EnactReview")),
            vec![(found, "FALSE"), (complete, "FALSE")],
        ),
        (
            "final-review-by-the-binder",
            format!("{unreasoned}{}", final_review("EnactBinder")),
            vec![(found, "FALSE"), (complete, "FALSE")],
        ),
        (
            "final-review-by-the-assembly",
            format!("{unreasoned}{}", final_review("FSBOD_02")),
            vec![(found, "FALSE"), (complete, "FALSE")],
        ),
        (
            "final-review-by-the-operator",
            format!("{unreasoned}{}", final_review("EnactOperator")),
            vec![(complete, "FALSE")],
        ),
        (
            "final-review-without-authority",
            format!(
                "{unreasoned}observe(EnactFinal, EnactRecord, FinalReviewFoundCandidateCompatible, AmendmentFinalCompatibilityScope).\n"
            ),
            vec![(found, "FALSE"), (complete, "FALSE")],
        ),
        (
            "reasoned-breach",
            reasoned.clone(),
            vec![(refused, "TRUE"), (complete, "FALSE")],
        ),
        (
            "reasoned-breach-overturned-by-final-review",
            format!("{reasoned}{}", final_review("EnactFinal")),
            vec![(refused, "FALSE"), (complete, "TRUE")],
        ),
        (
            "reasoned-breach-naming-no-corridor-provision",
            reasoned.replace("CorridorMaterialFloor", "OrdinaryTaxRate"),
            vec![(refused, "FALSE"), (complete, "FALSE")],
        ),
        (
            "reasoned-breach-without-reasons",
            reasoned
                .lines()
                .filter(|l| !l.contains("AmendmentIncompatibilityReasonsScope"))
                .map(|l| format!("{l}\n"))
                .collect::<String>(),
            vec![(refused, "FALSE"), (complete, "FALSE")],
        ),
        (
            "reasoned-breach-without-named-change",
            reasoned
                .lines()
                .filter(|l| !l.contains("AmendmentIncompatibleCandidateChangeScope"))
                .map(|l| format!("{l}\n"))
                .collect::<String>(),
            vec![(refused, "FALSE"), (complete, "FALSE")],
        ),
        (
            "one-attester-two-readings",
            format!(
                "{candidate}observe(EnactReview, EnactRecord, CoreReadExpansively, AmendmentCompatibilityScope).\n{}",
                final_review("EnactFinal")
            ),
            vec![(ambiguous, "TRUE"), (complete, "FALSE")],
        ),
    ] {
        add(name, "live", &format!("{consent}{fixture}"), pins(&queries, &v))?;
    }
    let decision = "obliged($final, DecideTheCandidatesCompatibilityWithTheCore, $record)";
    for (name, challenger, expected) in [
        ("challenge-obliges-the-final-review", "AmendmentProponent", "TRUE"),
        ("final-reviewer-cannot-challenge-itself", "EnactFinal", "FALSE"),
    ] {
        add(
            name,
            "live",
            &format!(
                "{consent}{unreasoned}authorized(EnactFinal, AmendmentFinalCompatibilityAuthority, EnactRecord).\n\
                 challenge({challenger}, EnactFinal, EnactCompatibilityRequest).\n\
                 observe({challenger}, EnactCompatibilityRequest, EnactRecord, AmendmentChallengedRecordScope).\n"
            ),
            pins(&[(decision, expected), (complete, "FALSE")], &v),
        )?;
    }
    let mut substituted = v.clone();
    substituted.insert("$candidate".into(), "SubstitutedCandidate".into());
    add(
        "candidate-not-consented",
        "live",
        &format!(
            "{consent}{}",
            candidate.replace("EnactCandidate", "SubstitutedCandidate")
        ),
        pins(&[(complete, "FALSE")], &substituted),
    )?;
    for (name, actor) in [
        (
            "result-service-as-effect-reviewer",
            mapping["$service"].as_str(),
        ),
        ("assembly-as-effect-reviewer", "FSBOD_02"),
    ] {
        add(
            name,
            "live",
            &format!("{consent}{}", candidate.replace("EnactReview", actor)),
            pins(&[(complete, "FALSE")], &v),
        )?;
    }
    let without_candidate: String = consent
        .lines()
        .filter(|s| !s.contains("AmendmentCandidateScope"))
        .map(|s| format!("{s}\n"))
        .collect();
    add(
        "political-result-not-source-bound",
        "live",
        &format!("{without_candidate}{candidate}"),
        pins(&[(complete, "FALSE")], &v),
    )?;
    add(
        "no-political-consent",
        "live",
        &candidate,
        pins(&[(complete, "FALSE")], &v),
    )?;
    for (name, scope, old) in [
        (
            "upstream-proposal-splice",
            "AmendmentProposalScope",
            mapping["$proposal"].as_str(),
        ),
        (
            "upstream-certificate-splice",
            "ResultCertificateScope",
            mapping["$amendment_certificate"].as_str(),
        ),
        (
            "upstream-candidate-splice",
            "AmendmentCandidateScope",
            "EnactCandidate",
        ),
    ] {
        let poisoned = format!(
            "{consent}observe({}, {}, SplicedValue, {scope}).\n{}",
            mapping["$source"],
            mapping["$result"],
            candidate.replace(old, "SplicedValue")
        );
        let mut named = v.clone();
        if scope == "AmendmentCandidateScope" {
            named.insert("$candidate".into(), "SplicedValue".into());
        }
        add(
            name,
            "live",
            &poisoned,
            pins(
                &[
                    (complete, "FALSE"),
                    ("related($record, AmendmentRecordAmbiguity)", "TRUE"),
                ],
                &named,
            ),
        )?;
    }
    let without_completeness: String = consent
        .lines()
        .filter(|s| !s.contains("CompleteUniqueCertificateSet"))
        .map(|s| format!("{s}\n"))
        .collect();
    add(
        "incomplete-political-certificate",
        "live",
        &format!("{without_completeness}{candidate}"),
        pins(&[(complete, "FALSE")], &v),
    )?;
    for branch in [
        "directly_regional_settlement_amendment",
        "competence_boundary_amendment",
    ] {
        let (regional, _) = state_form::amendment_example(context, branch)?;
        add(
            branch,
            "live",
            &format!("{regional}{candidate}"),
            pins(&[(complete, "TRUE")], &v),
        )?;
    }
    let conflict =
        "observe(EnactBinder, EnactRecord, DivergentCandidate, AmendmentCandidateScope).\n";
    add(
        "divergent-candidate-record",
        "live",
        &format!("{consent}{candidate}{conflict}"),
        pins(
            &[
                (complete, "FALSE"),
                ("related($record, AmendmentRecordAmbiguity)", "TRUE"),
            ],
            &v,
        ),
    )?;
    let duplicate = "authorized(DifferentBinder, AmendmentSourceBindingAuthority, DuplicateRecord).\nobserve(DifferentBinder, DuplicateRecord, EnactTransition, AmendmentTransitionScope).\n";
    add(
        "duplicate-transition-record",
        "live",
        &format!("{consent}{candidate}{duplicate}"),
        pins(&[(complete, "FALSE")], &v),
    )?;
    for (role, actor) in [
        ("AmendmentSourceBindingAuthority", "EnactBinder"),
        ("IndependentAmendmentEffectReviewAuthority", "EnactReview"),
        ("AmendmentChallengeReaderAuthority", "EnactReader"),
        ("AmendmentAlternateReaderAuthority", "EnactAlternate"),
    ] {
        let removed =
            candidate.replace(&format!("authorized({actor}, {role}, EnactRecord).\n"), "");
        add(
            &format!("unauthorized-{role}"),
            "live",
            &format!("{consent}{removed}"),
            pins(&[(complete, "FALSE")], &v),
        )?;
    }
    add(
        "wrong-publication",
        "live",
        &format!(
            "{consent}{candidate}{}",
            published.replace("EnactCandidate", "WrongPublishedBytes")
        ),
        pins(&[(complete, "TRUE"), (publication, "FALSE")], &v),
    )?;
    add(
        "wrong-selected-base",
        "live",
        &format!(
            "{consent}{candidate}{published}{}",
            selected.replace(
                "EnactBase, AmendmentSelectedPredecessorScope",
                "StaleBase, AmendmentSelectedPredecessorScope",
            )
        ),
        pins(&[(publication, "TRUE"), (effective, "FALSE")], &v),
    )?;
    add(
        "superseded",
        "live",
        &format!(
            "{consent}{candidate}{published}{}",
            selected.replace("CurrentUniqueSelection", "SupersededSelection")
        ),
        pins(&[(publication, "TRUE"), (effective, "FALSE")], &v),
    )?;
    let second = selected.replace("EnactRecord", "CompetingRecord");
    add(
        "competing-selection",
        "live",
        &format!("{consent}{candidate}{published}{selected}{second}"),
        pins(
            &[
                (effective, "FALSE"),
                ("related($record, AmendmentSelectionConflict)", "TRUE"),
                (
                    "related(CompetingRecord, AmendmentSelectionConflict)",
                    "TRUE",
                ),
            ],
            &v,
        ),
    )?;
    let defect = facts(&defect_body(), &v);
    let start = pins(&[(effective, "TRUE")], &v).replace(":expect-pins 1", ":expect-pins 8");
    let finish = pins(
        &[
            (complete, "FALSE"),
            (publication, "FALSE"),
            (effective, "FALSE"),
            ("contradict($record, AmendmentSourceAuthorization)", "TRUE"),
            (
                "obliged($operator, PreserveAmendmentHistoryAndCorrectAffectedUse, $record)",
                "TRUE",
            ),
            ("entitled(Adam, event { eats() })", "TRUE"),
            ("false(Jala)", "FALSE"),
        ],
        &v,
    );
    add(
        "defect-sequence",
        "live",
        &format!("{consent}{candidate}{published}{selected}"),
        format!(
            "{start}{defect}{}",
            finish.lines().skip(1).collect::<Vec<_>>().join("\n")
        ),
    )?;
    let challenge = facts(&nonresponse_body(), &v);
    let initial: String = challenge
        .lines()
        .filter(|s| s.starts_with("challenge(") || s.contains("AmendmentChallengeReaderAuthority"))
        .map(|s| format!("{s}\n"))
        .collect();
    let mut sequence = pins(
        &[
            (
                "obliged($reader, ReviewAmendmentSourceDispute, $request)",
                "TRUE",
            ),
            (
                "obliged($alternate, ReviewAmendmentSourceDispute, $request)",
                "FALSE",
            ),
        ],
        &v,
    )
    .replace(":expect-pins 2", ":expect-pins 3");
    sequence += &challenge;
    sequence += &format!(
        "? {}.\n# => TRUE\n",
        ground(
            "obliged($alternate, ReviewAmendmentSourceDispute, $request)",
            &v
        )
    );
    add("nonresponse-sequence", "live", &initial, sequence)?;
    add(
        "nonresponse-positive",
        "live",
        &challenge,
        pins(
            &[(
                "complete($request, AmendmentNonresponseEscalation, $alternate)",
                "TRUE",
            )],
            &v,
        ),
    )?;
    for (name, old, replacement) in [
        (
            "nonresponse-not-current",
            "CurrentIndependentNonresponseFinding",
            "OldNonresponseFinding",
        ),
        (
            "nonresponse-no-deadline",
            "NoticeOpportunityAndDeadlineFailureEstablished",
            "OnlySilenceObserved",
        ),
        ("nonresponse-self-review", "EnactReview", "EnactWitness"),
        (
            "nonresponse-no-independent-alternate",
            "EnactAlternate",
            "EnactReader",
        ),
    ] {
        let changed = challenge.replace(old, replacement);
        let mut named = v.clone();
        if name == "nonresponse-no-independent-alternate" {
            named.insert("$alternate".into(), "EnactReader".into());
        }
        add(
            name,
            "live",
            &changed,
            pins(
                &[(
                    "complete($request, AmendmentNonresponseEscalation, $alternate)",
                    "FALSE",
                )],
                &named,
            ),
        )?;
    }
    drop(add);
    // Deliberately remove only the independence conjunct from the certification
    // producing rule. The ordinary self-review case above is its paired control.
    let exact = rule(&body, complete);
    let edit = Edit {
        before: exact.clone(),
        after: exact.replace(&format!(" & {INDEPENDENT}"), ""),
    };
    super::apply_edits(&context.read(PATH)?, std::slice::from_ref(&edit))?;
    export.bases.insert(
        "amendment-self-review".into(),
        Base {
            base: Some("live".into()),
            edits: vec![edit],
            ..Base::default()
        },
    );
    export.add_case(
        context,
        "amendment-enactment/counterfactual-self-review",
        "amendment-self-review",
        &format!(
            "{consent}{}",
            candidate.replace("EnactReview", "EnactBinder")
        ),
        &[("expect", &pins(&[(complete, "TRUE")], &v))],
        vec![],
        true,
    )?;
    for (name, premise, changed) in [
        (
            "counterfactual-no-consent",
            "complete($consent_result, FSPOW_037, $consent_record)",
            &candidate,
        ),
        (
            "counterfactual-no-effect-match",
            "observe($review, $record, BoundedEffectsMatchReviewedCandidate, AmendmentEffectDispositionScope)",
            &candidate,
        ),
    ] {
        let mut weakened = body.clone();
        weakened.retain(|a| {
            if name == "counterfactual-no-consent" {
                !a.contains("$consent_source")
                    && !a.starts_with("complete($consent_result")
                    && !a.starts_with("authority(FSBOD_02")
            } else {
                a != premise
            }
        });
        let fixture = if name == "counterfactual-no-consent" {
            changed.clone()
        } else {
            format!(
                "{consent}{}",
                changed.replace(&format!("{}.\n", ground(premise, &v)), "")
            )
        };
        if name == "counterfactual-no-effect-match" {
            export.add_case(
                context,
                "amendment-enactment/effect-review-missing",
                "live",
                &fixture,
                &[("expect", &pins(&[(complete, "FALSE")], &v))],
                vec![],
                true,
            )?;
        }
        let edit = Edit {
            before: rule(&body, complete),
            after: rule(&weakened, complete),
        };
        let base = format!("amendment-{name}");
        super::apply_edits(&context.read(PATH)?, std::slice::from_ref(&edit))?;
        export.bases.insert(
            base.clone(),
            Base {
                base: Some("live".into()),
                edits: vec![edit],
                ..Base::default()
            },
        );
        export.add_case(
            context,
            &format!("amendment-enactment/{name}"),
            &base,
            &fixture,
            &[("expect", &pins(&[(complete, "TRUE")], &v))],
            vec![],
            true,
        )?;
    }
    let mut spliced = v.clone();
    spliced.insert("$candidate".into(), "SplicedValue".into());
    let fixture = format!(
        "{consent}observe({}, {}, SplicedValue, AmendmentCandidateScope).\n{}",
        mapping["$source"],
        mapping["$result"],
        candidate.replace("EnactCandidate", "SplicedValue")
    );
    let edit = Edit {
        before: rule(&body, complete),
        after: rule(
            &body
                .iter()
                .filter(|a| *a != "~related($record, AmendmentRecordAmbiguity)")
                .cloned()
                .collect::<Vec<_>>(),
            complete,
        ),
    };
    super::apply_edits(&context.read(PATH)?, std::slice::from_ref(&edit))?;
    export.bases.insert(
        "amendment-candidate-splice".into(),
        Base {
            base: Some("live".into()),
            edits: vec![edit],
            ..Base::default()
        },
    );
    export.add_case(
        context,
        "amendment-enactment/counterfactual-candidate-splice",
        "amendment-candidate-splice",
        &fixture,
        &[("expect", &pins(&[(complete, "TRUE")], &spliced))],
        vec![],
        true,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_is_explicit_isolated_and_repeatable() {
        let live = Context::discover().unwrap();
        let directory = tempfile::tempdir().unwrap();
        let context = Context::from_test_root(directory.path().to_owned());
        std::fs::create_dir_all(context.path("book-1/source")).unwrap();
        std::fs::create_dir_all(context.path("tests/pins")).unwrap();
        for path in [PATH, "book-1/source/state-form-source.json"] {
            std::fs::copy(live.path(path), context.path(path)).unwrap();
        }
        std::fs::write(context.path("tests/pins/suites.json"), r#"{"bases":{"live":{"path":"book-1/source/constitution.nibli"}},"cases":[{"id":"unrelated/control","base":"live","pins":["control.pins.nibli"],"allow_shell":true}]}"#).unwrap();
        let before = context.read(PATH).unwrap();
        super::super::run(&context, "amendment").unwrap();
        let after = context.read(PATH).unwrap();
        assert_eq!(
            before.split_once(BEGIN).unwrap().0,
            after.split_once(BEGIN).unwrap().0
        );
        assert_eq!(
            before.split_once(END).unwrap().1,
            after.split_once(END).unwrap().1
        );
        let inventory = context.read("tests/pins/suites.json").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&inventory).unwrap();
        assert_eq!(parsed["cases"][0]["allow_shell"], true);
        super::super::run(&context, "amendment").unwrap();
        assert_eq!(context.read(PATH).unwrap(), after);
        assert_eq!(context.read("tests/pins/suites.json").unwrap(), inventory);
    }

    #[test]
    fn new_heads_are_bounded_and_do_not_consume_old_law_labels() {
        for rule in rules() {
            let (body, head) = rule.split_once(" -> ").unwrap();
            assert!(!body.contains("become("));
            assert!(
                [
                    "member(",
                    "related(",
                    "complete(",
                    "obliged(",
                    "contradict("
                ]
                .iter()
                .any(|prefix| head.starts_with(prefix)),
                "{head}"
            );
            assert!(!rule.contains("admits("));
        }
    }
}
