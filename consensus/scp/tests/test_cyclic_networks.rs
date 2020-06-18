// Copyright (c) 2018-2020 MobileCoin Inc.

mod mock_network;

use mc_common::logger::{test_with_logger, Logger};
use serial_test_derive::serial;

/// Performs a consensus test for a cyclic network of `num_nodes` nodes.
fn cyclic_test_helper(num_nodes: usize, logger: Logger) {
    if num_nodes > 3 && mock_network::skip_slow_tests() {
        return;
    }

    let mut test_options = mock_network::TestOptions::new();
    test_options.values_to_submit = 1000;

    let network = mock_network::cyclic_topology::directed_cycle(num_nodes);
    mock_network::build_and_test(&network, &test_options, logger.clone());
}

#[test_with_logger]
#[serial]
fn cyclic_2(logger: Logger) {
    for _i in 0..1000 {
        cyclic_test_helper(2, logger.clone());
    }
}
