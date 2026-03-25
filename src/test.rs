#![cfg(test)]

extern crate std;

use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

use crate::{
    ContractError, Currency, FundingPreview, HistoryFilter, OptionalTradeMetadata,
    OptionalTradeStatus, SortOrder, StellarEscrowContract, StellarEscrowContractClient,
    TradeFormInput, TradePreview, TradeStatus,
};

fn setup() -> (Env, StellarEscrowContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarEscrowContract);
    let client = StellarEscrowContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let token = Address::generate(&env);
    client.initialize(&admin, &token, &100); // 1% fee
    (env, client, admin, seller, buyer)
}

fn no_filter() -> HistoryFilter {
    HistoryFilter { status: OptionalTradeStatus::None, from_ledger: None, to_ledger: None }
}

fn no_meta() -> OptionalTradeMetadata { OptionalTradeMetadata::None }

fn make_form_input(seller: &Address, buyer: &Address, amount: u64, arb: Option<Address>) -> TradeFormInput {
    TradeFormInput { seller: seller.clone(), buyer: buyer.clone(), amount, currency: Currency::Usdc, arbitrator: arb }
}

fn create_trade(client: &StellarEscrowContractClient, seller: &Address, buyer: &Address, amount: u64) -> u64 {
    client.create_trade(seller, buyer, &amount, &None, &no_meta())
}


// ---------------------------------------------------------------------------
// History tests
// ---------------------------------------------------------------------------

#[test]
fn test_history_empty_for_new_address() {
    let (_env, client, _, seller, _) = setup();
    let page = client.get_transaction_history(&seller, &no_filter(), &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 0);
    assert_eq!(page.records.len(), 0);
}

