// use std::io::Read;
// use solana_program::account_info::AccountInfo;
// use solana_program::entrypoint::ProgramResult;
// use solana_program::msg;
// use solana_program::program_error::ProgramError;
// use solana_program::pubkey::Pubkey;
// use solana_program::sysvar::Sysvar;
// use crate::utility;
//
// pub fn process_instruction(
//     program_id: &Pubkey,
//     accounts: &[AccountInfo],
//     data: &[u8],
// ) -> ProgramResult {
//     msg!("[my_coin]: process_instruction");
//
//     match data[0] {
//         0x00 => process_instruction_mint(program_id, accounts, &data[1..]),
//         0x01 => process_instruction_transfer(program_id, accounts, &data[1..]),
//         2_u8..=u8::MAX => todo!(),
//     }?;
//
//     Ok(())
// }
//
// fn process_instruction_mint(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
//     msg!("[my_coin]: process_instruction_mint");
//
//     let accounts_iter = &mut accounts.iter();
//     let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
//     let account_data = solana_program::account_info::next_account_info(accounts_iter)?;
//     // let account_system = solana_program::account_info::next_account_info(accounts_iter)?; // Program system
//     // let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program sysvar rent
//
//     utility::show_account_info(account_user);
//     utility::show_account_info(account_data);
//     msg!("begen account_data.date: {:?}", account_data.data);
//
//     let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(data.len());
//     let bump_seed = Pubkey::find_program_address(&[&account_user.key.to_bytes()], program_id).1;
//     msg!("rent_exemption: {:?}", rent_exemption);
//     msg!("bump_seed: {:?}", bump_seed);
//
//     // Data account is not initialized. Create an account and write data into it.
//     if **account_data.try_borrow_lamports().unwrap() == 0 {
//         solana_program::program::invoke_signed(
//             &solana_system_interface::instruction::create_account(
//                 account_user.key,
//                 account_data.key,
//                 rent_exemption,
//                 data.len() as u64,
//                 program_id,
//             ),
//             accounts,
//             &[&[&account_user.key.to_bytes(), &[bump_seed]]],
//         )?;
//         account_data.data.borrow_mut().copy_from_slice(data);
//     }
//
//     // Mint.
//     let mut buf = [0u8; 8];
//     buf.copy_from_slice(&account_data.data.borrow());
//     let old = u64::from_be_bytes(buf);
//     buf.copy_from_slice(&data);
//     let inc = u64::from_be_bytes(buf);
//     let new = old.checked_add(inc).unwrap();
//     account_data.data.borrow_mut().copy_from_slice(&new.to_be_bytes());
//
//     msg!("end account_data.date: {:?}", account_data.data);
//     Ok(())
// }
//
// fn process_instruction_transfer(program_id: &Pubkey, accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
//     msg!("[my_coin]: process_instruction_transfer");
//
//     let accounts_iter = &mut accounts.iter();
//     let account_user = solana_program::account_info::next_account_info(accounts_iter)?;
//     let account_data = solana_program::account_info::next_account_info(accounts_iter)?;
//     let account_into_user = solana_program::account_info::next_account_info(accounts_iter)?;
//     let account_into_data = solana_program::account_info::next_account_info(accounts_iter)?;
//
//     // let account_system = solana_program::account_info::next_account_info(accounts_iter)?; // Program system
//     // let _ = solana_program::account_info::next_account_info(accounts_iter)?; // Program sysvar rent
//
//     utility::show_account_info(account_user);
//     utility::show_account_info(account_data);
//     msg!("begen account_data.date: {:?}", account_data.data);
//
//     utility::show_account_info(account_into_user);
//     utility::show_account_info(account_into_data);
//     msg!("begen account_data.date: {:?}", account_into_data.data);
//
//     let rent_exemption = solana_program::rent::Rent::get()?.minimum_balance(data.len());
//     let bump_seed = Pubkey::find_program_address(&[&account_into_user.key.to_bytes()], program_id).1;
//     msg!("rent_exemption: {:?}", rent_exemption);
//     msg!("bump_seed: {:?}", bump_seed);
//
//     // Data account is not initialized. Create an account and write data into it.
//     if **account_into_data.try_borrow_lamports().unwrap() == 0 {
//         solana_program::program::invoke_signed(
//             &solana_system_interface::instruction::create_account(
//                 account_into_user.key,
//                 account_into_data.key,
//                 rent_exemption,
//                 data.len() as u64,
//                 program_id,
//             ),
//             accounts,
//             &[&[&account_user.key.to_bytes(), &[bump_seed]]],
//         )?;
//         account_data.data.borrow_mut().copy_from_slice(data);
//     }
//
//     // Transfer.
//     let amount = u64::from_be_bytes(data.bytes());
//     let old_user = crate::coin::read_balance(account_data)?;
//     let old_into = crate::coin::read_balance(account_into_data)?;
//     let new_user = old_user
//         .checked_sub())
//         .ok_or(ProgramError::InsufficientFunds)?;
//     let new_into = old_into
//         .checked_add(amount)
//         .ok_or(ProgramError::ArithmeticOverflow)?;
//     crate::coin::write_balance(account_data, new_user)?;
//     crate::coin::write_balance(account_into_data, new_into)?;
//
//     Ok(())
// }
