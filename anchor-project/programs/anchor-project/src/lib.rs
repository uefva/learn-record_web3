use anchor_lang::prelude::*;

mod counter;
mod account;

use counter::*;

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

    pub fn update(ctx: Context<Update>, data: Vec<u8>) -> Result<()> {
        counter::update(ctx, data)
    }
}