use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount,Mint};

use crate::{
    states::*,
    constant::*,
};

#[derive(Accounts)]
#[instruction(global_state_nonce:u8, mint_usd_nonce:u8)]
pub struct CreateGlobalState <'info>{
    #[account(mut)]
    pub super_owner:  Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [GLOBAL_STATE_TAG],
        bump = global_state_nonce,
        payer = super_owner,
        )]
    pub global_state: Account<'info, GlobalState>,

    #[account(init_if_needed,
        mint::decimals = USD_DECIMALS,
        mint::authority = global_state,
        seeds = [USD_MINT_TAG],
        bump = mint_usd_nonce,
        payer = super_owner)]
    pub mint_usd: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(token_vault_nonce:u8, global_state_nonce:u8, token_coll_nonce:u8, risk_level: u8)]
pub struct CreateTokenVault<'info> {
    #[account(mut)]
    pub payer:  Signer<'info>,

    #[account(
        init,
        seeds = [TOKEN_VAULT_TAG,mint_coll.key().as_ref()],
        bump = token_vault_nonce,
        payer = payer,
        constraint = payer.key() == global_state.super_owner)]
    pub token_vault: ProgramAccount<'info, TokenVault>,

    #[account(mut,
        seeds = [GLOBAL_STATE_TAG],
        bump = global_state_nonce)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    // pub token_a:Account<'info, TokenAccount>,
    // pub token_b:Account<'info, TokenAccount>,
    pub mint_coll:Account<'info, Mint>,

    #[account(init,
        token::mint = mint_coll,
        token::authority = token_vault,
        seeds = [
            TOKEN_VAULT_POOL_TAG, 
            token_vault.key().as_ref()
        ],
        bump = token_coll_nonce,
        payer = payer)]
    pub token_coll:Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(user_trove_nonce:u8, token_vault_nonce:u8)]
pub struct CreateUserTrove<'info> {
    #[account(mut)]
    pub trove_owner: Signer<'info>,

    #[account(
        init,
        seeds = [
            USER_TROVE_TAG,
            token_vault.key().as_ref(), 
            trove_owner.key().as_ref()
        ],
        bump = user_trove_nonce,
        payer = trove_owner,)]
    pub user_trove: ProgramAccount<'info, UserTrove>,

    #[account(mut,
        seeds = [
            TOKEN_VAULT_TAG,
            mint_coll.key().as_ref()
        ],
        bump = token_vault_nonce)]
    pub token_vault: ProgramAccount<'info, TokenVault>,

    #[account(mut,
        constraint = mint_coll.key() == token_vault.mint_coll)]
    pub mint_coll: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(amount: u64, token_vault_nonce: u8, user_trove_nonce: u8, token_coll_nonce: u8)]
pub struct DepositCollateral<'info> {
    pub owner:  Signer<'info>,

    #[account(mut,
        seeds = [
            USER_TROVE_TAG,
            token_vault.key().as_ref(), 
            owner.key().as_ref()
        ],
        bump = user_trove_nonce)]
    pub user_trove:ProgramAccount<'info, UserTrove>,

    #[account(mut,
        seeds = [
            TOKEN_VAULT_TAG,
            mint_coll.key().as_ref()
        ],
        bump = token_vault_nonce)]
    pub token_vault:ProgramAccount<'info, TokenVault>,

    #[account(mut,
        seeds = [
            TOKEN_VAULT_POOL_TAG,
            token_vault.key().as_ref()
        ],
        bump = token_coll_nonce)]
    pub pool_token_coll:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = user_token_coll.owner == owner.key(),
        constraint = user_token_coll.mint == token_vault.mint_coll)]
    pub user_token_coll:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = mint_coll.key() == token_vault.mint_coll)]
    pub mint_coll:Account<'info, Mint>,

    pub token_program:Program<'info, Token>,

    // accountInfos to invoke raydium staking program deposit function
    pub raydium_program_id: AccountInfo<'info>,

    #[account(mut)]
    pub raydium_pool_id: AccountInfo<'info>,

    pub raydium_pool_authority: AccountInfo<'info>,

    #[account(mut)]
    pub pool_associated_info_account: AccountInfo<'info>,
    
    pub pool_main_account: AccountInfo<'info>,

    #[account(mut)]
    pub pool_lp_account: AccountInfo<'info>,

    #[account(mut)]
    pub raydium_pool_lp_account: AccountInfo<'info>,

    #[account(mut)]
    pub pool_reward_token_account: AccountInfo<'info>,

    #[account(mut)]
    pub raydium_pool_reward_token_account: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,

    #[account(mut)]
    pub pool_reward_token_b_account: AccountInfo<'info>,

    #[account(mut)]
    pub raydium_pool_reward_token_b_account: AccountInfo<'info>,

    #[account(mut)]
    pub pool_info_account_one: AccountInfo<'info>,

    #[account(mut)]
    pub pool_info_account_two: AccountInfo<'info>,

    #[account(mut)]
    pub pool_info_account_three: AccountInfo<'info>,

    #[account(mut)]
    pub pool_info_account_four: AccountInfo<'info>,

    #[account(mut)]
    pub pool_info_account_five: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(amount: u64, token_vault_nonce: u8, user_trove_nonce: u8, token_coll_nonce: u8)]
