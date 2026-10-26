use anchor_lang::prelude::*;
use crate::err::PxsolError;


#[account]
pub struct Data {
    pub auth: Pubkey,  // Authority public key; only this key can update.
    pub bump: u8,      // PDA bump value ensuring unique address.
    pub data: Vec<u8>, // Business data with variable length.
}

impl Data {
    pub fn space_for(data_len: usize) -> usize {
        // 8 (discriminator) + 32 (authority) + 1 (bump) + 4 (vec len) + data_len.
        8 + 32 + 1 + 4 + data_len
    }
}

/// Fixed seed prefix for PDA derivation, ensuring a stable namespace and avoiding collisions.
const SEED: &[u8] = b"data";

#[derive(Accounts)]
pub struct Init<'info> {
    /// Instruction caller; must be mutable to pay costs and sign.
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,                                  // Creates the account on first call.
        payer = user,                          // Account rent/creation cost paid by user.
        seeds = [SEED, user.key().as_ref()],   // Must match initialization to locate the same PDA.
        bump,                                  // Must match initialization to locate the same PDA.
        space = Data::space_for(0)             // Initializes space with empty data.
    )]
    pub user_pda: Account<'info, Data>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    /// The authority allowed to mutate and fund reallocation.
    #[account(mut)]
    pub user: Signer<'info>,
    /// PDA derived from authority; reallocated dynamically to fit `new_data`.
    #[account(
        mut,                                                                 // Will write new data.
        seeds = [SEED, user.key().as_ref()],                                 // Must match initialization to locate the same PDA.
        bump = user_pda.bump,                                                // Must match initialization to locate the same PDA.
        constraint = user_pda.auth == user.key()
            @ PxsolError::Unauthorized,
        close = user
    )]
    pub user_pda: Account<'info, Data>,
}


#[derive(Accounts)]
#[instruction(new_data: Vec<u8>)]
pub struct Update<'info> {
    /// The authority allowed to mutate and fund reallocation.
    #[account(mut)]
    pub user: Signer<'info>,
    /// PDA derived from authority; reallocated dynamically to fit `new_data`.
    #[account(
        mut,                                                                 // Will write new data.
        seeds = [SEED, user.key().as_ref()],                                 // Must match initialization to locate the same PDA.
        bump = user_pda.bump,                                                // Must match initialization to locate the same PDA.
        realloc = Data::space_for(new_data.len()),                           // Reallocates space based on new data length.
        realloc::payer = user,                                               // Authority pays for expansion.
        realloc::zero = false,                                               // Preserves existing fields without zero-filling (only adjusts capacity).
        constraint = user_pda.auth == user.key() @ PxsolError::Unauthorized, // Additional authority equality check; throws `PxsolError::Unauthorized` on failure.
    )]
    pub user_pda: Account<'info, Data>,
    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
#[instruction(new_data: Vec<u8>)]
pub struct Append<'info> {
    /// The authority allowed to mutate and fund reallocation.
    #[account(mut)]
    pub user: Signer<'info>,
    /// PDA derived from authority; reallocated dynamically to fit `new_data`.
    #[account(
        mut,                                                                 // Will write new data.
        seeds = [SEED, user.key().as_ref()],                                 // Must match initialization to locate the same PDA.
        bump = user_pda.bump,                                                // Must match initialization to locate the same PDA.
        realloc = Data::space_for(user_pda.data.len() + new_data.len()),     // Reallocates space based on new data length.
        realloc::payer = user,                                               // Authority pays for expansion.
        realloc::zero = false,                                               // Preserves existing fields without zero-filling (only adjusts capacity).
        constraint = user_pda.auth == user.key() @ PxsolError::Unauthorized, // Additional authority equality check; throws `PxsolError::Unauthorized` on failure.
    )]
    pub user_pda: Account<'info, Data>,
    pub system_program: Program<'info, System>,
}
