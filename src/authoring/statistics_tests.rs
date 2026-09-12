// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

fn bounded_source(source: &Source) -> bool {
    let forbidden = Regex::new(
        r"\b(person|false|prisoner|severe|owe|entitled|decide|lose|reward|pay|promise)\s*\(",
    )
    .unwrap();
    let person_field = Regex::new(r"(?i)(susceptibility|loyalty|personalrisk|personalworth|personalguilt|personscore|protectedidentity)").unwrap();
    let allowed_heads = [
        "authority($operator, $purpose, $record)",
        "obliged($operator, PublishStatisticsMethodsUncertaintyExclusionsAndUndercount, $record)",
        "obliged($operator, ProtectStatisticsParticipantsAndSupportersFromRetaliation, $record)",
        "permits($operator, PublishPrivacySafeStatistics, $record)",
        "complete($record, RebuttableAggregateEqualityPresumption, $proceeding)",
        "permits($reader, InitiateIndependentSystemicEqualityAudit, $record)",
        "obliged($respondent, DiscloseMeasureBasisAndAnswerEqualityJustification, $proceeding)",
        "obliged($reader, HearRebuttalAndCorrectAggregatePresumption, $proceeding)",
        "permits($requester, $request_kind, $record)",
        "obliged($operator, ProvidePrivateStatisticsAccessCorrectionAndChallenge, $record)",
        "contradict($target, StatisticsUseAuthorization)",
        "obliged($reader, CorrectDisplacedAggregatePresumption, $proceeding)",
        "obliged($reader, ReviewAndCorrectUnsupportedStatisticalUse, $target)",
        "obliged($operator, PublishPrivacySafeStatisticsCorrection, $target)",
        "obliged($alternate, ReviewStatisticsRequestAndSecureIndependentRemedy, $target)",
        "obliged($operator, ProtectStatisticsParticipantsAndSupportersFromRetaliation, $target)",
    ];
    source.contracts.iter().all(|c| {
        !forbidden.is_match(&premises(source, c).join(" "))
            && c.heads.iter().all(|h| allowed_heads.contains(&h.as_str()))
            && fields(source, c)
                .iter()
                .all(|f| !person_field.is_match(&f[1]))
    }) && source
        .vocabularies
        .iter()
        .all(|(scope, _, values)| match scope.as_str() {
            "StatisticsPurposeScope" => {
                values.iter().map(String::as_str).collect::<Vec<_>>()
                    == [
                        "OfficialStatistics",
                        "PublicPlanning",
                        "EqualityDiagnostics",
                    ]
            }
            "StatisticsProceedingKindScope" => {
                values.iter().map(String::as_str).collect::<Vec<_>>()
                    == [
                        "CivilRemedialEquality",
                        "AdministrativeRemedialEquality",
                        "ConstitutionalRemedialEquality",
                    ]
            }
            "StatisticsRequestKindScope" => {
                values.iter().map(String::as_str).collect::<Vec<_>>()
                    == [
                        "InspectOwnStatisticalEvidence",
                        "CorrectOwnStatisticalEvidence",
                        "RequestLawfulStatisticalDeletion",
                        "ChallengeStatisticalUse",
                    ]
            }
            _ => true,
        })
}

#[test]
fn source_has_no_person_consequence_or_unbounded_consumer() {
    let context = Context::discover().unwrap();
    let source: Source = serde_json::from_str(&context.read(SOURCE).unwrap()).unwrap();
    assert!(bounded_source(&source));
    let mut hostile = source.clone();
    hostile.contracts[2].heads.push("false($dataset)".into());
    assert!(!bounded_source(&hostile));
    let mut hostile = source.clone();
    hostile.vocabularies[0].2.push("IndividualRisk".into());
    assert!(!bounded_source(&hostile));
    let mut hostile = source.clone();
    hostile.contracts[2]
        .fields
        .push(["$dataset".into(), "StatisticsPersonalRiskScope".into()]);
    assert!(!bounded_source(&hostile));
    let mut hostile = source.clone();
    hostile.contracts[2].extra.push("person($dataset)".into());
    assert!(!bounded_source(&hostile));
}

#[test]
fn consumers_reject_named_and_generic_person_side_readers() {
    let context = Context::discover().unwrap();
    let source: Source = serde_json::from_str(&context.read(SOURCE).unwrap()).unwrap();
    let allowed = rules(&source).into_iter().collect::<BTreeSet<_>>();
    let named = Regex::new(r"\b(Statistics\w*|Statistical\w*|ReviewedStatistical\w*|RebuttableAggregateEqualityPresumption|DiscloseMeasureBasisAndAnswerEqualityJustification|HearRebuttalAndCorrectAggregatePresumption)\b").unwrap();
    let generic =
        Regex::new(r"\b(complete|authority|permits|obliged|related)\(\$\w+,\s*\$\w+,\s*\$\w+\)")
            .unwrap();
    let personal_head =
        Regex::new(r"->\s*(person|false|prisoner|severe|owe|entitled|decide|lose|reward)\(")
            .unwrap();
    let check = |text: &str| {
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .all(|line| {
                if named.is_match(line) {
                    allowed.contains(line)
                } else {
                    !(generic.is_match(line) && personal_head.is_match(line))
                }
            })
    };
    let constitution = context.read("new-book-plans/constitution.nibli").unwrap();
    assert!(
        check(&constitution),
        "unreviewed statistical consumer in live constitution"
    );
    let hostile = context
        .read("tests/fixtures/statistics-person-side.nibli")
        .unwrap();
    for line in hostile
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        nibli_session::CoreSession::new()
            .compile_text(line)
            .expect("real corpus-valid hostile rule");
        assert!(
            !check(&format!("{constitution}\n{line}")),
            "watched consumer must fail: {line}"
        );
    }
}

#[test]
fn fields_are_single_valued_and_every_rule_compiles() {
    let context = Context::discover().unwrap();
    let source: Source = serde_json::from_str(&context.read(SOURCE).unwrap()).unwrap();
    for contract in &source.contracts {
        let mut scopes = BTreeSet::new();
        for [_, scope] in fields(&source, contract) {
            assert!(scopes.insert(scope), "duplicate field in {}", contract.id);
        }
        let values = bindings(&source, contract, "StatisticsTest");
        if contract.use_dependency {
            assert!(
                fixture(&source, contract, &values).contains("StatisticsUseAuthorizationScope")
            );
        }
    }
    for statement in rules(&source) {
        nibli_session::CoreSession::new()
            .compile_text(&statement)
            .expect("real corpus-valid statistical rule");
    }
}
