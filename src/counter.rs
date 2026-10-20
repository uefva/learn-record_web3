use solana_program::program_error::ProgramError;
use solana_program::sysvar::Sysvar;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};
use crate::utility;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("[main]: Hello, Web3!");

    msg!("program_id: {}", program_id);

    account_counter(program_id, accounts, data)?;

    Ok(())
}

pub fn account_counter(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("[main] account_counter");

    let accounts_iter = &mut accounts.iter();
    let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_data = solana_program::account_info::next_account_info(accounts_iter)?;
    let account_system = solana_program::account_info::next_account_info(accounts_iter)?; // Program system
    // let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program sysvar rent

    utility::show_account_info(account_user);
    utility::show_account_info(account_data);
    utility::show_account_info(account_system);
    msg!("begen account_data.date: {:?}", account_data.data);

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
        **account_user.lamports.borrow_mut() =
            account_user.lamports() + account_data.lamports() - rent_exemption;
        **account_data.lamports.borrow_mut() = rent_exemption;
    }

    let mut tmp = *account_data
        .data
        .borrow()
        .first()
        .ok_or(ProgramError::InvalidAccountData)?;
    tmp += 1;

    // Realloc space.
    account_data.resize(size_of_val(&tmp))?;
    // Overwrite old data with new data.
    // account_data.data.borrow_mut().copy_from_slice(data);
    account_data.data.borrow_mut()[0] = tmp;

    utility::show_account_info(account_user);
    utility::show_account_info(account_data);
    utility::show_account_info(account_system);
    msg!("begen account_data.date: {:?}", account_data.data);
    Ok(())
}
