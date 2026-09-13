use super::Transactions;
use crate::desktop::resize::Batch;
use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};

#[test]
fn completed_collection_does_not_block_configuration() {
    let mut transactions = Transactions {
        collecting: true,
        ..Default::default()
    };
    // end_resize_batch leaves collecting set even after its cohort is released.
    assert!(transactions.configuration_ready());
    transactions.depth = 1;
    assert!(!transactions.configuration_ready());
}

#[test]
fn pending_cohort_still_blocks_configuration() {
    let transactions = Transactions {
        batch: Some(Batch {
            workspace: 0,
            released: Arc::new(AtomicBool::new(false)),
            deadline: Instant::now(),
            clients: Vec::new(),
        }),
        ..Default::default()
    };
    assert!(!transactions.configuration_ready());
}
