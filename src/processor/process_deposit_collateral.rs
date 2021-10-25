use anchor_lang::{
    prelude::*,
    solana_program::{
        program::{invoke_signed},
        instruction::{AccountMeta, Instruction},
    }
};
use anchor_spl::token::{self,  Transfer, ID};
use arrayref::{array_mut_ref, mut_array_refs, array_ref};
use crate::{
    constant::*,
    instructions::*
};

#[derive(Clone, Copy, Debug)]
pub struct Stake {
    pub instruction: u8,
    pub amount: u64,
}

impl Stake {
    pub const LEN: usize = 9;

    pub fn get_size(&self) -> usize {
        Stake::LEN
    }

    pub fn pack(&self, output: &mut [u8]) -> Result<usize, ProgramError> {

        let output = array_mut_ref![output, 0, Stake::LEN];

        let (instruction_out, amount_out) = mut_array_refs![output, 1, 8];

        instruction_out[0] = self.instruction as u8;
        *amount_out = self.amount.to_le_bytes();

        Ok(Stake::LEN)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, ProgramError> {
        let mut output: [u8; Stake::LEN] = [0; Stake::LEN];
        if let Ok(len) = self.pack(&mut output[..]) {
            Ok(output[..len].to_vec())
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

fn get_token_balance(token_account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = token_account.try_borrow_data()?;
    let amount = array_ref![data, 64, 8];

    Ok(u64::from_le_bytes(*amount))
}

pub fn process_deposit_collateral(
    ctx: Context<DepositCollateral>, 
    amount: u64, 
    token_vault_nonce: u8, 
    user_trove_nonce: u8, 
    token_coll_nonce: u8
) -> ProgramResult {
    
    // transfer from user to pool
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_coll.to_account_info().clone(),
        to: ctx.accounts.pool_token_coll.to_account_info().clone(),
        authority: ctx.accounts.owner.to_account_info().clone(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info().clone();
    
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx, amount)?;

    ctx.accounts.token_vault.total_coll += amount;
    ctx.accounts.user_trove.locked_coll_balance += amount;

    // integration with raydium staking program to deposit lp

    // accounts for invoke raydium program instruction
    let mut raydium_accounts = Vec::with_capacity(17);
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.raydium_pool_id.key, false));
    raydium_accounts.push(AccountMeta::new_readonly(*ctx.accounts.raydium_pool_authority.key, false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.pool_associated_info_account.key, false));
    raydium_accounts.push(AccountMeta::new_readonly(*ctx.accounts.pool_main_account.key, true));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.pool_lp_account.key, false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.raydium_pool_lp_account.key, false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.pool_reward_token_account.key, false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.raydium_pool_reward_token_account.key, false));
    raydium_accounts.push(AccountMeta::new_readonly(ctx.accounts.clock.key(), false));
    raydium_accounts.push(AccountMeta::new_readonly(ctx.accounts.token_program.key(), false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.pool_reward_token_b_account.key, false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.raydium_pool_reward_token_b_account.key, false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.pool_info_account_one.key, false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.pool_info_account_two.key, false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.pool_info_account_three.key, false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.pool_info_account_four.key, false));
    raydium_accounts.push(AccountMeta::new(*ctx.accounts.pool_info_account_five.key, false));

    // AccountInfos for invoke_signed
    let account_infos = &[
        ctx.accounts.raydium_pool_id.clone(),
        ctx.accounts.raydium_pool_authority.clone(),
        ctx.accounts.pool_associated_info_account.clone(),
        ctx.accounts.pool_main_account.clone(),
        ctx.accounts.pool_lp_account.clone(),
        ctx.accounts.raydium_pool_lp_account.clone(),
        ctx.accounts.pool_reward_token_account.clone(),
        ctx.accounts.raydium_pool_reward_token_account.clone(),
        ctx.accounts.clock.to_account_info().clone(),
        ctx.accounts.token_program.to_account_info().clone(),
        ctx.accounts.pool_reward_token_b_account.clone(),
        ctx.accounts.raydium_pool_reward_token_b_account.clone(),
        ctx.accounts.pool_info_account_one.clone(),
        ctx.accounts.pool_info_account_two.clone(),
        ctx.accounts.pool_info_account_three.clone(),
        ctx.accounts.pool_info_account_four.clone(),
        ctx.accounts.pool_info_account_five.clone()
    ];

    // raydium program address
    let raydium_program = ctx.accounts.raydium_program_id.clone();
    
    // instruction to invoke raydium program
    let instruction = Instruction {
        program_id: *raydium_program.key,
        accounts: raydium_accounts,
        data: Stake {
            instruction: 11,
            amount: amount
        }
        .to_vec()?,
    };

    // seed of token_vault account to sign the transaction
    let signer_seeds = &[
        TOKEN_VAULT_TAG,
        ctx.accounts.token_vault.mint_coll.as_ref(),
        &[token_vault_nonce]
    ];
    let signer = &[&signer_seeds[..]];

    // invoke the raydium program
    invoke_signed(&instruction, account_infos, signer)?;

    // now lp tokens is sent from token vault to raydium

    // reward token amount
    let reward_amount = get_token_balance(&ctx.accounts.pool_reward_token_account)?;

    // transfer reward from pool to user
    let cpi_reward_accounts = Transfer {
        from: ctx.accounts.pool_token_coll.to_account_info(),
        to: ctx.accounts.user_token_coll.to_account_info(),
        authority: ctx.accounts.token_vault.to_account_info(),
    };

    let cip_reward_program = ctx.accounts.token_program.to_account_info();

    let cpi_reward_ctx = CpiContext::new_with_signer(cip_reward_program, cpi_reward_accounts, signer);
    msg!("transfering ...");
    token::transfer(cpi_reward_ctx, reward_amount)?;

    Ok(())
}
