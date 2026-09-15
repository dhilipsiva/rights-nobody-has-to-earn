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