#[test]
fn test_history_shows_created_trade() {
    let (_env, client, _, seller, buyer) = setup();
    let trade_id = create_trade(&client, &seller, &buyer, 1000);
    let page = client.get_transaction_history(&seller, &no_filter(), &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    let record = page.records.get(0).unwrap();
    assert_eq!(record.trade_id, trade_id);
    assert_eq!(record.amount, 1000);
    assert_eq!(record.status, TradeStatus::Created);
}

#[test]
fn test_history_visible_from_buyer_address() {
    let (_env, client, _, seller, buyer) = setup();
    create_trade(&client, &seller, &buyer, 500);
    let page = client.get_transaction_history(&buyer, &no_filter(), &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
}

#[test]
fn test_history_filter_by_status() {
    let (_env, client, _, seller, buyer) = setup();
    create_trade(&client, &seller, &buyer, 1000);
    create_trade(&client, &seller, &buyer, 2000);
    client.cancel_trade(&1);
    let filter = HistoryFilter {
        status: OptionalTradeStatus::Some(TradeStatus::Cancelled),
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
    env.ledger().set_sequence_number(1);
    create_trade(&client, &seller, &buyer, 1000);
    env.ledger().set_sequence_number(100);
    create_trade(&client, &seller, &buyer, 2000);
    let filter = HistoryFilter { status: OptionalTradeStatus::None, from_ledger: Some(50), to_ledger: Some(200) };
    let page = client.get_transaction_history(&seller, &filter, &SortOrder::Ascending, &0, &10);
    assert_eq!(page.total, 1);
    assert_eq!(page.records.get(0).unwrap().amount, 2000);
}

#[test]
fn test_history_sort_descending() {
    let (env, client, _, seller, buyer) = setup();
    env.ledger().set_sequence_number(1);
    create_trade(&client, &seller, &buyer, 100);
    env.ledger().set_sequence_number(10);
    create_trade(&client, &seller, &buyer, 200);
    let page = client.get_transaction_history(&seller, &no_filter(), &SortOrder::Descending, &0, &10);
    assert_eq!(page.records.get(0).unwrap().amount, 200);
    assert_eq!(page.records.get(1).unwrap().amount, 100);
}

#[test]
fn test_history_pagination() {
    let (_env, client, _, seller, buyer) = setup();
    for _ in 0..5 { create_trade(&client, &seller, &buyer, 1000); }
    let page1 = client.get_transaction_history(&seller, &no_filter(), &SortOrder::Ascending, &0, &3);
    assert_eq!(page1.records.len(), 3);
    assert_eq!(page1.total, 5);
    let page2 = client.get_transaction_history(&seller, &no_filter(), &SortOrder::Ascending, &3, &3);
    assert_eq!(page2.records.len(), 2);
}

#[test]
fn test_export_csv_returns_header_and_rows() {
    let (_env, client, _, seller, buyer) = setup();
    create_trade(&client, &seller, &buyer, 1000);
    let csv = client.export_transaction_csv(
        &seller,
        &HistoryFilter { status: OptionalTradeStatus::None, from_ledger: None, to_ledger: None },
    );
    assert!(csv.len() > 0);
}

// ---------------------------------------------------------------------------
// Trade creation form tests
// ---------------------------------------------------------------------------

#[test]
fn test_form_validate_valid_input() {
    let (_env, client, _, seller, buyer) = setup();
    client.validate_trade_form(&make_form_input(&seller, &buyer, 1_000_000, None));
}

#[test]
fn test_form_validate_zero_amount() {
    let (_env, client, _, seller, buyer) = setup();
    let result = client.try_validate_trade_form(&make_form_input(&seller, &buyer, 0, None));
    assert_eq!(result, Err(Ok(ContractError::InvalidAmount)));
}

#[test]
fn test_form_validate_buyer_seller_same() {
    let (_env, client, _, seller, _) = setup();
    let result = client.try_validate_trade_form(&make_form_input(&seller, &seller, 1_000_000, None));
    assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_form_validate_unregistered_arbitrator() {
    let (env, client, _, seller, buyer) = setup();
    let arb = Address::generate(&env);
    let result = client.try_validate_trade_form(&make_form_input(&seller, &buyer, 1_000_000, Some(arb)));
    assert_eq!(result, Err(Ok(ContractError::ArbitratorNotRegistered)));
}

#[test]
fn test_form_validate_registered_arbitrator_ok() {
    let (env, client, _, seller, buyer) = setup();
    let arb = Address::generate(&env);
    client.register_arbitrator(&arb);
    client.validate_trade_form(&make_form_input(&seller, &buyer, 1_000_000, Some(arb)));
}

#[test]
fn test_form_preview_returns_correct_fields() {
    let (_env, client, _, seller, buyer) = setup();
    let input = make_form_input(&seller, &buyer, 1_000_000, None);
    let preview = client.preview_trade(&input);
    assert_eq!(preview.seller, seller);
    assert_eq!(preview.buyer, buyer);
    assert_eq!(preview.amount, 1_000_000);
    assert_eq!(preview.currency, Currency::Usdc);
    assert_eq!(preview.arbitrator, None);
    assert_eq!(preview.estimated_fee, 10_000);
}

#[test]
fn test_form_confirm_creates_trade() {
    let (_env, client, _, seller, buyer) = setup();
    let input = make_form_input(&seller, &buyer, 500_000, None);
    let preview = client.preview_trade(&input);
    let trade_id = client.confirm_trade_form(&input, &preview);
    let trade = client.get_trade(&trade_id);
    assert_eq!(trade.seller, seller);
    assert_eq!(trade.buyer, buyer);
    assert_eq!(trade.amount, 500_000);
    assert_eq!(trade.status, TradeStatus::Created);
}

#[test]
fn test_form_confirm_rejects_mismatched_preview() {
    let (_env, client, _, seller, buyer) = setup();
    let input = make_form_input(&seller, &buyer, 500_000, None);
    let preview = client.preview_trade(&input);
    let bad = TradePreview {
        seller: preview.seller.clone(), buyer: preview.buyer.clone(),
        amount: 999_999, currency: preview.currency.clone(),
        arbitrator: preview.arbitrator.clone(), estimated_fee: preview.estimated_fee,
    };
    assert!(client.try_confirm_trade_form(&input, &bad).is_err());
}

#[test]
fn test_form_confirm_rejects_zero_amount() {
    let (_env, client, _, seller, buyer) = setup();
    let input = make_form_input(&seller, &buyer, 0, None);
    let fake = TradePreview {
        seller: seller.clone(), buyer: buyer.clone(), amount: 0,
        currency: Currency::Usdc, arbitrator: None, estimated_fee: 0,
    };
    assert!(client.try_confirm_trade_form(&input, &fake).is_err());
}

#[test]
fn test_form_confirm_rejects_same_buyer_seller() {
    let (_env, client, _, seller, _) = setup();
    let input = make_form_input(&seller, &seller, 1_000_000, None);
    let fake = TradePreview {
        seller: seller.clone(), buyer: seller.clone(), amount: 1_000_000,
        currency: Currency::Usdc, arbitrator: None, estimated_fee: 10_000,
    };
    assert!(client.try_confirm_trade_form(&input, &fake).is_err());
}

// ---------------------------------------------------------------------------
// Trade funding flow tests
// ---------------------------------------------------------------------------

#[test]
fn test_funding_preview_returns_correct_fields() {
    let (_env, client, _, seller, buyer) = setup();
    let trade_id = create_trade(&client, &seller, &buyer, 1_000_000);
    let preview = client.get_funding_preview(&trade_id, &buyer);
    assert_eq!(preview.trade_id, trade_id);
    assert_eq!(preview.buyer, buyer);
    assert_eq!(preview.seller, seller);
    assert_eq!(preview.amount, 1_000_000);
    assert_eq!(preview.fee, 10_000); // 1%
    assert!(!preview.allowance_sufficient); // mock env: allowance defaults to 0
}

#[test]
fn test_funding_preview_rejects_wrong_buyer() {
    let (env, client, _, seller, buyer) = setup();
    let trade_id = create_trade(&client, &seller, &buyer, 500_000);
    let stranger = Address::generate(&env);
    let result = client.try_get_funding_preview(&trade_id, &stranger);
    assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_funding_preview_rejects_non_created_trade() {
    let (_env, client, _, seller, buyer) = setup();
    let trade_id = create_trade(&client, &seller, &buyer, 500_000);
    client.cancel_trade(&trade_id);
    let result = client.try_get_funding_preview(&trade_id, &buyer);
    assert_eq!(result, Err(Ok(ContractError::InvalidStatus)));
}

#[test]
fn test_fund_trade_with_preview_succeeds() {
    let (_env, client, _, seller, buyer) = setup();
    let trade_id = create_trade(&client, &seller, &buyer, 1_000_000);
    let preview = client.get_funding_preview(&trade_id, &buyer);
    // mock_all_auths bypasses token allowance/transfer checks
    client.fund_trade_with_preview(&trade_id, &buyer, &preview);
    let trade = client.get_trade(&trade_id);
    assert_eq!(trade.status, TradeStatus::Funded);
}

#[test]
fn test_fund_trade_rejects_mismatched_preview() {
    let (_env, client, _, seller, buyer) = setup();
    let trade_id = create_trade(&client, &seller, &buyer, 1_000_000);
    let preview = client.get_funding_preview(&trade_id, &buyer);
    let bad = FundingPreview {
        trade_id: preview.trade_id, buyer: preview.buyer.clone(),
        seller: preview.seller.clone(), amount: 999_999, // tampered
        fee: preview.fee, buyer_balance: preview.buyer_balance,
        allowance_sufficient: preview.allowance_sufficient,
    };
    assert!(client.try_fund_trade_with_preview(&trade_id, &buyer, &bad).is_err());
}

#[test]
fn test_fund_trade_rejects_already_funded() {
    let (_env, client, _, seller, buyer) = setup();
    let trade_id = create_trade(&client, &seller, &buyer, 1_000_000);
    let preview = client.get_funding_preview(&trade_id, &buyer);
    client.fund_trade_with_preview(&trade_id, &buyer, &preview);
    // Second attempt: trade is now Funded, not Created
    let result = client.try_fund_trade_with_preview(&trade_id, &buyer, &preview);
    assert_eq!(result, Err(Ok(ContractError::InvalidStatus)));
}

#[test]
fn test_fund_trade_rejects_wrong_buyer() {
    let (env, client, _, seller, buyer) = setup();
    let trade_id = create_trade(&client, &seller, &buyer, 500_000);
    let stranger = Address::generate(&env);
    let bad = FundingPreview {
        trade_id, buyer: stranger.clone(), seller: seller.clone(),
        amount: 500_000, fee: 5_000, buyer_balance: 0, allowance_sufficient: false,
    };
    let result = client.try_fund_trade_with_preview(&trade_id, &stranger, &bad);
    assert_eq!(result, Err(Ok(ContractError::Unauthorized)));
}

#[test]
fn test_fund_trade_rejects_cancelled_trade() {
    let (_env, client, _, seller, buyer) = setup();
    let trade_id = create_trade(&client, &seller, &buyer, 500_000);
    let preview = client.get_funding_preview(&trade_id, &buyer);
    client.cancel_trade(&trade_id);
    let result = client.try_fund_trade_with_preview(&trade_id, &buyer, &preview);
    assert_eq!(result, Err(Ok(ContractError::InvalidStatus)));
}
