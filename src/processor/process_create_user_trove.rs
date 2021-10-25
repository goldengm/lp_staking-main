use anchor_lang::prelude::*;

use crate::{
    instructions::*
};

pub fn process_create_user_trove(
    ctx: Context<CreateUserTrove>, 
    user_trove_nonce:u8, 
    token_vault_nonce:u8
) -> ProgramResult {
    
    ctx.accounts.user_trove.locked_coll_balance = 0;
    ctx.accounts.user_trove.debt = 0;
    Ok(())
}
