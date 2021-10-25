use anchor_lang::prelude::*;
use crate::{
    constant::*,
    error::*,
    pyth,
    pyth::*
};
use std::convert::TryInto;
use std::convert::TryFrom;
use spl_math::{precise_number::PreciseNumber};


pub fn get_market_price_devnet(risk_level: u8) -> u64 {
    return 10_000_000_000;
}
pub fn assert_debt_allowed(locked_coll_balance: u64, user_debt: u64, amount: u64, risk_level: u8)-> ProgramResult{
    let market_price = get_market_price_devnet(risk_level);
    let debt_limit = market_price * locked_coll_balance / 100_000_000_000;

    if debt_limit < user_debt + amount {
        return Err(StablePoolError::NotAllowed.into())
    }
    Ok(())
}

pub fn assert_limit_mint(cur_timestamp: u64, last_mint_time: u64)-> ProgramResult{
    if cur_timestamp < last_mint_time + LIMIT_MINT_TIME {
        return Err(StablePoolError::NotAllowed.into())
    }
    Ok(())
}

pub fn get_pyth_product_quote_currency(pyth_product: &pyth::Product) -> Result<[u8; 32]> {
    const LEN: usize = 14;
    const KEY: &[u8; LEN] = b"quote_currency";

    let mut start = 0;
    while start < pyth::PROD_ATTR_SIZE {
        let mut length = pyth_product.attr[start] as usize;
        start += 1;

        if length == LEN {
            let mut end = start + length;
            if end > pyth::PROD_ATTR_SIZE {
                msg!("Pyth product attribute key length too long");
                return Err(StablePoolError::InvalidOracleConfig.into());
            }

            let key = &pyth_product.attr[start..end];
            if key == KEY {
                start += length;
                length = pyth_product.attr[start] as usize;
                start += 1;

                end = start + length;
                if length > 32 || end > pyth::PROD_ATTR_SIZE {
                    msg!("Pyth product quote currency value too long");
                    return Err(StablePoolError::InvalidOracleConfig.into());
                }

                let mut value = [0u8; 32];
                value[0..length].copy_from_slice(&pyth_product.attr[start..end]);
                return Ok(value);
            }
        }

        start += length;
        start += 1 + pyth_product.attr[start] as usize;
    }

    msg!("Pyth product quote currency not found");
    Err(StablePoolError::InvalidOracleConfig.into())
}

pub fn get_pyth_price(pyth_price_info: &AccountInfo, clock: &Clock) -> Result<Decimal> {
    const STALE_AFTER_SLOTS_ELAPSED: u64 = 5;

    let pyth_price_data = pyth_price_info.try_borrow_data()?;
    let pyth_price = pyth::load::<pyth::Price>(&pyth_price_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if pyth_price.ptype != pyth::PriceType::Price {
        msg!("Oracle price type is invalid");
        return Err(StablePoolError::InvalidOracleConfig.into());
    }

    let slots_elapsed = clock
        .slot
        .checked_sub(pyth_price.valid_slot)
        .ok_or(StablePoolError::MathOverflow)?;
    if slots_elapsed >= STALE_AFTER_SLOTS_ELAPSED {
        msg!("Oracle price is stale");
        return Err(StablePoolError::InvalidOracleConfig.into());
    }

    let price: u64 = pyth_price.agg.price.try_into().map_err(|_| {
        msg!("Oracle price cannot be negative");
        StablePoolError::InvalidOracleConfig
    })?;

    let market_price = if pyth_price.expo >= 0 {
        let exponent = pyth_price
            .expo
            .try_into()
            .map_err(|_| StablePoolError::MathOverflow)?;
        let zeros = 10u64
            .checked_pow(exponent)
            .ok_or(StablePoolError::MathOverflow)?;
        Decimal::from(price).try_mul(zeros)?
    } else {
        let exponent = pyth_price
            .expo
            .checked_abs()
            .ok_or(StablePoolError::MathOverflow)?
            .try_into()
            .map_err(|_| StablePoolError::MathOverflow)?;
        let decimals = 10u64
            .checked_pow(exponent)
            .ok_or(StablePoolError::MathOverflow)?;
        Decimal::from(price).try_div(decimals)?
    };

    Ok(market_price)
}

pub fn get_market_price(
    oracle_program_id:Pubkey, 
    quote_currency:[u8; 32], 
    pyth_product_info:&AccountInfo, 
    pyth_price_info:&AccountInfo, 
    clock:&Clock
)->Result<u128>{
    // get market price
    if &oracle_program_id != pyth_product_info.owner {
        msg!("Pyth product account provided is not owned by the lending market oracle program");
        return Err(StablePoolError::InvalidOracleConfig.into());
    }
    if &oracle_program_id != pyth_price_info.owner {
        msg!("Pyth price account provided is not owned by the lending market oracle program");
        return Err(StablePoolError::InvalidOracleConfig.into());
    }

    let pyth_product_data = pyth_product_info.try_borrow_data()?;
    let pyth_product = pyth::load::<pyth::Product>(&pyth_product_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    if pyth_product.magic != pyth::MAGIC {
        msg!("Pyth product account provided is not a valid Pyth account");
        return Err(StablePoolError::InvalidOracleConfig.into());
    }
    if pyth_product.ver != pyth::VERSION_2 {
        msg!("Pyth product account provided has a different version than expected");
        return Err(StablePoolError::InvalidOracleConfig.into());
    }
    if pyth_product.atype != pyth::AccountType::Product as u32 {
        msg!("Pyth product account provided is not a valid Pyth product account");
        return Err(StablePoolError::InvalidOracleConfig.into());
    }

    let pyth_price_pubkey_bytes: &[u8; 32] = pyth_price_info
        .key
        .as_ref()
        .try_into()
        .map_err(|_| StablePoolError::InvalidAccountInput)?;
    if &pyth_product.px_acc.val != pyth_price_pubkey_bytes {
        msg!("Pyth product price account does not match the Pyth price provided");
        return Err(StablePoolError::InvalidOracleConfig.into());
    }

    let _quote_currency = get_pyth_product_quote_currency(pyth_product)?;
    if quote_currency != _quote_currency {
        msg!("Lending market quote currency does not match the oracle quote currency");
        return Err(StablePoolError::InvalidOracleConfig.into());
    }

    let market_price = get_pyth_price(pyth_price_info, clock)?;

    Ok(market_price.try_round_u64().unwrap() as u128)
}
pub fn assert_devnet()->ProgramResult {
    if !DEVNET_MODE {
        return Err(StablePoolError::InvalidCluster.into());
    }
    Ok(())
}
