use crate::state::*;
use crate::*;

#[derive(Accounts)]
pub struct ConsumeRandomness<'info> {
    #[account(
        mut,
        // TESTING - Comment out these seeds for testing
        // seeds = [
        //     payer.key().as_ref(),
        // ],
        // TESTING - Uncomment these seeds for testing
        seeds = [
            vrf.key().as_ref(),
            payer.key().as_ref()
        ],
        bump = state.load()?.bump,
        has_one = vrf @ LootboxError::InvalidVrfAccount
    )]
    pub state: AccountLoader<'info, UserState>,
    pub vrf: AccountLoader<'info, VrfAccountData>,
    #[account(
        mut,
        seeds=["lootbox".as_bytes(), payer.key().as_ref()],
        bump
      )]
    pub lootbox_pointer: Account<'info, LootboxPointer>,
    /// CHECK: ...
    pub payer: AccountInfo<'info>,
}

impl ConsumeRandomness<'_> {
    pub fn process_instruction(ctx: &mut Context<Self>) -> Result<()> {
        let vrf = ctx.accounts.vrf.load()?;
        let state = &mut ctx.accounts.state.load_mut()?;

        let result_buffer = vrf.get_result()?;
        if result_buffer == [0u8; 32] {
            msg!("vrf buffer empty");
            return Ok(());
        }

        if result_buffer == state.result_buffer {
            msg!("result_buffer unchanged");
            return Ok(());
        }

        let available_gear: Vec<Pubkey> = Self::AVAILABLE_GEAR
            .into_iter()
            .map(|key| key.parse::<Pubkey>().unwrap())
            .collect();

        // maximum value to convert randomness buffer
        let max_result = available_gear.len();
        let value: &[u8] = bytemuck::cast_slice(&result_buffer[..]);
        let i = (value[0] as usize) % max_result;
        msg!("The chosen mint index is {} out of {}", i, max_result);

        let mint = available_gear[i];
        msg!("Next mint is {:?}", mint);
        ctx.accounts.lootbox_pointer.mint = mint;
        ctx.accounts.lootbox_pointer.redeemable = true;

        Ok(())
    }

    const AVAILABLE_GEAR: [&'static str; 5] = [
        "CudtPyZYtPrwJufP6k7gAzPYfV12CLdpu4MHXfLSig9",
        "E6cg8XYnor3g58o2qP7N2DHV9JNDJgwpNDAh7QrSw72w",
        "CeMJNozciBS8Ke9Y7saYbJeeaq2vSoEptgXSvoMBq1Q3",
        "H6fpRxytdX6MpBAffyxMitsqoyhHqE71SwNPsEvt1oQ2",
        "Hpvp9ZuFnK1E5aBxVM3thGqsybGNBTf6ucXT2ZEX7EPr",
    ];
}