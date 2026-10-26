use anchor_lang::prelude::*;
use crate::account::*;

pub fn init(ctx: Context<Init>) -> Result<()> {
    let account_user = &ctx.accounts.user;
    let account_user_pda = &mut ctx.accounts.user_pda;
    account_user_pda.auth = account_user.key();
    account_user_pda.bump = ctx.bumps.user_pda;
    account_user_pda.data = Vec::new();
    Ok(())
}

pub fn close(_ctx: Context<Close>) -> Result<()> {
    Ok(())
}

pub fn append(ctx: Context<Append>, mut data:Vec<u8>) -> Result<()> {
    let account_user = &ctx.accounts.user;
    let account_user_pda = &mut ctx.accounts.user_pda;

    account_user_pda.data.append(&mut data);

    msg!("append: {:?}", account_user_pda.data);

    // If the account was shrunk, Anchor won't automatically refund excess lamports. Refund any surplus (over the
    // new rent-exempt minimum) back to the user.
    let account_user_pda_info = account_user_pda.to_account_info();
    let rent_exemption = Rent::get()?.minimum_balance(account_user_pda_info.data_len());
    let hold = **account_user_pda_info.lamports.borrow();
    if hold > rent_exemption {
        let refund = hold.saturating_sub(rent_exemption);
        **account_user_pda_info.lamports.borrow_mut() = rent_exemption;
        **account_user.lamports.borrow_mut() = account_user.lamports().checked_add(refund).unwrap();
    }

    Ok(())
}

pub fn update(ctx: Context<Update>, data: Vec<u8>) -> Result<()> {
    let account_user = &ctx.accounts.user;
    let account_user_pda = &mut ctx.accounts.user_pda;

    // Update the data field with the new data.
    account_user_pda.data = data;

    // If the account was shrunk, Anchor won't automatically refund excess lamports. Refund any surplus (over the
    // new rent-exempt minimum) back to the user.
    let account_user_pda_info = account_user_pda.to_account_info();
    let rent_exemption = Rent::get()?.minimum_balance(account_user_pda_info.data_len());
    let hold = **account_user_pda_info.lamports.borrow();
    if hold > rent_exemption {
        let refund = hold.saturating_sub(rent_exemption);
        **account_user_pda_info.lamports.borrow_mut() = rent_exemption;
        **account_user.lamports.borrow_mut() = account_user.lamports().checked_add(refund).unwrap();
    }
    Ok(())
}