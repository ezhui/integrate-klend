use anchor_lang::prelude::*;

declare_id!("FmWaHzC2q91qJEX1CHsnUh97d5keajKJwWxNEdNiqUfV");

#[program]
pub mod integrate_klend {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