pub struct WithdrawCollateral<'info> {
    pub owner:  Signer<'info>,

    #[account(mut,
        seeds = [
            USER_TROVE_TAG,
            token_vault.key().as_ref(), 
            owner.key().as_ref()
        ],
        bump = user_trove_nonce)]
    pub user_trove:ProgramAccount<'info, UserTrove>,

    #[account(mut,
        seeds = [
            TOKEN_VAULT_TAG,
            mint_coll.key().as_ref()
        ],
        bump = token_vault_nonce)]
    pub token_vault:ProgramAccount<'info, TokenVault>,

    #[account(mut,
        seeds = [
            TOKEN_VAULT_POOL_TAG,
            token_vault.key().as_ref()
        ],
        bump = token_coll_nonce)]
    pub pool_token_coll:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = user_token_coll.owner == owner.key(),
        constraint = user_token_coll.mint == token_vault.mint_coll)]
    pub user_token_coll:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = mint_coll.key() == token_vault.mint_coll)]
    pub mint_coll:Account<'info, Mint>,

    pub token_program:Program<'info, Token>,
    
    // accountInfos to invoke raydium staking program withdraw function
    pub raydium_program_id: AccountInfo<'info>,

    #[account(mut)]
    pub raydium_pool_id: AccountInfo<'info>,

    pub raydium_pool_authority: AccountInfo<'info>,

    #[account(mut)]
    pub pool_associated_info_account: AccountInfo<'info>,
    
    pub pool_main_account: AccountInfo<'info>,

    #[account(mut)]
    pub pool_lp_account: AccountInfo<'info>,

    #[account(mut)]
    pub raydium_pool_lp_account: AccountInfo<'info>,

    #[account(mut)]
    pub pool_reward_token_account: AccountInfo<'info>,

    #[account(mut)]
    pub raydium_pool_reward_token_account: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,

    #[account(mut)]
    pub pool_reward_token_b_account: AccountInfo<'info>,

    #[account(mut)]
    pub raydium_pool_reward_token_b_account: AccountInfo<'info>,

    #[account(mut)]
    pub pool_info_account_one: AccountInfo<'info>,

    #[account(mut)]
    pub pool_info_account_two: AccountInfo<'info>,

    #[account(mut)]
    pub pool_info_account_three: AccountInfo<'info>,

    #[account(mut)]
    pub pool_info_account_four: AccountInfo<'info>,

    #[account(mut)]
    pub pool_info_account_five: AccountInfo<'info>
}


#[derive(Accounts)]
#[instruction(amount: u64, token_vault_nonce: u8, user_trove_nonce: u8, global_state_nonce: u8, mint_usd_nonce: u8, user_usd_token_nonce: u8)]
pub struct BorrowUsd<'info> {
    #[account(mut)]
    pub owner:  Signer<'info>,
    
    #[account(mut,
        seeds = [TOKEN_VAULT_TAG,mint_coll.key().as_ref()],
        bump = token_vault_nonce)]
    pub token_vault: ProgramAccount<'info, TokenVault>,

    #[account(mut,
        seeds = [USER_TROVE_TAG,token_vault.key().as_ref(), owner.key().as_ref()],
        bump = user_trove_nonce)]
    pub user_trove: ProgramAccount<'info, UserTrove>,
    
    #[account(mut,
        seeds = [GLOBAL_STATE_TAG],
        bump = global_state_nonce)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    #[account(mut,
        seeds = [USD_MINT_TAG],
        bump = mint_usd_nonce,
        constraint = mint_usd.key() == global_state.mint_usd)]
    pub mint_usd: Account<'info, Mint>,

    #[account(
        init_if_needed,
        token::mint = mint_usd,
        token::authority = owner,
        seeds = [
            USER_USD_TOKEN_TAG, 
            owner.key().as_ref(), 
            mint_usd.key().as_ref()
        ],
        bump = user_usd_token_nonce,
        payer = owner)]
    pub user_token_usd: Account<'info, TokenAccount>,

    #[account(mut,
        constraint = mint_coll.key() == token_vault.mint_coll)]
    pub mint_coll:Account<'info, Mint>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,

    pub clock: Sysvar<'info, Clock>,
}

 
#[derive(Accounts)]
#[instruction(amount: u64, token_vault_nonce: u8, user_trove_nonce: u8, global_state_nonce: u8, mint_usd_nonce: u8, user_usd_token_nonce: u8)]
pub struct RepayUsd<'info> {
    pub owner:  Signer<'info>,
    #[account(mut,
        seeds = [
            TOKEN_VAULT_TAG,
            mint_coll.key().as_ref()
        ],
        bump = token_vault_nonce)]
    pub token_vault:ProgramAccount<'info, TokenVault>,

    #[account(mut,
        seeds = [
            USER_TROVE_TAG,
            token_vault.key().as_ref(), 
            owner.key().as_ref()
        ],
        bump = user_trove_nonce)]
    pub user_trove:ProgramAccount<'info, UserTrove>,

    #[account(mut,
        seeds = [GLOBAL_STATE_TAG],
        bump = global_state_nonce)]
    pub global_state: ProgramAccount<'info, GlobalState>,

    #[account(mut,
        seeds = [USD_MINT_TAG],
        bump = mint_usd_nonce,
        constraint = mint_usd.key() == global_state.mint_usd)]
    pub mint_usd:Account<'info, Mint>,
    
    #[account(mut)]
    pub user_token_usd:Account<'info, TokenAccount>,

    #[account(mut,
        constraint = mint_coll.key() == token_vault.mint_coll)]
    pub mint_coll:Account<'info, Mint>,
    
    pub token_program:Program<'info, Token>,
}
