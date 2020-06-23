// Copyright (c) 2018-2020 MobileCoin Inc.

//! Predicates for use in trust decisions for SCP.
use mc_common::{HashMap, HashSet, NodeID};
use std::{collections::BTreeSet, sync::Arc};

use crate::{
    core_types::{Ballot, GenericNodeId, Value},
    msg::Msg,
};

/// An interface for predicates, used for performing searches for quorums and blocking sets.
/// See `findQuorum`, `findBlockingSet`.
pub trait Predicate<V: Value, ID: GenericNodeId = NodeID>: Clone {
    /// The type of result this predicates could return.
    type Result;

    /// Tests whether the predicate is true for a given message.
    /// Retruns Some(Predicate) if `msg` satisfies the predicate, `None` otherwise.
    /// This allows the predicate to evolve it's state as it is called on more and more messages.
    fn test(&self, msg: &Msg<V, ID>) -> Option<Self>;

    /// Returns the result stored inside the predicate.
    fn result(&self) -> Self::Result;
}

/// A predicate for narrowing down a set of ballots.
#[derive(Clone)]
pub struct BallotSetPredicate<V: Value> {
    /// The ballots to consider for the evaluation of this predicate.
    pub ballots: HashSet<Ballot<V>>,

    /// The test function to apply to the ballots in this predicate.
    pub test_fn: Arc<dyn Fn(&Msg<V>, &HashSet<Ballot<V>>) -> HashSet<Ballot<V>>>,
}

impl<V: Value> Predicate<V> for BallotSetPredicate<V> {
    type Result = HashSet<Ballot<V>>;

    fn test(&self, msg: &Msg<V>) -> Option<Self> {
        if self.ballots.is_empty() {
            return None;
        }

        let nextBallots = (self.test_fn)(msg, &self.ballots);
        if nextBallots.is_empty() {
            return None;
        }

        Some(Self {
            ballots: nextBallots,
            test_fn: self.test_fn.clone(),
        })
    }

    fn result(&self) -> Self::Result {
        self.ballots.clone()
    }
}

/// A predicate for ranges of ballots, where the range is over the counter.
#[derive(Clone)]
pub struct BallotRangePredicate<V: Value> {
    /// Map of value to counter ranges, representing ballot ranges.
    pub ballot_ranges: HashMap<Vec<V>, (u32, u32)>,

    /// The test function to apply to the ballot ranges in this predicate.
    pub test_fn: Arc<dyn Fn(&Msg<V>, &HashMap<Vec<V>, (u32, u32)>) -> HashMap<Vec<V>, (u32, u32)>>,
}

impl<V: Value> Predicate<V> for BallotRangePredicate<V> {
    type Result = HashMap<Vec<V>, (u32, u32)>;

    fn test(&self, msg: &Msg<V>) -> Option<Self> {
        if self.ballot_ranges.is_empty() {
            return None;
        }

        let ballot_ranges = (self.test_fn)(msg, &self.ballot_ranges);
        if ballot_ranges.is_empty() {
            return None;
        }

        Some(Self {
            ballot_ranges,
            test_fn: self.test_fn.clone(),
        })
    }

    fn result(&self) -> Self::Result {
        self.ballot_ranges.clone()
    }
}

/// A predicate for narrowing down a set of values.
#[derive(Clone)]
pub struct ValueSetPredicate<V: Value> {
    /// The values over which to apply the test function.
    pub values: BTreeSet<V>,

    /// The test function to narrow down the values in this predicate.
    pub test_fn: Arc<dyn Fn(&Msg<V>, &BTreeSet<V>) -> BTreeSet<V>>,
}

impl<V: Value> Predicate<V> for ValueSetPredicate<V> {
    type Result = BTreeSet<V>;

    fn test(&self, msg: &Msg<V>) -> Option<Self> {
        if self.values.is_empty() {
            return None;
        }

        let next_values = (self.test_fn)(msg, &self.values);
        if next_values.is_empty() {
            return None;
        }

        Some(Self {
            values: next_values,
            test_fn: self.test_fn.clone(),
        })
    }

    fn result(&self) -> Self::Result {
        self.values.clone()
    }
}

