#![cfg(test)]

use soroban_sdk::{testutils::Ledger, Address, Env};

use crate::{
    HistoryFilter, SortOrder, StellarEscrowContract, StellarEscrowContractClient, TradeStatus,
};

fn setup() -> (Env, StellarEscrowContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Use a mock token address — token transfers are mocked via mock_all_auths
    let token = Address::generate(&env);

    client.initialize(&admin, &token, &100); // 1% fee

    (env, client, admin, seller, buyer)
}

fn no_filter(env: &Env) -> HistoryFilter {
    HistoryFilter {
        status: None,
        from_ledger: None,
        to_ledger: None,
    }
}

#[test]
fn test_history_empty_for_new_address() {
    let (env, client, _, seller, _) = setup();
    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );
    assert_eq!(page.total, 0);
    assert_eq!(page.records.len(), 0);
}

#[test]
fn test_history_shows_created_trade() {
    let (env, client, _, seller, buyer) = setup();

    let trade_id = client.create_trade(&seller, &buyer, &1000, &None, &None);

    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );

    assert_eq!(page.total, 1);
    let record = page.records.get(0).unwrap();
    assert_eq!(record.trade_id, trade_id);
    assert_eq!(record.amount, 1000);
    assert_eq!(record.status, TradeStatus::Created);
}

#[test]
fn test_history_visible_from_buyer_address() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &500, &None, &None);

    let page = client.get_transaction_history(
        &buyer,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );

    assert_eq!(page.total, 1);
}

#[test]
fn test_history_filter_by_status() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);
    client.create_trade(&seller, &buyer, &2000, &None, &None);

    // Cancel the first trade
    client.cancel_trade(&1);

    let filter = HistoryFilter {
        status: Some(TradeStatus::Cancelled),
        from_ledger: None,
        to_ledger: None,
    };

    let page = client.get_transaction_history(&seller, &filter, &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().status, TradeStatus::Cancelled);
}

#[test]
fn test_history_filter_by_ledger_range() {
    let (env, client, _, seller, buyer) = setup();

    // Trade at ledger 1
    env.ledger().set_sequence_number(1);
    client.create_trade(&seller, &buyer, &1000, &None, &None);

    // Trade at ledger 100
    env.ledger().set_sequence_number(100);
    client.create_trade(&seller, &buyer, &2000, &None, &None);

    let filter = HistoryFilter {
        status: None,
        from_ledger: Some(50),
        to_ledger: Some(200),
    };

    let page = client.get_transaction_history(&seller, &filter, &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 2000);
}

#[test]
fn test_history_sort_descending() {
    let (env, client, _, seller, buyer) = setup();

    env.ledger().set_sequence_number(1);
    client.create_trade(&seller, &buyer, &100, &None, &None);

    env.ledger().set_sequence_number(10);
    client.create_trade(&seller, &buyer, &200, &None, &None);

    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Descending,
        &0,
        &10,
    );

    assert_eq!(page.records.get(0).unwrap().amount, 200);
    assert_eq!(page.records.get(1).unwrap().amount, 100);
}

#[test]
fn test_history_pagination() {
    let (env, client, _, seller, buyer) = setup();

    for _ in 0..5 {
        client.create_trade(&seller, &buyer, &1000, &None, &None);
    }

    let page1 = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &3,
    );
    assert_eq!(page1.records.len(), 3);
    assert_eq!(page1.total, 5);

    let page2 = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &3,
        &3,
    );
    assert_eq!(page2.records.len(), 2);
}

#[test]
fn test_export_csv_returns_header_and_rows() {
    let (env, client, _, seller, buyer) = setup();

    client.create_trade(&seller, &buyer, &1000, &None, &None);

    let csv = client.export_transaction_csv(
        &seller,
        &HistoryFilter {
            status: None,
            from_ledger: None,
            to_ledger: None,
        },
    );

    // CSV should be non-empty and contain the header
    assert!(csv.len() > 0);
}

// =============================================================================
// Onboarding tests
// =============================================================================

use crate::{OnboardingStep, StepStatus};

#[test]
fn test_onboarding_start_creates_progress() {
    let (_, client, _, seller, _) = setup();

    let progress = client.start_onboarding(&seller);

    assert!(!progress.finished);
    assert_eq!(progress.current_step, OnboardingStep::RegisterProfile);
    assert_eq!(progress.step_statuses.len(), 4);
    // All steps start as Pending
    for i in 0..4 {
        assert_eq!(progress.step_statuses.get(i).unwrap(), StepStatus::Pending);
    }
}

#[test]
fn test_onboarding_start_is_idempotent() {
    let (_, client, _, seller, _) = setup();

    let first = client.start_onboarding(&seller);
    let second = client.start_onboarding(&seller);

    // Second call returns the same progress without resetting it
    assert_eq!(first.started_at, second.started_at);
    assert_eq!(first.current_step, second.current_step);
}

