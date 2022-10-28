use anchor_lang::prelude::*;
use anchor_nft_staking::UserStakeInfo;
use anchor_spl::token;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Burn, Mint, MintTo, Token, TokenAccount},
};
pub use switchboard_v2::{
    OracleQueueAccountData, PermissionAccountData, SbState, VrfAccountData, VrfRequestRandomness,
};

pub mod instructions;
use instructions::*;

pub mod error;
use error::*;

pub mod state;
use state::*;


declare_id!("E1QecFyDBGpRczaBjmQ7o2ry2wzAX1MgcrQQajroayWc");

#[program]
pub mod loot_boxes {
    use super::*;

    pub fn init_user(ctx: Context<InitUser>, params: InitUserParams) -> Result<()> {
        InitUser::process_instruction(&ctx, &params)
    }

    pub fn open_lootbox(mut ctx: Context<OpenLootbox>, box_number: u64) -> Result<()> {
        OpenLootbox::process_instruction(&mut ctx, box_number)
    }

    pub fn consume_randomness(mut ctx: Context<ConsumeRandomness>) -> Result<()> {
        ConsumeRandomness::process_instruction(&mut ctx)
    }

    pub fn claim_lootbox(mut ctx: Context<ClaimLootbox>) -> Result<()> {
        ClaimLootbox::process_instruction(&mut ctx)
    }
}