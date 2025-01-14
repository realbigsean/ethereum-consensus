use crate::altair;
use crate::bellatrix;
use crate::bellatrix::presets::Preset;

// use all items previously defined in the prior fork:
pub use altair::mainnet::*;
pub use bellatrix::*;

pub const INACTIVITY_PENALTY_QUOTIENT_BELLATRIX: u64 = 16777216;
pub const MIN_SLASHING_PENALTY_QUOTIENT_BELLATRIX: u64 = 32;
pub const PROPORTIONAL_SLASHING_MULTIPLIER_BELLATRIX: u64 = 3;
pub const MAX_BYTES_PER_TRANSACTION: usize = 1073741824;
pub const MAX_TRANSACTIONS_PER_PAYLOAD: usize = 1048576;
pub const BYTES_PER_LOGS_BLOOM: usize = 256;
pub const MAX_EXTRA_DATA_BYTES: usize = 32;

pub const PRESET: Preset = Preset {
    inactivity_penalty_quotient_bellatrix: INACTIVITY_PENALTY_QUOTIENT_BELLATRIX,
    min_slashing_penalty_quotient_bellatrix: MIN_SLASHING_PENALTY_QUOTIENT_BELLATRIX,
    proportional_slashing_multiplier_bellatrix: PROPORTIONAL_SLASHING_MULTIPLIER_BELLATRIX,
    max_bytes_per_transaction: MAX_BYTES_PER_TRANSACTION,
    max_transactions_per_payload: MAX_TRANSACTIONS_PER_PAYLOAD,
    bytes_per_logs_bloom: BYTES_PER_LOGS_BLOOM,
    max_extra_data_bytes: MAX_EXTRA_DATA_BYTES,
};

pub type Transaction = bellatrix::Transaction<MAX_BYTES_PER_TRANSACTION>;

pub type ExecutionPayload = bellatrix::ExecutionPayload<
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
>;

pub type ExecutionPayloadHeader = bellatrix::ExecutionPayloadHeader<
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
>;

pub type BlindedBeaconBlock = bellatrix::BlindedBeaconBlock<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
    SYNC_COMMITTEE_SIZE,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
>;

pub type BlindedBeaconBlockBody = bellatrix::BlindedBeaconBlockBody<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
    SYNC_COMMITTEE_SIZE,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
>;

pub type SignedBlindedBeaconBlock = bellatrix::SignedBlindedBeaconBlock<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
    SYNC_COMMITTEE_SIZE,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
>;
