#![no_std]

mod errors;
mod events;
mod history;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};
use soroban_sdk::token::TokenClient;

pub use errors::ContractError;
pub use types::{DisputeResolution, HistoryFilter, HistoryPage, SortOrder, Trade, TradeStatus, TransactionRecord};

use storage::{
    get_accumulated_fees, get_admin, get_fee_bps, get_trade, get_usdc_token,
    has_arbitrator, increment_trade_counter, index_trade_for_address,
    is_initialized, is_paused, remove_arbitrator, save_arbitrator, save_trade,
    set_accumulated_fees, set_admin, set_fee_bps, set_initialized, set_paused,
    set_trade_counter, set_usdc_token,
};

/// Return ContractPaused if the contract is currently paused.
fn require_not_paused(env: &Env) -> Result<(), ContractError> {
    if is_paused(env) {
        return Err(ContractError::ContractPaused);
    }
    Ok(())
}

#[contract]
pub struct StellarEscrowContract;

#[contractimpl]
impl StellarEscrowContract {
    /// Initialize the contract with admin, USDC token address, and platform fee
    pub fn initialize(env: Env, admin: Address, usdc_token: Address, fee_bps: u32) -> Result<(), ContractError> {
        if is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        if fee_bps > 10000 {
            return Err(ContractError::InvalidFeeBps);
        }
        admin.require_auth();
        set_admin(&env, &admin);
        set_usdc_token(&env, &usdc_token);
        set_fee_bps(&env, fee_bps);
        set_trade_counter(&env, 0);
        set_accumulated_fees(&env, 0);
        set_initialized(&env);
        Ok(())
    }

    /// Register an arbitrator (admin only)
    pub fn register_arbitrator(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        save_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_registered(&env, arbitrator);
        Ok(())
    }

    /// Remove an arbitrator (admin only)
    pub fn remove_arbitrator_fn(env: Env, arbitrator: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        let admin = get_admin(&env)?;
        admin.require_auth();
        remove_arbitrator(&env, &arbitrator);
        events::emit_arbitrator_removed(&env, arbitrator);
        Ok(())
    }

    /// Update platform fee (admin only)
    pub fn update_fee(env: Env, fee_bps: u32) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        if fee_bps > 10000 {
            return Err(ContractError::InvalidFeeBps);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_fee_bps(&env, fee_bps);
        events::emit_fee_updated(&env, fee_bps);
        Ok(())
    }

    /// Withdraw accumulated fees (admin only)
    pub fn withdraw_fees(env: Env, to: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        let fees = get_accumulated_fees(&env)?;
        if fees == 0 {
            return Err(ContractError::NoFeesToWithdraw);
        }
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(&env.current_contract_address(), &to, &(fees as i128));
        set_accumulated_fees(&env, 0);
        events::emit_fees_withdrawn(&env, fees, to);
        Ok(())
    }

    /// Create a new trade
    pub fn create_trade(
        env: Env,
        seller: Address,
        buyer: Address,
        amount: u64,
        arbitrator: Option<Address>,
    ) -> Result<u64, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        if amount == 0 {
            return Err(ContractError::InvalidAmount);
        }
        seller.require_auth();
        if let Some(ref arb) = arbitrator {
            if !has_arbitrator(&env, arb) {
                return Err(ContractError::ArbitratorNotRegistered);
            }
        }
        let trade_id = increment_trade_counter(&env)?;
        let fee_bps = get_fee_bps(&env)?;
        let fee = amount
            .checked_mul(fee_bps as u64)
            .ok_or(ContractError::Overflow)?
            .checked_div(10000)
            .ok_or(ContractError::Overflow)?;

        let now = env.ledger().sequence();
        let trade = Trade {
            id: trade_id,
            seller: seller.clone(),
            buyer: buyer.clone(),
            amount,
            fee,
            arbitrator,
            status: TradeStatus::Created,
            created_at: now,
            updated_at: now,
        };

