
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("[process_instruction]: Hello, world!");

    msg!("program_id: {}", _program_id);
    msg!("accounts count: {}", _accounts.len());

    for (i, account) in _accounts.iter().enumerate() {
        msg!("--- Account [{}] ---", i);
        msg!("pubkey: {}", account.key);
        msg!("owner: {}", account.owner);
        msg!("is_signer: {}", account.is_signer);
        msg!("is_writable: {}", account.is_writable);
        msg!("executable: {}", account.executable);
        msg!("lamports: {}", account.lamports());
        msg!("data_len: {}", account.data_len());
    }

    msg!("instruction_data len: {}", _instruction_data.len());
    msg!("instruction_data: {:?}", _instruction_data);

    Ok(())
}