impl<V: Value> ValueSetPredicate<V> {
    /// Given a list of results, each containg a set of values, find the "biggest" set of values.
    /// Sets of values are sorted by their length, and if the lenght matches then by their values.
    pub fn filter_to_max_values(
        results: Vec<(HashSet<NodeID>, BTreeSet<V>)>,
    ) -> Option<BTreeSet<V>> {
        if results.is_empty() {
            return None;
        }

        let (_node_ids, max_values) = results
            .into_iter()
            .max_by(|a, b| {
                let (a_node_ids, a_values) = a;
                let (b_node_ids, b_values) = b;
                if a_node_ids.len() != b_node_ids.len() {
                    a_node_ids.len().cmp(&b_node_ids.len())
                } else {
                    a_values.cmp(&b_values)
                }
            })
            .unwrap();

        Some(max_values)
    }
}

/// A predicate for narrowing down (min, max) ranges. Works in conjunction with
/// `Msg.accepts_commits()` and `Msg.votes_or_accepts_commits()`.
#[derive(Clone)]
pub struct MinMaxPredicate<V: Value> {
    /// The min value which will be tested in this predicate.
    pub min: u32,

    /// The max value which will be tested in this predicate.
    pub max: u32,

    /// The values over which to apply this predicate.
    pub values: Vec<V>,

    /// The test function to use for narrowing down the values with this predicate.
    pub test_fn: Arc<dyn Fn(&Msg<V>, &Vec<V>, u32, u32) -> Option<(u32, u32)>>,
}

impl<V: Value> Predicate<V> for MinMaxPredicate<V> {
    // (min, max)
    type Result = (u32, u32);

    fn test(&self, msg: &Msg<V>) -> Option<Self> {
        if self.min > self.max {
            return None;
        }
        match (self.test_fn)(msg, &self.values, self.min, self.max) {
            None => None,
            Some((min, max)) => Some(Self {
                min,
                max,
                values: self.values.clone(),
                test_fn: self.test_fn.clone(),
            }),
        }
    }
    fn result(&self) -> Self::Result {
        (self.min, self.max)
    }
}

impl<V: Value> MinMaxPredicate<V> {
    /// Given a list of (min, max) ranges, find the highest (min, max) range.
    /// The logic behind what is the "highest" (min, max) is chosen arbitrarily.
    /// In theory there could be multiple quorums with different (min, max)
    /// accepted-committed values, and we need a way to pick one of the
    /// possible ranges consistently between nodes.
    pub fn get_highest_ballot(results: Vec<(HashSet<NodeID>, (u32, u32))>) -> Option<(u32, u32)> {
        results
            .into_iter()
            .map(|(_node_ids, min_max)| min_max)
            .max()
    }
}

/// A predicate for determining whether a message matches a certain condition.
#[derive(Clone)]
pub struct FuncPredicate<'a, V: Value, ID: GenericNodeId = NodeID> {
    /// The test function to apply for this predicate.
    pub test_fn: &'a dyn Fn(&Msg<V, ID>) -> bool,
}

impl<'a, V: Value, ID: GenericNodeId> Predicate<V, ID> for FuncPredicate<'a, V, ID> {
    type Result = ();

    fn test(&self, msg: &Msg<V, ID>) -> Option<Self> {
        if (self.test_fn)(msg) {
            Some(self.clone())
        } else {
            None
        }
    }

    fn result(&self) -> Self::Result {}
}

#[cfg(test)]
mod predicates_tests {
    use super::*;
    use crate::{core_types::*, msg::*, test_utils::{test_node_id, quorum_set_from_str}};
    use mc_common::HashMap;
    use std::iter::FromIterator;

