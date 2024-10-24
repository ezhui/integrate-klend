#![cfg(test)]

mod context;
mod utilities;

use context::{MintType, RateXTestContext};
use solana_program_test::*;
use utilities::kamino::{JITOSOL_MINT, MAIN_MARKET};

#[tokio::test]
async fn test_kamino() {
    let rtc = RateXTestContext::new().await;
    let alice = &rtc.users[0];

    rtc.set_sysvar_clock(1729752082).await; // KAMINO_SCOPE_PRICES

    alice.klend_init_user_metadata().await;
    let obligation = alice.klend_init_obligation(&MAIN_MARKET, 0, 0).await;
    alice.klend_refresh_reserve().await;
    alice.klend_refresh_obligation(&obligation, &vec![]).await;

    rtc.mint_token_by_type(alice, 500_000_000, MintType::JitoSol)
        .await;
    alice.assert_mint_balance(JITOSOL_MINT, 500_000_000).await;

    alice
        .klend_deposit_reserve_jitosol_liquidity(500_000_000)
        .await;

    alice.klend_deposit_obligation_collateral(&obligation).await;

    // Fail to borrow liquidity ...
    alice
        .klend_borrow_obligation_liquidity(&obligation, "SOL", 100_000_000)
        .await;
}
