#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CompareResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::GAME;

use cw20_base::contract::{execute_mint, query_token_info};
use cw20_base::state::{MinterData, TokenInfo, TOKEN_INFO};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:krzyzyk";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply: Uint128::zero(),
        mint: Some(MinterData {
            minter: info.sender.clone(),
            cap: None,
        }),
    };
    TOKEN_INFO.save(deps.storage, &data)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ChooseOption { address, option } => {
            Ok(try_choose_option(deps, info, address, option)?)
        }
        ExecuteMsg::Mint { recipient, amount } => {
            Ok(execute_mint(deps, env, info, recipient, amount)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Compare {
            address_one,
            address_two,
        } => to_binary(&query_compare(deps, address_one, address_two)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
    }
}

// validation of option, fail if otpion is invalid
// can restart option
// 
pub fn try_choose_option(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    option: String,
) -> Result<Response, ContractError> {
    let config = TOKEN_INFO.load(deps.storage)?;

    if config.mint.is_none() || config.mint.as_ref().unwrap().minter != info.sender {
        return Err(ContractError::Unauthorized {});
    };

    let address_to_save_option = deps
        .api
        .addr_humanize(&deps.api.addr_canonicalize(&address).unwrap())
        .unwrap();

    GAME.save(deps.storage, &address_to_save_option, &option)?;

    Ok(Response::new().add_attribute("saved_option", &option))
}

pub fn query_compare(
    deps: Deps,
    address_one: String,
    address_two: String,
) -> StdResult<CompareResponse> {
    let first_address = deps
        .api
        .addr_humanize(&deps.api.addr_canonicalize(&address_one).unwrap())
        .unwrap();
    let second_address = deps
        .api
        .addr_humanize(&deps.api.addr_canonicalize(&address_two).unwrap())
        .unwrap();

    let option_first_addr = GAME.may_load(deps.storage, &first_address).unwrap();
    let option_second_addr = GAME.may_load(deps.storage, &second_address).unwrap();

    Ok(CompareResponse {
        option_addr_one: option_first_addr.unwrap(),
        option_addr_two: option_second_addr.unwrap(),
    })
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw20::TokenInfoResponse;

    use super::*;

    fn do_instantiate(mut deps: DepsMut) -> TokenInfoResponse {
        let instantiate_msg = InstantiateMsg {
            name: "Auto gen".to_string(),
            symbol: "AUTO".to_string(),
            decimals: 6,
        };

        let info = mock_info("creator", &[]);
        let env = mock_env();
        let res = instantiate(deps.branch(), env, info, instantiate_msg).unwrap();

        assert_eq!(0, res.messages.len());

        let meta = query_token_info(deps.as_ref()).unwrap();
        assert_eq!(
            meta,
            TokenInfoResponse {
                name: "Auto gen".to_string(),
                symbol: "AUTO".to_string(),
                decimals: 6,
                total_supply: Uint128::zero()
            }
        );
        meta
    }

    mod instantiate {
        use super::*;

        #[test]
        fn basic() {
            let mut deps = mock_dependencies(&[]);
            let amount = Uint128::new(12345678);
            do_instantiate(deps.as_mut());

            let msg = ExecuteMsg::Mint {
                recipient: "addrr0000".to_string(),
                amount: amount,
            };

            let info = mock_info("creator", &[]);
            let env = mock_env();

            let res = execute(deps.as_mut(), env, info, msg).unwrap();
            assert_eq!(0, res.messages.len());

            assert_eq!(
                query_token_info(deps.as_ref()).unwrap(),
                TokenInfoResponse {
                    name: "Auto gen".to_string(),
                    symbol: "AUTO".to_string(),
                    decimals: 6,
                    total_supply: Uint128::new(12345678)
                }
            )
        }

        #[test]
        fn test_queries() {
            let mut deps = mock_dependencies(&[]);
            let amount = Uint128::from(11223344u128);
            do_instantiate(deps.as_mut());

            // Mint to addrr0000 from creator
            let msg = ExecuteMsg::Mint {
                recipient: "addrr0000".into(),
                amount: amount,
            };

            let info = mock_info("creator", &[]);
            let env = mock_env();

            let res = execute(deps.as_mut(), env, info, msg).unwrap();
            assert_eq!(0, res.messages.len());

            let msg = ExecuteMsg::ChooseOption {
                address: "addrr0000".into(),
                option: "Papier".into(),
            };

            let info = mock_info("creator", &[]);
            let env = mock_env();

            let res = execute(deps.as_mut(), env, info, msg).unwrap();
            assert_eq!(0, res.messages.len());

            let msg = ExecuteMsg::ChooseOption {
                address: "addrr0001".into(),
                option: "Kamien".into(),
            };

            let info = mock_info("creator", &[]);
            let env = mock_env();

            let res = execute(deps.as_mut(), env, info, msg).unwrap();
            assert_eq!(0, res.messages.len());

            let env = mock_env();
            let data = query(
                deps.as_ref(),
                env,
                QueryMsg::Compare {
                    address_one: String::from("addrr0000"),
                    address_two: String::from("addrr0001"),
                },
            )
            .unwrap();

            let loaded: CompareResponse = from_binary(&data).unwrap();
            assert_eq!(loaded.option_addr_one, "Papier".to_string());
            assert_ne!(loaded.option_addr_one, "xd".to_string())
        }
    }
}