    #[test]
    // BallotSetPredicate can be used to pick a quorum that intersects with a given set of ballots.
    pub fn test_ballot_set_predicate_quorum() {
        let local_node_id = test_node_id(1);
        let local_node_quorum_set = quorum_set_from_str("([2],2,3,4,5)");
        let node_2_quorum_set = quorum_set_from_str("([1],1,3,4,5)");
        let node_3_quorum_set = quorum_set_from_str("([1],1,2,4,5)");
        let node_4_quorum_set = quorum_set_from_str("([1],1,2,3,5)");
        let node_5_quorum_set = quorum_set_from_str("([1],1,2,3,4)");

        let ballot_1 = Ballot::new(1, &[1111]);
        let ballot_2 = Ballot::new(1, &[2222]);
        let ballot_3 = Ballot::new(1, &[3333]);

        let mut msgs = HashMap::<NodeID, Msg<u32>>::default();

        // Node 2 and 3 form a quorum, voting on ballot_1
        let topic = Topic::Prepare(PreparePayload::<u32> {
            B: ballot_1.clone(),
            P: None,
            PP: None,
            CN: 0,
            HN: 0,
        });
        msgs.insert(
            test_node_id(2),
            Msg::new(test_node_id(2), node_2_quorum_set, 1, topic.clone()),
        );
        msgs.insert(
            test_node_id(3),
            Msg::new(test_node_id(3), node_3_quorum_set, 1, topic),
        );

        // Node 4 and 5 also form a quorum, voting on ballot_2
        let topic = Topic::Prepare(PreparePayload::<u32> {
            B: ballot_2,
            P: None,
            PP: None,
            CN: 0,
            HN: 0,
        });
        msgs.insert(
            test_node_id(4),
            Msg::new(test_node_id(4), node_4_quorum_set, 1, topic.clone()),
        );
        msgs.insert(
            test_node_id(5),
            Msg::new(test_node_id(5), node_5_quorum_set, 1, topic),
        );

        // Look for quorum intersecting with ballot_1 and some ballot for which there is no quorum
        let (node_ids, pred) = local_node_quorum_set.findQuorum(
            &local_node_id,
            &msgs,
            BallotSetPredicate {
                ballots: HashSet::from_iter(vec![ballot_1.clone(), ballot_3]),
                test_fn: Arc::new(|msg, ballots| {
                    ballots
                        .intersection(&msg.votes_or_accepts_prepared())
                        .cloned()
                        .collect()
                }),
            },
        );
        assert_eq!(
            node_ids,
            HashSet::from_iter(vec![test_node_id(1), test_node_id(2), test_node_id(3)])
        );
        assert_eq!(pred.result(), HashSet::from_iter(vec![ballot_1]));
    }

    #[test]
    // BallotSetPredicate can be used to pick a blocking set that intersects with a given set of ballots.
    pub fn test_ballot_set_predicate_blocking_set() {
        // Node 2 and 3 form a blocking set. Node 5 and 6 also form a blocking set.
        let local_node_quorum_set = quorum_set_from_str("([2],([2],2,3,4),([2],5,6,7))");
        let node_2_quorum_set = quorum_set_from_str("([1],3,4)");
        let node_3_quorum_set = quorum_set_from_str("([1],2,4)");
        let node_5_quorum_set = quorum_set_from_str("([1],6,7)");
        let node_6_quorum_set = quorum_set_from_str("([1],5,7)");

        let ballot_1 = Ballot::new(1, &[1111]);
        let ballot_2 = Ballot::new(1, &[2222]);
        let ballot_3 = Ballot::new(1, &[3333]);

        let mut msgs = HashMap::<NodeID, Msg<u32>>::default();

        // Node 2 and 3 form a blocking set , voting on ballot_1
        let topic = Topic::Prepare(PreparePayload::<u32> {
            B: ballot_1.clone(),
            P: None,
            PP: None,
            CN: 0,
            HN: 0,
        });
        msgs.insert(
            test_node_id(2),
            Msg::new(test_node_id(2), node_2_quorum_set, 1, topic.clone()),
        );
        msgs.insert(
            test_node_id(3),
            Msg::new(test_node_id(3), node_3_quorum_set, 1, topic),
        );

        // Node 5 and 6 also form a blocking set, voting on ballot_2
        let topic = Topic::Prepare(PreparePayload::<u32> {
            B: ballot_2,
            P: None,
            PP: None,
            CN: 0,
            HN: 0,
        });
        msgs.insert(
            test_node_id(5),
            Msg::new(test_node_id(5), node_5_quorum_set, 1, topic.clone()),
        );
        msgs.insert(
            test_node_id(6),
            Msg::new(test_node_id(6), node_6_quorum_set, 1, topic),
        );

        // Look for quorum intersecting with ballot_1 and some ballot for which there is no quorum
        let (node_ids, pred) = local_node_quorum_set.findBlockingSet(
            &msgs,
            BallotSetPredicate {
                ballots: HashSet::from_iter(vec![ballot_1.clone(), ballot_3]),
                test_fn: Arc::new(|msg, ballots| {
                    ballots
                        .intersection(&msg.votes_or_accepts_prepared())
                        .cloned()
                        .collect()
                }),
            },
        );
        assert_eq!(
            node_ids,
            HashSet::from_iter(vec![test_node_id(2), test_node_id(3)])
        );
        assert_eq!(pred.result(), HashSet::from_iter(vec![ballot_1]));
    }

