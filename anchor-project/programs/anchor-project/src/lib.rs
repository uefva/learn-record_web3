use anchor_lang::prelude::*;

mod counter;
mod account;
mod err;
mod data;

use account::*;

declare_id!("2sPwcBjsWBRnLZEShoYWCRuY8PAat9HGGsUHhjf3VXZK");

#[program]
pub mod pxsol_ss_anchor {
    use super::*;

    pub fn init(ctx: Context<Init>) -> Result<()> {
        counter::init(ctx)
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        counter::close(ctx)
    }

    pub fn append(ctx: Context<Append>, data: Vec<u8>) -> Result<()> {
        counter::append(ctx, data)
    }

    pub fn update(ctx: Context<Update>, data: Vec<u8>) -> Result<()> {
        counter::update(ctx, data)
    }

    pub fn init_mint(ctx: Context<CreateMint>) -> Result<()>{
            counter::init_mint(ctx)
    }

    pub fn init_token(ctx: Context<CreateToken>) -> Result<()>{
        counter::init_token(ctx)
    }
}