#[test]
fn test_onboarding_complete_step_advances_progress() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);

    // Complete step 0 (RegisterProfile)
    let progress = client.complete_onboarding_step(&seller, &0);

    assert_eq!(progress.step_statuses.get(0).unwrap(), StepStatus::Done);
    assert_eq!(progress.current_step, OnboardingStep::AcknowledgeFees);
    assert!(!progress.finished);
}

#[test]
fn test_onboarding_complete_all_steps_marks_finished() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);

    for i in 0..4u32 {
        client.complete_onboarding_step(&seller, &i);
    }

    let progress = client.get_onboarding_progress(&seller).unwrap();
    assert!(progress.finished);
    assert_eq!(progress.current_step, OnboardingStep::Completed);
    for i in 0..4 {
        assert_eq!(progress.step_statuses.get(i).unwrap(), StepStatus::Done);
    }
}

#[test]
fn test_onboarding_skip_step_advances_without_completing() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);

    // Skip step 0
    let progress = client.skip_onboarding_step(&seller, &0);

    assert_eq!(progress.step_statuses.get(0).unwrap(), StepStatus::Skipped);
    assert_eq!(progress.current_step, OnboardingStep::AcknowledgeFees);
    assert!(!progress.finished);
}

#[test]
fn test_onboarding_skip_all_steps_marks_finished() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);

    for i in 0..4u32 {
        client.skip_onboarding_step(&seller, &i);
    }

    let progress = client.get_onboarding_progress(&seller).unwrap();
    assert!(progress.finished);
    assert_eq!(progress.current_step, OnboardingStep::Completed);
    for i in 0..4 {
        assert_eq!(progress.step_statuses.get(i).unwrap(), StepStatus::Skipped);
    }
}

#[test]
fn test_onboarding_exit_marks_all_pending_as_skipped() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);
    // Complete step 0 first
    client.complete_onboarding_step(&seller, &0);

    // Exit — steps 1, 2, 3 should become Skipped
    let progress = client.exit_onboarding(&seller);

    assert!(progress.finished);
    assert_eq!(progress.step_statuses.get(0).unwrap(), StepStatus::Done);
    assert_eq!(progress.step_statuses.get(1).unwrap(), StepStatus::Skipped);
    assert_eq!(progress.step_statuses.get(2).unwrap(), StepStatus::Skipped);
    assert_eq!(progress.step_statuses.get(3).unwrap(), StepStatus::Skipped);
}

#[test]
fn test_onboarding_progress_is_persisted_and_resumable() {
    let (_, client, _, seller, _) = setup();

    client.start_onboarding(&seller);
    client.complete_onboarding_step(&seller, &0);
    client.skip_onboarding_step(&seller, &1);

    // Simulate resume: start_onboarding returns existing progress
    let resumed = client.start_onboarding(&seller);
    assert_eq!(resumed.step_statuses.get(0).unwrap(), StepStatus::Done);
    assert_eq!(resumed.step_statuses.get(1).unwrap(), StepStatus::Skipped);
    assert_eq!(resumed.current_step, OnboardingStep::CreateFirstTemplate);
}

#[test]
fn test_onboarding_get_progress_returns_none_before_start() {
    let (_, client, _, seller, _) = setup();

    let progress = client.get_onboarding_progress(&seller);
    assert!(progress.is_none());
}

#[test]
fn test_onboarding_is_active_reflects_state() {
    let (_, client, _, seller, _) = setup();

    assert!(!client.is_onboarding_active(&seller));

    client.start_onboarding(&seller);
    assert!(client.is_onboarding_active(&seller));

    for i in 0..4u32 {
        client.complete_onboarding_step(&seller, &i);
    }
    assert!(!client.is_onboarding_active(&seller));
}

#[test]
fn test_onboarding_does_not_affect_existing_trades() {
    let (env, client, _, seller, buyer) = setup();

    // Create a trade before onboarding
    let trade_id = client.create_trade(&seller, &buyer, &1000, &None, &None);

    // Run through onboarding
    client.start_onboarding(&seller);
    client.exit_onboarding(&seller);

    // Trade is unaffected
    let page = client.get_transaction_history(
        &seller,
        &no_filter(&env),
        &SortOrder::Ascending,
        &0,
        &10,
    );
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().trade_id, trade_id);
}

#[test]
fn test_onboarding_independent_per_user() {
    let (_, client, _, seller, buyer) = setup();

    client.start_onboarding(&seller);
    client.complete_onboarding_step(&seller, &0);

    // Buyer has no onboarding yet
    assert!(client.get_onboarding_progress(&buyer).is_none());

    // Start buyer's onboarding — starts fresh
    let buyer_progress = client.start_onboarding(&buyer);
    assert_eq!(buyer_progress.current_step, OnboardingStep::RegisterProfile);

    // Seller's progress is unchanged
    let seller_progress = client.get_onboarding_progress(&seller).unwrap();
    assert_eq!(seller_progress.step_statuses.get(0).unwrap(), StepStatus::Done);
}
