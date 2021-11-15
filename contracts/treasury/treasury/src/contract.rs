#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CanonicalAddr, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdError, StdResult, Uint128,
};

use crate::error::TreasuryError;
use terraswap::asset::AssetInfo;
use white_whale::treasury::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use white_whale::treasury::state::{State, ADMIN, STATE, VAULT_ASSETS};
use white_whale::treasury::vault_assets::{get_identifier, VaultAsset};
type TreasuryResult = Result<Response, TreasuryError>;

/*
    The treasury behaves similarly to a community fund with the provisio that funds in the treasury are used to provide staking rewards to stakers.
    It is controlled by the governance contract and serves to grow its holdings and become a safeguard/protective measure in keeping the peg.
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> TreasuryResult {
    STATE.save(deps.storage, &State { traders: vec![] })?;
    let admin_addr = Some(info.sender);
    ADMIN.set(deps, admin_addr)?;

    Ok(Response::default())
}

// Routers; here is a separate router which handles Execution of functions on the contract or performs a contract Query
// Each router function defines a number of handlers using Rust's pattern matching to
// designated how each ExecutionMsg or QueryMsg will be handled.

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, _env: Env, info: MessageInfo, msg: ExecuteMsg) -> TreasuryResult {
    match msg {
        ExecuteMsg::SetAdmin { admin } => {
            let admin_addr = deps.api.addr_validate(&admin)?;
            let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
            ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
            Ok(Response::default()
                .add_attribute("previous admin", previous_admin)
                .add_attribute("admin", admin))
        }
        ExecuteMsg::AddTrader { trader } => add_trader(deps, info, trader),
        ExecuteMsg::RemoveTrader { trader } => remove_trader(deps, info, trader),
        ExecuteMsg::TraderAction { msgs } => execute_action(deps, info, msgs),
        ExecuteMsg::UpdateAssets { to_add, to_remove } => {
            update_assets(deps, info, to_add, to_remove)
        }
    }
}

pub fn execute_action(
    deps: DepsMut,
    msg_info: MessageInfo,
    msgs: Vec<CosmosMsg<Empty>>,
) -> TreasuryResult {
    let state = STATE.load(deps.storage)?;
    if !state
        .traders
        .contains(&deps.api.addr_canonicalize(msg_info.sender.as_str())?)
    {
        return Err(TreasuryError::SenderNotWhitelisted {});
    }

    Ok(Response::new().add_messages(msgs))
}

pub fn update_assets(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<VaultAsset>,
    to_remove: Vec<AssetInfo>,
) -> TreasuryResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for new_asset in to_add.into_iter() {
        let id = get_identifier(&new_asset.asset.info).as_str();
        // update function for new or existing keys
        let insert = |vault_asset: Option<VaultAsset>| -> StdResult<VaultAsset> {
            match vault_asset {
                Some(_) => Err(StdError::generic_err("Asset already present.")),
                None => {
                    let mut asset = new_asset.clone();
                    asset.asset.amount = Uint128::zero();
                    Ok(asset)
                }
            }
        };
        VAULT_ASSETS.update(deps.storage, id, insert)?;
    }

    for asset_id in to_remove {
        VAULT_ASSETS.remove(deps.storage, get_identifier(&asset_id).as_str());
    }

    Ok(Response::new().add_attribute("action", "update_cw20_token_list"))
}

pub fn add_trader(deps: DepsMut, msg_info: MessageInfo, trader: String) -> TreasuryResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    if state
        .traders
        .contains(&deps.api.addr_canonicalize(&trader)?)
    {
        return Err(TreasuryError::AlreadyInList {});
    }

    // Add contract to whitelist.
    state.traders.push(deps.api.addr_canonicalize(&trader)?);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Added contract to whitelist: ", trader))
}

pub fn remove_trader(deps: DepsMut, msg_info: MessageInfo, trader: String) -> TreasuryResult {
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let mut state = STATE.load(deps.storage)?;
    if !state
        .traders
        .contains(&deps.api.addr_canonicalize(&trader)?)
    {
        return Err(TreasuryError::NotInList {});
    }

    // Remove contract from whitelist.
    let canonical_addr = deps.api.addr_canonicalize(&trader)?;
    state.traders.retain(|addr| *addr != canonical_addr);
    STATE.save(deps.storage, &state)?;

    // Respond and note the change
    Ok(Response::new().add_attribute("Removed contract from whitelist: ", trader))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetTotalValue {} => to_binary(&compute_total_value(deps, env)?),
        QueryMsg::GetHoldingValue { identifier } => {
            to_binary(&compute_holding_value(deps, &env, identifier)?)
        }
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = STATE.load(deps.storage)?;
    let traders: Vec<CanonicalAddr> = state.traders;
    let resp = ConfigResponse {
        traders: traders
            .iter()
            .map(|trader| -> String { deps.api.addr_humanize(trader).unwrap().to_string() })
            .collect(),
    };
    Ok(resp)
}

pub fn compute_holding_value(deps: Deps, env: &Env, holding: String) -> StdResult<Uint128> {
    let mut vault_asset: VaultAsset = VAULT_ASSETS.load(deps.storage, holding.as_str())?;
    let value = vault_asset.value(deps, env, None)?;
    Ok(value)
}

pub fn compute_total_value(_deps: Deps, _env: Env) -> StdResult<Uint128> {
    Ok(Uint128::zero())
}