    #[test]
    // ValueSetPredicate can be used to pick a set of values that has reached quorum.
    pub fn test_value_set_predicate_quorum() {
        let local_node_id = test_node_id(1);
        let local_node_quorum_set = quorum_set_from_str("([2],2,3,4,5)");
        let node_2_quorum_set = quorum_set_from_str("([1],1,3,4,5)");
        let node_3_quorum_set = quorum_set_from_str("([1],1,2,4,5)");
        let node_4_quorum_set = quorum_set_from_str("([1],1,2,3,5)");
        let node_5_quorum_set = quorum_set_from_str("([1],1,2,3,4)");

        let values_1 = BTreeSet::from_iter(vec!["a".to_string(), "A".to_string()]);
        let values_2 = BTreeSet::from_iter(vec!["b".to_string(), "B".to_string()]);

        let mut msgs = HashMap::<NodeID, Msg<String>>::default();

        // Node 2 and 3 form a quorum, voting on values_1
        let topic = Topic::Nominate(NominatePayload {
            X: values_1.clone(),
            Y: BTreeSet::default(),
        });
        msgs.insert(
            test_node_id(2),
            Msg::new(test_node_id(2), node_2_quorum_set, 1, topic.clone()),
        );
        msgs.insert(
            test_node_id(3),
            Msg::new(test_node_id(3), node_3_quorum_set, 1, topic),
        );

        // Node 4 and 5 form a quorum, voting on values_2
        let topic = Topic::Nominate(NominatePayload {
            X: values_2,
            Y: BTreeSet::default(),
        });
        msgs.insert(
            test_node_id(4),
            Msg::new(test_node_id(4), node_4_quorum_set, 1, topic.clone()),
        );
        msgs.insert(
            test_node_id(5),
            Msg::new(test_node_id(5), node_5_quorum_set, 1, topic),
        );

        // Look for quorum that can agree on [1110, 1111]
        let (node_ids, pred) = local_node_quorum_set.findQuorum(
            &local_node_id,
            &msgs,
            ValueSetPredicate {
                values: BTreeSet::from_iter(vec![
                    "a".to_string(),
                    "A".to_string(),
                    "c".to_string(),
                    "C".to_string(),
                ]),
                test_fn: Arc::new(|msg, values| match msg.votes_or_accepts_nominated() {
                    None => BTreeSet::default(),
                    Some(values2) => values.intersection(&values2).cloned().collect(),
                }),
            },
        );
        assert_eq!(
            node_ids,
            HashSet::from_iter(vec![test_node_id(1), test_node_id(2), test_node_id(3)])
        );
        assert_eq!(pred.result(), values_1);
    }

