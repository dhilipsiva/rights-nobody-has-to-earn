// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

#[test]
fn every_thread_ends_in_one_of_the_five_states_with_a_rerun() {
    let context = Context::discover().expect("repository");
    let receipts = receipts(&context).expect("receipts source");
    validate(&context, &receipts).expect("every receipt binds to a thread and a rerun");
    assert!(
        receipts.len() >= 9,
        "the receipt table shrank to {}; a thread the book still tells cannot \
         lose its receipt",
        receipts.len()
    );
}

#[test]
fn disclosure_is_never_counted_as_closure() {
    let context = Context::discover().expect("repository");
    let receipts = receipts(&context).expect("receipts source");
    let closed = receipts
        .iter()
        .filter(|receipt| receipt.state == "resolved-for-claim")
        .count();
    assert!(
        closed < receipts.len(),
        "every thread is marked resolved. This design has never been in that \
         state, and a table that says so is the failure this item exists to \
         prevent."
    );
    // The two states that are honest about there being no repair must still
    // carry what does not follow — that is the whole content of the entry.
    for receipt in &receipts {
        if receipt.state == "open-defect" || receipt.state == "irreducible-limitation" {
            assert!(
                receipt.still_not_established.len() > 40,
                "{}: an unrepaired thread has to say what it does not establish",
                receipt.id
            );
        }
    }
}

/// The narration detector is what stops a chapter claiming a repair in prose
/// and leaving it without an ending, so both halves are checked here: it still
/// fires on the book's own idiom, and it does not fire on a sentence that
/// denies a repair rather than reporting one.
#[test]
fn a_denied_repair_is_not_a_narrated_one() {
    for narration in [
        "The repair is to give the record that entry.",
        "Under an earlier version of this design it worked.",
        "That hole is now closed in two places.",
        "the resolution was a deletion",
        "the alarm no longer has a way to stay silent",
    ] {
        assert!(
            narrates_a_repair(narration),
            "the detector stopped seeing a narrated repair in {narration:?}"
        );
    }
    for denial in [
        "What the design produces is a correction owed, and the duty to repair \
         is not the repair.",
        "A duty is not a delivery and a remedy is not a restoration.",
    ] {
        assert!(
            !narrates_a_repair(denial),
            "the detector read a denial as a narrated repair: {denial:?}"
        );
    }
}
