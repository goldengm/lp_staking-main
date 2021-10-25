use anchor_lang::prelude::*;
use anchor_spl::token::{self,  Burn, ID};

use crate::{
    constant::*,
    instructions::*,
};

pub fn process_repay_usd(
    ctx: Context<RepayUsd>, 
    amount: u64, 
    token_vault_nonce: u8, 
    user_trove_nonce: u8, 
    global_state_nonce: u8, 
    mint_usd_nonce: u8, 
    user_usd_token_nonce: u8
) -> ProgramResult {

    let mut _amount = amount;
    if ctx.accounts.user_trove.debt < amount {
        _amount = ctx.accounts.user_trove.debt;
    }
    // burn
    let cpi_accounts = Burn {
        mint: ctx.accounts.mint_usd.to_account_info().clone(),
        to: ctx.accounts.user_token_usd.to_account_info().clone(),
        authority: ctx.accounts.owner.to_account_info().clone(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info().clone();
    
    let signer_seeds = &[
        GLOBAL_STATE_TAG,
        &[global_state_nonce],
    ];
    let signer = &[&signer_seeds[..]];

    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    
    token::burn(cpi_ctx, _amount)?;

    ctx.accounts.token_vault.total_debt -= _amount;
    ctx.accounts.user_trove.debt -= _amount;

    Ok(())
}