    #[test]
    // ValueSetPredicate can be used to pick a set values that has reached blocking threshold.
    pub fn test_value_set_predicate_blocking_set() {
        // Node 2 and 3 form a blocking set. Node 5 and 6 also form a blocking set.
        let local_node_quorum_set = quorum_set_from_str("([2],([2],2,3,4),([2],5,6,7))");
        let node_2_quorum_set = quorum_set_from_str("([1],3,4)");
        let node_3_quorum_set = quorum_set_from_str("([1],2,4)");
        let node_5_quorum_set = quorum_set_from_str("([1],6,7)");
        let node_6_quorum_set = quorum_set_from_str("([1],5,7)");

        let values_1 = BTreeSet::from_iter(vec!["a".to_string(), "A".to_string()]);
        let values_2 = BTreeSet::from_iter(vec!["b".to_string(), "B".to_string()]);

        let mut msgs = HashMap::<NodeID, Msg<String>>::default();

        // Node 2 and 3 form a blocking set , voting on values_1
        let topic = Topic::Nominate(NominatePayload {
            X: values_1.clone(),
            Y: BTreeSet::default(),
        });
        msgs.insert(
            test_node_id(2),
            Msg::new(test_node_id(2), node_2_quorum_set, 1, topic.clone()),
        );
        msgs.insert(
            test_node_id(3),
            Msg::new(test_node_id(3), node_3_quorum_set, 1, topic),
        );

        // Node 5 and 6 also form a blocking set, voting on values_2
        let topic = Topic::Nominate(NominatePayload {
            X: values_2,
            Y: BTreeSet::default(),
        });
        msgs.insert(
            test_node_id(5),
            Msg::new(test_node_id(5), node_5_quorum_set, 1, topic.clone()),
        );
        msgs.insert(
            test_node_id(6),
            Msg::new(test_node_id(6), node_6_quorum_set, 1, topic),
        );

        // Look for blocking set that intersects with ["a", "A", "c", C"]
        let (node_ids, pred) = local_node_quorum_set.findBlockingSet(
            &msgs,
            ValueSetPredicate {
                values: BTreeSet::from_iter(vec![
                    "a".to_string(),
                    "A".to_string(),
                    "c".to_string(),
                    "C".to_string(),
                ]),
                test_fn: Arc::new(|msg, values| match msg.votes_or_accepts_nominated() {
                    None => BTreeSet::default(),
                    Some(values2) => values.intersection(&values2).cloned().collect(),
                }),
            },
        );
        assert_eq!(
            node_ids,
            HashSet::from_iter(vec![test_node_id(2), test_node_id(3)])
        );
        assert_eq!(pred.result(), values_1);
    }

    #[test]
    // MinMaxPredicate can be used to narrow down min and max CN/HN values in a quorum.
    pub fn test_quorum_min_max_predicate() {
        let local_node_id = test_node_id(1);
        let local_node_quorum_set = quorum_set_from_str("([2],2,3,4,5)");
        let node_2_quorum_set = quorum_set_from_str("([1],1,3,4,5)");
        let node_3_quorum_set = quorum_set_from_str("([1],1,2,4,5)");
        let node_4_quorum_set = quorum_set_from_str("([1],1,2,3,5)");
        let node_5_quorum_set = quorum_set_from_str("([1],1,2,3,4)");

        let ballot_1 = Ballot::new(1, &["a".to_string(), "A".to_string()]);
        let ballot_2 = Ballot::new(1, &["b".to_string(), "B".to_string()]);

        let mut msgs = HashMap::<NodeID, Msg<String>>::default();

        // Node 2 and 3 form a quorum, committing ballot_1
        let topic = Topic::Commit(CommitPayload {
            B: ballot_1.clone(),
            PN: 0,
            CN: 10,
            HN: 20,
        });
        msgs.insert(
            test_node_id(2),
            Msg::new(test_node_id(2), node_2_quorum_set, 1, topic),
        );

        let topic = Topic::Commit(CommitPayload {
            B: ballot_1.clone(),
            PN: 0,
            CN: 15,
            HN: 25,
        });
        msgs.insert(
            test_node_id(3),
            Msg::new(test_node_id(3), node_3_quorum_set, 1, topic),
        );

        // Node 4 and 5 form a quorum, committing ballot_2
        let topic = Topic::Commit(CommitPayload {
            B: ballot_2,
            PN: 0,
            CN: 30,
            HN: 40,
        });
        msgs.insert(
            test_node_id(4),
            Msg::new(test_node_id(4), node_4_quorum_set, 1, topic.clone()),
        );
        msgs.insert(
            test_node_id(5),
            Msg::new(test_node_id(5), node_5_quorum_set, 1, topic),
        );

        let (node_ids, pred) = local_node_quorum_set.findQuorum(
            &local_node_id,
            &msgs,
            MinMaxPredicate {
                min: 1,
                max: <u32>::max_value(),
                values: ballot_1.X,
                test_fn: Arc::new(|msg, values, min, max| msg.accepts_commits(values, min, max)),
            },
        );
        assert_eq!(
            node_ids,
            HashSet::from_iter(vec![test_node_id(1), test_node_id(2), test_node_id(3)])
        );

        // Our inputs are (1, max u32), (10, 20), (15, 25). (15, 20) is the intersection of
        // those.
        assert_eq!(pred.result(), (15, 20));
    }
}
