use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{ Approve, Mint, MintTo, Revoke, Token, TokenAccount },
};
use mpl_token_metadata::{
    instruction::{ freeze_delegated_account, thaw_delegated_account },
    ID as METADATA_PROGRAM_ID,
};

declare_id!("AxJjgFUbaQn8ypKgRy56EKco2cQ95oGEN7oJUdo7VCUY");

#[program]
pub mod anchor_nft_staking {
    use super::*;

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        require!(ctx.accounts.stake_state.stake_state == StakeState::Unstaked, StakeError::AlreadyStaked);

        let clock = Clock::get().unwrap();

        msg!("Approving delegate"); 
        let cpi_approve_program = ctx.accounts.token_program.to_account_info();
        let cpi_approve_accounts = Approve {
            to: ctx.accounts.nft_token_account.to_account_info(),
            delegate: ctx.accounts.program_authority.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_approve_context = CpiContext::new(cpi_approve_program, cpi_approve_accounts);
        token::approve(cpi_approve_context, 1)?;

        msg!("Freezing delegated account");
        let authority_bump = *ctx.bumps.get("program_authority").unwrap();
        
        invoke_signed(
            &mpl_token_metadata::instruction::freeze_delegated_account(
                ctx.accounts.metadata_program.key(),
                ctx.accounts.program_authority.key(),
                ctx.accounts.nft_token_account.key(),
                ctx.accounts.nft_edition.key(),
                ctx.accounts.nft_mint.key(),
            ),
            &[
                ctx.accounts.metadata_program.to_account_info(),
                ctx.accounts.program_authority.to_account_info(),
                ctx.accounts.nft_token_account.to_account_info(),
                ctx.accounts.nft_edition.to_account_info(),
                ctx.accounts.nft_mint.to_account_info(),
            ],
            &[&[b"authority", &[authority_bump]]],
        )?;        

        ctx.accounts.stake_state.token_account = ctx.accounts.nft_token_account.key();
        ctx.accounts.stake_state.user_pubkey = ctx.accounts.user.key();
        ctx.accounts.stake_state.stake_state = StakeState::Staked;
        ctx.accounts.stake_state.stake_start_time = clock.unix_timestamp;
        ctx.accounts.stake_state.last_reddem_time = clock.unix_timestamp;
        ctx.accounts.stake_state.is_initialized = true;

        Ok(())
    }

    pub fn redeem(ctx: Context<Redeem>) -> Result<()> {
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut, 
        associated_token::mint = nft_mint, 
        associated_token::authority = user
    )]
    pub nft_token_account: Account<'info, TokenAccount>,
    pub nft_mint: Account<'info, Mint>,
    #[account(
        owner = METADATA_PROGRAM_ID,
    )]
    /// CHECK: Manual validation
    pub nft_edition: UncheckedAccount<'info>,
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + std::mem::size_of::<UserStakeInfo>(),
        seeds = [user.key().as_ref(), nft_token_account.key().as_ref()],
        bump 
    )]
    pub stake_state: Account<'info, UserStakeInfo>,
    #[account( mut, seeds = ["authority".as_bytes().as_ref()], bump )]    
    /// CHECK: Manual validation
    pub program_authority: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub metadata_program: Program<'info, Metadata>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Redeem {}

#[derive(Accounts)]
pub struct Unstake {}

#[derive(Clone)]
pub struct Metadata;


impl anchor_lang::Id for Metadata {
    fn id() -> Pubkey {
        METADATA_PROGRAM_ID
    }
}


#[account]
pub struct UserStakeInfo {
    pub token_account: Pubkey,
    pub stake_start_time: i64,
    pub last_reddem_time: i64,
    pub user_pubkey: Pubkey,
    pub stake_state: StakeState,
    pub is_initialized: bool,
}

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum StakeState {
    Unstaked,
    Staked
}

impl Default for StakeState {
    fn default() -> Self {
        StakeState::Unstaked
    }
}

#[error_code]
pub enum StakeError {
    #[msg("NFT is already in stake")]
    AlreadyStaked
}