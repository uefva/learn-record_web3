use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
};
use solana_program::sysvar::Sysvar;
use solana_program::program_error::ProgramError;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("[main]: Hello, Web3!");

    msg!("program_id: {}", program_id);

    // print_accounts_info(program_id, accounts.clone(), data);

    // account_counter(program_id, accounts)?;

    account_txt(program_id, accounts, data)?;

    Ok(())
}

pub fn account_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    msg!("[main] account_counter");

    for account in accounts {
        account_info(account);

        if account.owner != program_id {
            msg!("Invalid account owner: {}", account.owner);
            return Err(ProgramError::IncorrectProgramId);
        }

        let mut account_data = account.try_borrow_mut_data()?;

        let counter = account_data
            .first_mut()
            .ok_or(ProgramError::AccountDataTooSmall)?;

        msg!("account_data before: {}", *counter);

        *counter = counter
            .checked_add(1)
            .ok_or(ProgramError::InvalidAccountData)?;

        msg!("account_data after: {}", *counter);
    }

    Ok(())
}

pub fn account_info(
    account: &AccountInfo,
) {
    msg!("------ [main]: print_account_info ------");
    msg!("key: {}", account.key);
    msg!("lamports: {}", account.lamports());
    msg!("account data: {:?}", account.data);
    msg!("owner: {}", account.owner);
    msg!("is_signer: {}", account.is_signer);
    msg!("is_writable: {}", account.is_writable);
    msg!("executable: {}", account.executable);
    msg!("-----------------------------------------");
}

pub fn print_accounts_info(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) {
    msg!("[main]: print_accounts_info");

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


pub fn account_txt(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("[main] account_txt");

    let accounts_iter = &mut accounts.iter();
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_data = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_system = solana_program::account_info::next_account_info(accounts_iter)?; // Program system
    // let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program sysvar rent

    msg!("account_user: {:?}", account_user);
    msg!("account_data: {:?}", account_data);
    msg!("account_system: {:?}", account_system);

    let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(data.len());
    let bump_seed = Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id).1;
    msg!("rent_exemption: {:?}", rent_exemption);
    msg!("bump_seed: {:?}", bump_seed);

    // Data account is not initialized. Create an account and write data into it.
    if **account_data.try_borrow_lamports().unwrap() == 0 {
        solana_program::program::invoke_signed(
            &solana_system_interface::instruction::create_account(
                account_user.key,
                account_data.key,
                rent_exemption,
                data.len() as u64,
                program_id,
            ),
            accounts,
            &[&[&account_user.key.to_bytes(), &[bump_seed]]],
        )?;
        account_data.data.borrow_mut().copy_from_slice(data);
        return Ok(());
    }

    // Fund the data account to let it rent exemption.
    if rent_exemption > account_data.lamports() {
        solana_program::program::invoke(
            &solana_system_interface::instruction::transfer(
                account_user.key,
                account_data.key,
                rent_exemption - account_data.lamports(),
            ),
            accounts,
        )?;
    }
    // Withdraw excess funds and return them to users. Since the funds in the pda account belong to the program, we do
    // not need to use instructions to transfer them here.
    if rent_exemption < account_data.lamports() {
        **account_user.lamports.borrow_mut() = account_user.lamports() + account_data.lamports() - rent_exemption;
        **account_data.lamports.borrow_mut() = rent_exemption;
    }

    msg!("begen account_data.date: {:?}", account_data.data);
    let d = unsafe { *account_data.data.borrow_mut().as_mut_ptr() + 1};

    // Realloc space.
    account_data.resize(data.len())?;
    // Overwrite old data with new data.
    // account_data.data.borrow_mut().copy_from_slice(data);
    account_data.data.borrow_mut().copy_from_slice(&[d]);

    msg!("after account_data.date: {:?}", account_data.data);
    Ok(())
}
