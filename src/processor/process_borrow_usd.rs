use anchor_lang::prelude::*;
use anchor_spl::token::{self,  MintTo, ID};

use crate::{
    error::*,
    constant::*,
    instructions::*,
    utils::*,
};

pub fn process_borrow_usd(
    ctx: Context<BorrowUsd>, 
    amount: u64, 
    token_vault_nonce: u8, 
    user_trove_nonce: u8, 
    global_state_nonce: u8, 
    mint_usd_nonce: u8, 
    user_usd_token_nonce: u8
) -> ProgramResult {
    
    assert_debt_allowed(ctx.accounts.user_trove.locked_coll_balance, ctx.accounts.user_trove.debt, amount, ctx.accounts.token_vault.risk_level)?;
    
    let cur_timestamp = ctx.accounts.clock.unix_timestamp as u64;
    
    assert_limit_mint(cur_timestamp, ctx.accounts.user_trove.last_mint_time)?;
    // mint to user
    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint_usd.to_account_info().clone(),
        to: ctx.accounts.user_token_usd.to_account_info().clone(),
        authority: ctx.accounts.global_state.to_account_info().clone(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info().clone();
    
    let signer_seeds = &[
        GLOBAL_STATE_TAG,
        &[global_state_nonce],
    ];
    let signer = &[&signer_seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::mint_to(cpi_ctx, amount)?;

    ctx.accounts.token_vault.total_debt += amount;
    ctx.accounts.user_trove.debt += amount;
    ctx.accounts.user_trove.last_mint_time = cur_timestamp;
    msg!("borrow4");
    Ok(())
}
