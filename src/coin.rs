#![allow(unexpected_cfgs)]

use std::str::FromStr;

use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use solana_program::{msg, program, rent};

// 铸币权限。当前用本机 payer 的地址方便 devnet 测试；
// 想换回 Ada，把这里改成她的地址即可。
const MINT_AUTHORITY: &str = "FmZvpz8Nub192NdoWDTA8MJfcPQvrqPR2dZdXhzZj3N9";

// 给 coin 的 PDA 加命名空间，避免和旧的 counter 程序（种子只有 user）撞账户
const SEED_PREFIX: &[u8] = b"coin";

// 余额固定占 8 字节，u64 大端
const ACCOUNT_SPACE: usize = 8;

fn read_amount(data: &[u8]) -> Result<u64, ProgramError> {
    if data.len() != ACCOUNT_SPACE {
        return Err(ProgramError::InvalidInstructionData);
    }
    let mut buf = [0u8; ACCOUNT_SPACE];
    buf.copy_from_slice(data);
    Ok(u64::from_be_bytes(buf))
}

fn read_balance(account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = account.try_borrow_data()?;
    if data.len() != ACCOUNT_SPACE {
        return Err(ProgramError::InvalidAccountData);
    }
    let mut buf = [0u8; ACCOUNT_SPACE];
    buf.copy_from_slice(&data);
    Ok(u64::from_be_bytes(buf))
}

fn write_balance(account: &AccountInfo, balance: u64) -> Result<(), ProgramError> {
    account
        .try_borrow_mut_data()?
        .copy_from_slice(&balance.to_be_bytes());
    Ok(())
}

pub fn process_instruction_mint(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("[coin]: process_instruction_mint");

    let amount = read_amount(data)?;

    let accounts_iter = &mut accounts.iter();
    let account_user = next_account_info(accounts_iter)?;
    let account_user_pda = next_account_info(accounts_iter)?;
    let _ = next_account_info(accounts_iter)?; // Program system
    let _ = next_account_info(accounts_iter)?; // Program sysvar rent

    if !account_user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Only the hardcoded authority can mint.
    let authority = Pubkey::from_str(MINT_AUTHORITY).map_err(|_| ProgramError::InvalidArgument)?;
    if *account_user.key != authority {
        msg!("[coin]: {} is not the mint authority", account_user.key);
        return Err(ProgramError::InvalidArgument);
    }

    let user_key_bytes = account_user.key.to_bytes();
    let user_seeds: &[&[u8]] = &[SEED_PREFIX, &user_key_bytes];
    let (expected_pda, bump_seed) = Pubkey::find_program_address(user_seeds, program_id);
    if *account_user_pda.key != expected_pda {
        return Err(ProgramError::InvalidArgument);
    }

    // Data account is not initialized. Create an account and write data into it.
    if **account_user_pda.try_borrow_lamports()? == 0 {
        let rent_exemption = rent::Rent::get()?.minimum_balance(ACCOUNT_SPACE);
        let signer_seeds: &[&[u8]] = &[SEED_PREFIX, &user_key_bytes, &[bump_seed]];
        program::invoke_signed(
            &solana_system_interface::instruction::create_account(
                account_user.key,
                account_user_pda.key,
                rent_exemption,
                ACCOUNT_SPACE as u64,
                program_id,
            ),
            accounts,
            &[signer_seeds],
        )?;
    }

    // Mint.
    let old = read_balance(account_user_pda)?;
    let new = old
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    write_balance(account_user_pda, new)?;
    Ok(())
}

pub fn process_instruction_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("[coin]: process_instruction_transfer");

    let amount = read_amount(data)?;

    let accounts_iter = &mut accounts.iter();
    let account_user = next_account_info(accounts_iter)?;
    let account_user_pda = next_account_info(accounts_iter)?;
    let account_into = next_account_info(accounts_iter)?;
    let account_into_pda = next_account_info(accounts_iter)?;
    let _ = next_account_info(accounts_iter)?; // Program system
    let _ = next_account_info(accounts_iter)?; // Program sysvar rent

    if !account_user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    // 自己转给自己时两个 PDA 是同一个账户，先后写入会把余额算成 old + inc
    if account_user.key == account_into.key {
        return Err(ProgramError::InvalidArgument);
    }

    let user_key_bytes = account_user.key.to_bytes();
    let user_seeds: &[&[u8]] = &[SEED_PREFIX, &user_key_bytes];
    let (expected_pda, _) = Pubkey::find_program_address(user_seeds, program_id);
    if *account_user_pda.key != expected_pda {
        return Err(ProgramError::InvalidArgument);
    }

    let into_key_bytes = account_into.key.to_bytes();
    let into_seeds: &[&[u8]] = &[SEED_PREFIX, &into_key_bytes];
    let (expected_into_pda, bump_seed) = Pubkey::find_program_address(into_seeds, program_id);
    if *account_into_pda.key != expected_into_pda {
        return Err(ProgramError::InvalidArgument);
    }

    // 目标 PDA 不存在时顺手创建
    if **account_into_pda.try_borrow_lamports()? == 0 {
        let rent_exemption = rent::Rent::get()?.minimum_balance(ACCOUNT_SPACE);
        let signer_seeds: &[&[u8]] = &[SEED_PREFIX, &into_key_bytes, &[bump_seed]];
        program::invoke_signed(
            &solana_system_interface::instruction::create_account(
                account_user.key,
                account_into_pda.key,
                rent_exemption,
                ACCOUNT_SPACE as u64,
                program_id,
            ),
            accounts,
            &[signer_seeds],
        )?;
    }

    // Transfer.
    let old_user = read_balance(account_user_pda)?;
    let old_into = read_balance(account_into_pda)?;
    let new_user = old_user
        .checked_sub(amount)
        .ok_or(ProgramError::InsufficientFunds)?;
    let new_into = old_into
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    write_balance(account_user_pda, new_user)?;
    write_balance(account_into_pda, new_into)?;
    Ok(())
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    msg!("[coin]: process_instruction");
    match data.first().copied() {
        Some(0x00) => process_instruction_mint(program_id, accounts, &data[1..]),
        Some(0x01) => process_instruction_transfer(program_id, accounts, &data[1..]),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
