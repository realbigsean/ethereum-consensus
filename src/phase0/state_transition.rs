use crate::phase0 as spec;

use crate::state_transition::{Context, Error, Validation};
use spec::{process_block, process_slots, verify_block_signature, BeaconState, SignedBeaconBlock};
use ssz_rs::prelude::*;

pub fn state_transition<
    const SLOTS_PER_HISTORICAL_ROOT: usize,
    const HISTORICAL_ROOTS_LIMIT: usize,
    const ETH1_DATA_VOTES_BOUND: usize,
    const VALIDATOR_REGISTRY_LIMIT: usize,
    const EPOCHS_PER_HISTORICAL_VECTOR: usize,
    const EPOCHS_PER_SLASHINGS_VECTOR: usize,
    const MAX_VALIDATORS_PER_COMMITTEE: usize,
    const PENDING_ATTESTATIONS_BOUND: usize,
    const MAX_PROPOSER_SLASHINGS: usize,
    const MAX_ATTESTER_SLASHINGS: usize,
    const MAX_ATTESTATIONS: usize,
    const MAX_DEPOSITS: usize,
    const MAX_VOLUNTARY_EXITS: usize,
>(
    state: &mut BeaconState<
        SLOTS_PER_HISTORICAL_ROOT,
        HISTORICAL_ROOTS_LIMIT,
        ETH1_DATA_VOTES_BOUND,
        VALIDATOR_REGISTRY_LIMIT,
        EPOCHS_PER_HISTORICAL_VECTOR,
        EPOCHS_PER_SLASHINGS_VECTOR,
        MAX_VALIDATORS_PER_COMMITTEE,
        PENDING_ATTESTATIONS_BOUND,
    >,
    signed_block: &mut SignedBeaconBlock<
        MAX_PROPOSER_SLASHINGS,
        MAX_VALIDATORS_PER_COMMITTEE,
        MAX_ATTESTER_SLASHINGS,
        MAX_ATTESTATIONS,
        MAX_DEPOSITS,
        MAX_VOLUNTARY_EXITS,
    >,
    validation: Validation,
    context: &Context,
) -> Result<(), Error> {
    let validate_result = match validation {
        Validation::Enabled => true,
        Validation::Disabled => false,
    };

    let slot = signed_block.message.slot;

    process_slots(state, slot, context)?;
    if validate_result {
        verify_block_signature(state, signed_block, context)?;
    }
    let block = &mut signed_block.message;
    process_block(state, block, context)?;
    if validate_result && block.state_root != state.hash_tree_root()? {
        return Err(Error::InvalidStateRoot);
    }

    Ok(())
}
