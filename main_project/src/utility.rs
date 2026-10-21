use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::pubkey::Pubkey;

pub fn show_account_info(account: &AccountInfo) {
    msg!("------ [main]: show_account_info ------");
    msg!("key: {}", account.key);
    msg!("lamports: {}", account.lamports());
    msg!("account data: {:?}", account.data);
    msg!("owner: {}", account.owner);
    msg!("is_signer: {}", account.is_signer);
    msg!("is_writable: {}", account.is_writable);
    msg!("executable: {}", account.executable);
    msg!("-----------------------------------------");
}

pub fn show_accounts_info(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) {
    msg!("[main]: show_accounts_info");

    msg!("program_id: {}", program_id);
    msg!("accounts count: {}", accounts.len());

    for (i, account) in accounts.iter().enumerate() {
        msg!("--- Account [{}] ---", i);
        msg!("key: {}", account.key);
        msg!("lamports: {}", account.lamports());
        msg!("account data: {:?}", account.data);
        msg!("owner: {}", account.owner);
        msg!("is_signer: {}", account.is_signer);
        msg!("is_writable: {}", account.is_writable);
        msg!("executable: {}", account.executable);
        msg!("data: [{:?}]", data);
        msg!("");
    }
}
