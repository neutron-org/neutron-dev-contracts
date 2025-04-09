#[cfg(test)]
use crate::{
    contract::{execute, instantiate},
    msg::{ExecuteMsg, InstantiateMsg},
    state::{Config, CONFIG},
};
use cosmwasm_std::{
    testing::{message_info, mock_dependencies, mock_env},
    Addr, Response,
};
use neutron_sdk::NeutronResult;
use std::time::{Duration, Instant};

fn setup_contract(owner: &str, hash_iterations: u64) -> NeutronResult<()> {
    let msg = InstantiateMsg {
        owner: owner.to_string(),
        hash_iterations,
    };
    let info = message_info(&Addr::unchecked(owner), &[]);
    let env = mock_env();
    let mut deps = mock_dependencies();

    instantiate(deps.as_mut(), env, info, msg)?;
    Ok(())
}

fn execute_hashes_with_timing(hash_iterations: u64) -> (NeutronResult<Response>, Duration) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&Addr::unchecked("owner"), &[]);

    #[allow(clippy::unwrap_used)]
    CONFIG
        .save(
            deps.as_mut().storage,
            &Config {
                owner: "owner".to_string(),
                hash_iterations,
            },
        )
        .unwrap();

    // Measure execution time
    let start = Instant::now();
    let result = execute(deps.as_mut(), env, info, ExecuteMsg::Hashes {});
    let duration = start.elapsed();

    (result, duration)
}

#[test]
fn test_hash_iterations_benchmark() {
    let owner = "owner";

    // Test with different hash_iterations values
    let iterations = [100, 1000, 10000, 100000];

    for &iter in &iterations {
        // Setup contract with current iteration value
        #[allow(clippy::unwrap_used)]
        setup_contract(owner, iter).unwrap();

        // Execute and measure time
        let (result, duration) = execute_hashes_with_timing(iter);

        // Verify result
        assert!(result.is_ok(), "Execution failed for {} iterations", iter);

        // Print benchmark results
        println!("Hash iterations: {}, Time: {:?}", iter, duration);
        // a local run output:
        // Hash iterations: 100, Time: 993.25µs
        // Hash iterations: 1000, Time: 11.465625ms
        // Hash iterations: 10000, Time: 67.84ms
        // Hash iterations: 100000, Time: 529.474875ms
    }
}

#[test]
fn test_update_config_permissions() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let owner = "owner";
    let non_owner = "non_owner";
    let initial_hash_iterations = 1000;
    let new_hash_iterations = 2000;
    let new_owner = "new_owner";

    // Setup contract with initial owner and hash_iterations
    let msg = InstantiateMsg {
        owner: owner.to_string(),
        hash_iterations: initial_hash_iterations,
    };
    let info = message_info(&Addr::unchecked(owner), &[]);
    #[allow(clippy::unwrap_used)]
    instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Verify initial config
    #[allow(clippy::unwrap_used)]
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.owner, owner);
    assert_eq!(config.hash_iterations, initial_hash_iterations);

    // Test: Owner can update config
    let update_msg = ExecuteMsg::UpdateConfig {
        owner: new_owner.to_string(),
        hash_iterations: new_hash_iterations,
    };
    let info = message_info(&Addr::unchecked(owner), &[]);
    let result = execute(deps.as_mut(), env.clone(), info, update_msg);
    assert!(result.is_ok(), "Owner should be able to update config");

    // Verify config was updated
    #[allow(clippy::unwrap_used)]
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.owner, new_owner);
    assert_eq!(config.hash_iterations, new_hash_iterations);

    // Test: Non-owner cannot update config
    let update_msg = ExecuteMsg::UpdateConfig {
        owner: "another_owner".to_string(),
        hash_iterations: 3000,
    };
    let info = message_info(&Addr::unchecked(non_owner), &[]);
    let result = execute(deps.as_mut(), env, info, update_msg);
    assert!(
        result.is_err(),
        "Non-owner should not be able to update config"
    );

    // Verify config was not changed
    #[allow(clippy::unwrap_used)]
    let config = CONFIG.load(&deps.storage).unwrap();
    assert_eq!(config.owner, new_owner);
    assert_eq!(config.hash_iterations, new_hash_iterations);
}
