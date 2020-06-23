// Copyright (c) 2018-2020 MobileCoin Inc.

//! Utilities for Stellar Consensus Protocol tests.
use crate::{core_types::Value, slot::Slot, QuorumSet, QuorumSetMember, SlotIndex};
use mc_common::{logger::Logger, NodeID, ResponderId};
use mc_crypto_keys::Ed25519Pair;
use mc_util_from_random::FromRandom;
use rand::SeedableRng;
use rand_hc::Hc128Rng as FixedRng;
use std::{collections::BTreeSet, fmt, str::FromStr, sync::Arc};

use pest::Parser;
use pest_derive::Parser;
/// Helper for parsing quorum sets from string representations using "pest"
/// Used in crate tests.
#[doc="PEST Parser"]
#[derive(Parser)]
#[grammar = "quorum_set_parser.pest"]
struct QuorumSetParser;

/// Error for transaction validation
pub struct TransactionValidationError;
impl fmt::Display for TransactionValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("TransactionValidationError")
    }
}

/// Returns Ok.
pub fn trivial_validity_fn<T: Value>(_value: &T) -> Result<(), TransactionValidationError> {
    Ok(())
}

/// Returns `values`.
pub fn trivial_combine_fn<T: Value>(values: BTreeSet<T>) -> BTreeSet<T> {
    values
}

/// Returns at most the first `n` values.
#[allow(unused)]
pub fn get_bounded_combine_fn<V: Value>(
    max_elements: usize,
) -> impl Fn(BTreeSet<V>) -> BTreeSet<V> {
    move |values: BTreeSet<V>| -> BTreeSet<V> { values.into_iter().take(max_elements).collect() }
}

/// Creates NodeID from integer for testing.
pub fn test_node_id(node_id: u32) -> NodeID {
    let (node_id, _signer) = test_node_id_and_signer(node_id);
    node_id
}

/// Creates NodeID and Signer keypair from integer for testing.
pub fn test_node_id_and_signer(node_id: u32) -> (NodeID, Ed25519Pair) {
    let mut seed_bytes = [0u8; 32];
    let node_id_bytes = node_id.to_be_bytes();
    seed_bytes[..node_id_bytes.len()].copy_from_slice(&node_id_bytes[..]);

    let mut seeded_rng: FixedRng = SeedableRng::from_seed(seed_bytes);
    let signer_keypair = Ed25519Pair::from_random(&mut seeded_rng);
    (
        NodeID {
            responder_id: ResponderId::from_str(&format!("node{}.test.com:8443", node_id)).unwrap(),
            public_key: signer_keypair.public_key(),
        },
        signer_keypair,
    )
}

/// Recovers the u32 node_id value for a NodeID created using test_node_id_and_signer
pub fn recover_test_node_index(node_id: &NodeID) -> u32 {
    node_id
        .responder_id
        .0
        .split('.')
        .fuse()
        .next()
        .expect("unexpected responder_id")[4..]
        .parse::<u32>()
        .expect("unable to parse node index")
}

/// Creates a new slot.
pub fn get_slot(
    slot_index: SlotIndex,
    node_id: &NodeID,
    quorum_set: &QuorumSet,
    logger: Logger,
) -> Slot<u32, TransactionValidationError> {
    Slot::<u32, TransactionValidationError>::new(
        node_id.clone(),
        quorum_set.clone(),
        slot_index,
        Arc::new(trivial_validity_fn),
        Arc::new(trivial_combine_fn),
        logger,
    )
}

/// Three nodes that form a three-node cycle.
///
/// * Node 1 has the quorum slice {1,2}, where {2} is a blocking set.
/// * Node 2 has the quorum slice {2,3}, where {3} is a blocking set.
/// * Node 3 has the quorum slice {1,3}, where {1} is a blocking set.
/// * The only quorum is the set of all three nodes {1, 2, 3}.
pub fn three_node_cycle() -> (
    (NodeID, QuorumSet),
    (NodeID, QuorumSet),
    (NodeID, QuorumSet),
) {
    let node_1 = (
        test_node_id(1),
        QuorumSet::new_with_node_ids(1, vec![test_node_id(2)]),
    );
    let node_2 = (
        test_node_id(2),
        QuorumSet::new_with_node_ids(1, vec![test_node_id(3)]),
    );
    let node_3 = (
        test_node_id(3),
        QuorumSet::new_with_node_ids(1, vec![test_node_id(1)]),
    );
    (node_1, node_2, node_3)
}

