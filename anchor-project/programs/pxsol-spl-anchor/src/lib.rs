use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022::{self, Token2022, TransferChecked};
use anchor_spl::token_interface::{Mint, TokenAccount};

declare_id!("21bhu54GetZ1EkmC9J1BkQvGAv37hxoZvcg3nuEy7r9h");

/// Amount to airdrop: 5 tokens with 9 decimals
const AIRDROP_AMOUNT: u64 = 5_000_000_000;
const TOKEN_DECIMALS: u8 = 9;

#[program]
pub mod pxsol_spl_anchor {
    use super::*;

    /// Airdrop SPL tokens to a user
    ///
    /// Anyone can call this instruction to receive tokens from the airdrop pool.
    /// The user must pay the transaction fee but no other permissions are required.
    pub fn airdrop(ctx: Context<Airdrop>) -> Result<()> {
        // The pool authority is a PDA of this program with empty seeds, so the
        // program can sign the transfer on its behalf without a stored keypair.
        let signer_seeds_flat = &[&[][..], &[ctx.bumps.mana_auth]];
        let signer_seeds = &[&signer_seeds_flat[..]];

        let cpi_program = ctx.accounts.token_program.key();
        let cpi_account = TransferChecked {
            from: ctx.accounts.mana_spla.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.user_spla.to_account_info(),
            authority: ctx.accounts.mana_auth.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_account, signer_seeds);
        token_2022::transfer_checked(cpi_ctx, AIRDROP_AMOUNT, TOKEN_DECIMALS)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Airdrop<'info> {
    /// Recipient; pays for the new associated token account and the tx fee.
    #[account(mut)]
    pub user: Signer<'info>,
    /// Recipient ATA, created on the fly if it does not exist yet.
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_spla: InterfaceAccount<'info, TokenAccount>,
    /// The program itself; the constraint blocks passing a fake program id.
    #[account(
        constraint = mana.key() == crate::ID @ ErrorCode::InvalidManaProgram
    )]
    pub mana: Program<'info, crate::program::PxsolSplAnchor>,
    /// Pool authority PDA: seeds = [b""] under this program id.
    #[account(
        seeds = [b""],
        bump,
        seeds::program = mana.key(),
    )]
    pub mana_auth: SystemAccount<'info>,
    /// Pool ATA holding the tokens to hand out.
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = mana_auth,
        associated_token::token_program = token_program,
    )]
    pub mana_spla: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Program<'info, Token2022>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid mana program")]
    InvalidManaProgram,
}
