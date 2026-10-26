use anchor_lang::account;
use anchor_lang::prelude::*;

#[account]
pub struct Data {
    pub auth: Pubkey,  // Authority public key; only this key can update.
    pub bump: u8,      // PDA bump value ensuring unique address.
    pub data: Vec<u8>, // Business data with variable length.
}

impl Data {
    pub fn space_for(data_len: usize) -> usize {
        // 8 (discriminator) + 32 (authority) + 1 (bump) + 4 (vec len) + data_len.
        8 + 32 + 1 + 4 + data_len
    }
}