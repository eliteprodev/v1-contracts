use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{Api, MessageInfo};

use white_whale::luna_vault::msg::ExecuteMsg;
use crate::tests::common_integration::{instantiate_msg};
use crate::contract::execute;
use crate::error::LunaVaultError;
use crate::state::STATE;
use crate::tests::common::TEST_CREATOR;
use crate::tests::instantiate::mock_instantiate;
use crate::tests::mock_querier::mock_dependencies;

#[test]
fn unsuccessful_set_state_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let msg = instantiate_msg(3, "warchest".to_string(), "anchor".to_string(), "bluna".to_string());
    let info = MessageInfo {
        sender: deps.api.addr_validate("unauthorized").unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(LunaVaultError::Admin(_)) => (),
        _ => panic!("Must return StableVaultError::Admin"),
    }
}

#[test]
fn successful_set_state() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let original_state = STATE.load(deps.as_mut().storage).unwrap();

    let msg = ExecuteMsg::UpdateState {
        anchor_money_market_address: Some(String::from("new_anchor_money_market_address")),
        aust_address: Some(String::from("new_aust_address")),
        allow_non_whitelisted: Some(true),
    };
    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let new_state = STATE.load(deps.as_mut().storage).unwrap();
    assert_ne!(original_state, new_state.clone());
    assert_eq!(
        new_state.anchor_money_market_address,
        deps.api
            .addr_validate("new_anchor_money_market_address")
            .unwrap()
    );
    assert_eq!(
        new_state.bluna_address,
        deps.api.addr_validate("new_aust_address").unwrap()
    );
    assert_eq!(new_state.allow_non_whitelisted, true);
}
