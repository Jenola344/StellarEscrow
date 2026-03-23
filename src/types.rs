use soroban_sdk::{contracttype, Address, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToBuyer,
    ReleaseToSeller,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trade {
    pub id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub arbitrator: Option<Address>,
    pub status: TradeStatus,
    /// Ledger sequence number when the trade was created
    pub created_at: u32,
    /// Ledger sequence number of the last status update
    pub updated_at: u32,
}

/// A richer view of a trade used for history queries
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionRecord {
    pub trade_id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub status: TradeStatus,
    pub created_at: u32,
    pub updated_at: u32,
}

/// Filter options for history queries
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryFilter {
    /// Optional status filter
    pub status: Option<TradeStatus>,
    /// Minimum ledger sequence (inclusive)
    pub from_ledger: Option<u32>,
    /// Maximum ledger sequence (inclusive)
    pub to_ledger: Option<u32>,
}

/// Paginated history result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryPage {
    pub records: Vec<TransactionRecord>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

// =============================================================================
// User Management (Issue #64)
// =============================================================================

/// Verification status of a user, set by admin
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Unverified,
    Pending,
    Verified,
    Rejected,
}

/// On-chain user profile
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserProfile {
    pub address: Address,
    /// SHA-256 hash of off-chain display name (stored as 32 bytes)
    pub username_hash: soroban_sdk::Bytes,
    /// SHA-256 hash of off-chain contact info
    pub contact_hash: soroban_sdk::Bytes,
    pub verification: VerificationStatus,
    pub registered_at: u32,
    pub updated_at: u32,
}

/// Per-user preferences stored as a key→value map entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserPreference {
    pub key: soroban_sdk::String,
    pub value: soroban_sdk::String,
}

/// Aggregated analytics for a user
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserAnalytics {
    pub address: Address,
    pub total_trades: u32,
    pub trades_as_seller: u32,
    pub trades_as_buyer: u32,
    pub total_volume: u64,
    pub completed_trades: u32,
    pub disputed_trades: u32,
    pub cancelled_trades: u32,
}
