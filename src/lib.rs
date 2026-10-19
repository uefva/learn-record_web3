
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("[main]: Hello, Web3!");

    print_account_info(program_id, accounts, data);



    Ok(())
}

pub fn print_account_info(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) {
    msg!("[main]: print_account_info");

    msg!("program_id: {}", program_id);
    msg!("accounts count: {}", accounts.len());

    for (i, account) in accounts.iter().enumerate() {
        msg!("--- Account [{}] ---", i);
        msg!("pubkey: {}", account.key);
        msg!("owner: {}", account.owner);
        msg!("is_signer: {}", account.is_signer);
        msg!("is_writable: {}", account.is_writable);
        msg!("executable: {}", account.executable);
        msg!("lamports: {}", account.lamports());
        msg!("data len: {}", account.data_len());
        msg!("account data: {:?}", account.data);
        msg!("data: [{:?}]", data);
        msg!("");
        msg!("");
    }
}