        save_trade(&env, trade_id, &trade);
        // Index trade for both parties so history lookups work for either address
        index_trade_for_address(&env, &seller, trade_id);
        index_trade_for_address(&env, &buyer, trade_id);
        events::emit_trade_created(&env, trade_id, seller, buyer, amount);
        Ok(trade_id)
    }

    /// Buyer funds the trade
    pub fn fund_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        token_client.transfer(
            &trade.buyer,
            &env.current_contract_address(),
            &(trade.amount as i128),
        );
        trade.status = TradeStatus::Funded;
        trade.updated_at = env.ledger().sequence();
        save_trade(&env, trade_id, &trade);
        events::emit_trade_funded(&env, trade_id);
        Ok(())
    }

    /// Seller marks trade as completed
    pub fn complete_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded {
            return Err(ContractError::InvalidStatus);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Completed;
        trade.updated_at = env.ledger().sequence();
        save_trade(&env, trade_id, &trade);
        events::emit_trade_completed(&env, trade_id);
        Ok(())
    }

    /// Buyer confirms receipt and releases funds
    pub fn confirm_receipt(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        let trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        trade.buyer.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        token_client.transfer(
            &env.current_contract_address(),
            &trade.seller,
            &(payout as i128),
        );
        let current_fees = get_accumulated_fees(&env)?;
        let new_fees = current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
        set_accumulated_fees(&env, new_fees);
        events::emit_trade_confirmed(&env, trade_id, payout, trade.fee);
        Ok(())
    }

    /// Raise a dispute
    pub fn raise_dispute(env: Env, trade_id: u64, caller: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Funded && trade.status != TradeStatus::Completed {
            return Err(ContractError::InvalidStatus);
        }
        if trade.arbitrator.is_none() {
            return Err(ContractError::ArbitratorNotRegistered);
        }
        if caller != trade.buyer && caller != trade.seller {
            return Err(ContractError::Unauthorized);
        }
        caller.require_auth();
        trade.status = TradeStatus::Disputed;
        trade.updated_at = env.ledger().sequence();
        save_trade(&env, trade_id, &trade);
        events::emit_dispute_raised(&env, trade_id, caller);
        Ok(())
    }

    /// Resolve a dispute (arbitrator only)
    pub fn resolve_dispute(
        env: Env,
        trade_id: u64,
        resolution: DisputeResolution,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        let trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Disputed {
            return Err(ContractError::InvalidStatus);
        }
        let arbitrator = trade.arbitrator.ok_or(ContractError::ArbitratorNotRegistered)?;
        arbitrator.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        let recipient = match resolution {
            DisputeResolution::ReleaseToBuyer => trade.buyer.clone(),
            DisputeResolution::ReleaseToSeller => trade.seller.clone(),
        };
        let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
        token_client.transfer(
            &env.current_contract_address(),
            &recipient,
            &(payout as i128),
        );
        let current_fees = get_accumulated_fees(&env)?;
        let new_fees = current_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
        set_accumulated_fees(&env, new_fees);
        events::emit_dispute_resolved(&env, trade_id, resolution, recipient);
        Ok(())
    }

    /// Cancel an unfunded trade
    pub fn cancel_trade(env: Env, trade_id: u64) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;
        let mut trade = get_trade(&env, trade_id)?;
        if trade.status != TradeStatus::Created {
            return Err(ContractError::InvalidStatus);
        }
        trade.seller.require_auth();
        trade.status = TradeStatus::Cancelled;
        trade.updated_at = env.ledger().sequence();
        save_trade(&env, trade_id, &trade);
        events::emit_trade_cancelled(&env, trade_id);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Query functions
    // -------------------------------------------------------------------------

    /// Get trade details
    pub fn get_trade(env: Env, trade_id: u64) -> Result<Trade, ContractError> {
        get_trade(&env, trade_id)
    }

    /// Get accumulated fees
    pub fn get_accumulated_fees(env: Env) -> Result<u64, ContractError> {
        get_accumulated_fees(&env)
    }

    /// Check if arbitrator is registered
    pub fn is_arbitrator_registered(env: Env, arbitrator: Address) -> bool {
        has_arbitrator(&env, &arbitrator)
    }

    /// Get platform fee in basis points
    pub fn get_platform_fee_bps(env: Env) -> Result<u32, ContractError> {
        get_fee_bps(&env)
    }

    // -------------------------------------------------------------------------
    // Emergency Pause
    // -------------------------------------------------------------------------

    /// Pause all contract operations (admin only).
    /// While paused, all state-mutating calls return ContractPaused.
    pub fn pause(env: Env) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_paused(&env, true);
        events::emit_paused(&env, admin);
        Ok(())
    }

    /// Unpause the contract, resuming normal operations (admin only).
    pub fn unpause(env: Env) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        set_paused(&env, false);
        events::emit_unpaused(&env, admin);
        Ok(())
    }

    /// Emergency withdrawal of all contract token balance to a destination
    /// address (admin only). Allowed even while paused so funds can always
    /// be recovered during an incident.
    pub fn emergency_withdraw(env: Env, to: Address) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        let admin = get_admin(&env)?;
        admin.require_auth();
        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        // Use the full contract balance, not just accumulated fees
        let balance = token_client.balance(&env.current_contract_address());
        if balance > 0 {
            token_client.transfer(&env.current_contract_address(), &to, &balance);
        }
        // Zero out accumulated fees to keep state consistent
        set_accumulated_fees(&env, 0);
        events::emit_emergency_withdraw(&env, to, balance as u64);
        Ok(())
    }

    /// Returns true if the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        is_paused(&env)
    }

    /// Batch create trades - optimized for multiple trades
    pub fn batch_create_trades(
        env: Env,
        seller: Address,
        trades: soroban_sdk::Vec<(Address, u64, Option<Address>)>,
    ) -> Result<soroban_sdk::Vec<u64>, ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        // Enforce batch size limits (max 100 trades per batch for gas optimization)
        if trades.is_empty() {
            return Err(ContractError::EmptyBatch);
        }
        if trades.len() > 100 {
            return Err(ContractError::BatchLimitExceeded);
        }

        seller.require_auth();

        let fee_bps = get_fee_bps(&env)?;
        let mut trade_ids = soroban_sdk::Vec::new(&env);
        let mut total_amount: u64 = 0;

        for (buyer, amount, arbitrator) in trades.iter() {
            if amount == 0 {
                return Err(ContractError::InvalidAmount);
            }

            if let Some(ref arb) = arbitrator {
                if !has_arbitrator(&env, arb) {
                    return Err(ContractError::ArbitratorNotRegistered);
                }
            }

            let trade_id = increment_trade_counter(&env)?;
            let fee = amount
                .checked_mul(fee_bps as u64)
                .ok_or(ContractError::Overflow)?
                .checked_div(10000)
                .ok_or(ContractError::Overflow)?;

            let trade = Trade {
                id: trade_id,
                seller: seller.clone(),
                buyer: buyer.clone(),
                amount,
                fee,
                arbitrator,
                status: TradeStatus::Created,
                created_at: env.ledger().sequence(),
                updated_at: env.ledger().sequence(),
            };

            save_trade(&env, trade_id, &trade);
            trade_ids.push_back(trade_id);
            total_amount = total_amount.checked_add(amount).ok_or(ContractError::Overflow)?;
        }

        // Emit single batch event instead of multiple individual events
        events::emit_batch_trades_created(&env, trade_ids.len() as u32, total_amount);

        Ok(trade_ids)
    }

    /// Batch fund trades - optimized for multiple funding operations
    pub fn batch_fund_trades(
        env: Env,
        buyer: Address,
        trade_ids: soroban_sdk::Vec<u64>,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        // Enforce batch size limits for gas optimization
        if trade_ids.is_empty() {
            return Err(ContractError::EmptyBatch);
        }
        if trade_ids.len() > 100 {
            return Err(ContractError::BatchLimitExceeded);
        }

        buyer.require_auth();

        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        let mut total_amount: u64 = 0;

        // First pass: validate all trades and calculate total amount
        for trade_id in trade_ids.iter() {
            let trade = get_trade(&env, trade_id)?;
            if trade.status != TradeStatus::Created {
                return Err(ContractError::InvalidStatus);
            }
            if trade.buyer != buyer {
                return Err(ContractError::Unauthorized);
            }
            total_amount = total_amount.checked_add(trade.amount).ok_or(ContractError::Overflow)?;
        }

        // Single transfer for all trades (gas optimization)
        token_client.transfer(&buyer, &env.current_contract_address(), &(total_amount as i128));

        // Second pass: update trade statuses
        for trade_id in trade_ids.iter() {
            let mut trade = get_trade(&env, trade_id)?;
            trade.status = TradeStatus::Funded;
            trade.updated_at = env.ledger().sequence();
            save_trade(&env, trade_id, &trade);
        }

        // Emit single batch event
        events::emit_batch_trades_funded(&env, trade_ids.len() as u32, total_amount);

        Ok(())
    }

    /// Batch confirm trades - optimized for multiple confirmations
    pub fn batch_confirm_trades(
        env: Env,
        buyer: Address,
        trade_ids: soroban_sdk::Vec<u64>,
    ) -> Result<(), ContractError> {
        if !is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }
        require_not_paused(&env)?;

        // Enforce batch size limits for gas optimization
        if trade_ids.is_empty() {
            return Err(ContractError::EmptyBatch);
        }
        if trade_ids.len() > 100 {
            return Err(ContractError::BatchLimitExceeded);
        }

        buyer.require_auth();

        let token = get_usdc_token(&env)?;
        let token_client = TokenClient::new(&env, &token);
        let mut total_payout: u64 = 0;
        let mut total_fees: u64 = 0;
        let mut seller_payouts: soroban_sdk::Map<Address, u64> = soroban_sdk::Map::new(&env);

        // First pass: validate all trades and calculate payouts
        for trade_id in trade_ids.iter() {
            let trade = get_trade(&env, trade_id)?;
            if trade.status != TradeStatus::Completed {
                return Err(ContractError::InvalidStatus);
            }
            if trade.buyer != buyer {
                return Err(ContractError::Unauthorized);
            }
            let payout = trade.amount.checked_sub(trade.fee).ok_or(ContractError::Overflow)?;
            total_payout = total_payout.checked_add(payout).ok_or(ContractError::Overflow)?;
            total_fees = total_fees.checked_add(trade.fee).ok_or(ContractError::Overflow)?;
            let current = seller_payouts.get(trade.seller.clone()).unwrap_or(0);
            let new_val = current.checked_add(payout).ok_or(ContractError::Overflow)?;
            seller_payouts.set(trade.seller.clone(), new_val);
        }

        // Transfer to each seller (grouped by seller for efficiency)
        for (seller, payout) in seller_payouts.iter() {
            token_client.transfer(&env.current_contract_address(), &seller, &(payout as i128));
        }

        // Update accumulated fees
        let current_fees = get_accumulated_fees(&env)?;
        let new_fees = current_fees.checked_add(total_fees).ok_or(ContractError::Overflow)?;
        set_accumulated_fees(&env, new_fees);

        // Emit single batch event
        events::emit_batch_trades_confirmed(&env, trade_ids.len() as u32, total_payout, total_fees);

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Transaction history (Issue #38)
    // -------------------------------------------------------------------------

    /// Return paginated, filtered, sorted transaction history for an address.
    pub fn get_transaction_history(
        env: Env,
        address: Address,
        filter: HistoryFilter,
        sort: SortOrder,
        offset: u32,
        limit: u32,
    ) -> Result<HistoryPage, ContractError> {
        history::get_history(&env, address, filter, sort, offset, limit)
    }

    /// Export transaction history for an address as a CSV string.
    /// Columns: trade_id,amount,fee,status,created_at,updated_at
    pub fn export_transaction_csv(
        env: Env,
        address: Address,
        filter: HistoryFilter,
    ) -> Result<soroban_sdk::String, ContractError> {
        history::export_csv(&env, address, filter)
    }
}