/// The four-node network from Fig. 2 of the [Stellar Whitepaper](https://www.stellar.org/papers/stellar-consensus-protocol).
///
/// * Node 1 has the quorum slice {1,2,3}, where {2}, {3}, {2,3} are blocking sets.
/// * Nodes 2,3, and 4 have the quorum slice {2,3,4}.
/// * The only quorum is the set of all nodes {1,2,3,4}.
pub fn fig_2_network() -> (
    (NodeID, QuorumSet),
    (NodeID, QuorumSet),
    (NodeID, QuorumSet),
    (NodeID, QuorumSet),
) {
    let node_1 = (
        test_node_id(1),
        QuorumSet::new_with_node_ids(2, vec![test_node_id(2), test_node_id(3)]),
    );
    let node_2 = (
        test_node_id(2),
        QuorumSet::new_with_node_ids(2, vec![test_node_id(3), test_node_id(4)]),
    );
    let node_3 = (
        test_node_id(3),
        QuorumSet::new_with_node_ids(2, vec![test_node_id(2), test_node_id(4)]),
    );
    let node_4 = (
        test_node_id(4),
        QuorumSet::new_with_node_ids(2, vec![test_node_id(2), test_node_id(4)]),
    );

    (node_1, node_2, node_3, node_4)
}

/// A three-node network where the only quorum is the set of all three nodes.
/// Each node is a blocking set for each other.
pub fn three_node_dense_graph() -> (
    (NodeID, QuorumSet),
    (NodeID, QuorumSet),
    (NodeID, QuorumSet),
) {
    let node_1 = (
        test_node_id(1),
        QuorumSet::new_with_node_ids(2, vec![test_node_id(2), test_node_id(3)]),
    );
    let node_2 = (
        test_node_id(2),
        QuorumSet::new_with_node_ids(2, vec![test_node_id(1), test_node_id(3)]),
    );
    let node_3 = (
        test_node_id(3),
        QuorumSet::new_with_node_ids(2, vec![test_node_id(1), test_node_id(2)]),
    );
    (node_1, node_2, node_3)
}


///////////////////////////////////////////////////////////////////////////////
/// QuorumSet Parsing
///////////////////////////////////////////////////////////////////////////////


/// Generates a QuorumSet<NodeID> from a string using pest parser
pub fn test_quorum_set_from_string(
    quorum_set_string: &str,
) -> Result<QuorumSet<NodeID>, pest::error::Error<Rule>> {
    let inner_rules = QuorumSetParser::parse(Rule::quorum_set, quorum_set_string)?
        .next()
        .unwrap()
        .into_inner();
    let mut quorum_set: QuorumSet<NodeID> = QuorumSet::empty();
    for pair in inner_rules {
        match pair.as_rule() {
            Rule::empty_set => {
                return Ok(quorum_set);
            }
            Rule::threshold => {
                let threshold_string = pair.into_inner().next().unwrap().as_str();
                quorum_set.threshold = str::parse(threshold_string).unwrap();
            }
            Rule::members => {
                for member in pair.into_inner() {
                    match member.as_rule() {
                        Rule::node => {
                            let node: u32 = str::parse::<u32>(member.as_str()).unwrap();
                            let node_id = test_node_id(node);
                            quorum_set.members.push(QuorumSetMember::Node(node_id));
                        }
                        Rule::quorum_set => {
                            let inner_set = test_quorum_set_from_string(member.as_str())?;
                            quorum_set
                                .members
                                .push(QuorumSetMember::InnerSet(inner_set));
                        }
                        _ => panic!("unexpected rule!"),
                    }
                }
            }
            _ => panic!("unexpected rule!"),
        }
    }
    Ok(quorum_set)
}

/// creates a easy-to-read string from a QuorumSet<NodeID>
pub fn test_quorum_set_to_string(quorum_set: &QuorumSet<NodeID>) -> String {
    let mut quorum_set_string = format!("([{}]", quorum_set.threshold);
    for member in quorum_set.members.iter() {
        match member {
            QuorumSetMember::Node(node_id) => {
                quorum_set_string.push_str(&format!(
                    ",{}",
                    recover_test_node_index(node_id)
                ));
            }
            QuorumSetMember::InnerSet(inner_set) => {
                quorum_set_string.push(',');
                quorum_set_string.push_str(&test_quorum_set_to_string(inner_set));
            }
        }
    }
    quorum_set_string.push(')');
    quorum_set_string
}

#[cfg(test)]
mod quorum_set_parser_tests {
    use super::*;

    #[test]
    fn test_quorum_set_construction() {
        let qs_string = "([3],1,2,3,4,([2],5,6,([1],7,8)))".to_owned();
        let qs = test_quorum_set_from_string(&qs_string).expect("failed to parse");
        let qs_new_string = test_quorum_set_to_string(&qs);
        assert_eq!(qs_string, qs_new_string);
    }

    #[test]
    #[should_panic]
    fn test_quorum_set_parser_fails() {
        let bad_qs_string = "([3],1, [5], 2,3, 4,([2],5, 6,([1],8,7)))".to_owned();
        let _qs = test_quorum_set_from_string(&bad_qs_string).expect("failed to parse");
    }
}

