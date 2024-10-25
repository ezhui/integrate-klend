#![allow(clippy::too_many_arguments)]
#![allow(dead_code)]

use crate::utilities::helper::{
    create_token_account, create_user, get_associated_token_address, get_keypair,
    get_or_create_associated_token_address, get_sysvar_clock, get_token_balance,
    process_instructions,
};
use crate::utilities::kamino::{
    compose_klend_borrow_obligation_liquidity_ix, compose_klend_deposit_obligation_collateral_ix,
    compose_klend_deposit_reserve_liquidity_ix, compose_klend_init_obligation_farms_for_reserve_ix,
    compose_klend_init_obligation_ix, compose_klend_init_user_metadata_ix,
    compose_klend_refresh_obligation_ix, compose_klend_refresh_reserve_ix, JITOSOL_MINT,
    KLEND_PROGRAM_ID, MAIN_MARKET, MAIN_MARKET_AUTHORITY, RESERVE_JITOSOL_COLLATERAL_MINT,
    RESERVE_JITOSOL_COLLATERAL_SUPPLY_VAULT, RESERVE_JITOSOL_LIQUIDITY_SUPPLY_VAULT,
    RESERVE_JITOSOL_STATE, RESERVE_SOL_COLLATERAL_MINT, RESERVE_SOL_FARM_STATE,
    RESERVE_SOL_LIQUIDITY_FEE_VAULT, RESERVE_SOL_LIQUIDITY_MINT,
    RESERVE_SOL_LIQUIDITY_SUPPLY_VAULT, RESERVE_SOL_STATE, RESERVE_USDC_LIQUIDITY_FEE_VAULT,
    RESERVE_USDC_LIQUIDITY_MINT, RESERVE_USDC_LIQUIDITY_SUPPLY_VAULT, RESERVE_USDC_STATE,
};
use anchor_client::solana_sdk::instruction::Instruction;
use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signature::Signer;
use anchor_lang::prelude::Pubkey;
use anchor_spl::token::spl_token;
use solana_program_test::ProgramTestContext;
use std::{cell::RefCell, rc::Rc};

pub struct UserTestContext {
    pub context: Rc<RefCell<ProgramTestContext>>,
    pub admin: Keypair,
    pub user: Keypair,
}

impl UserTestContext {
    pub async fn new(context: Rc<RefCell<ProgramTestContext>>) -> UserTestContext {
        let admin = get_keypair("tests/fixtures/admin.json").await;

        let user = create_user(&mut context.borrow_mut()).await;

        UserTestContext {
            context,
            admin,
            user,
        }
    }

    pub async fn assert_mint_balance(&self, mint: Pubkey, expect: u64) {
        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();

        let account = get_associated_token_address(&self.user.pubkey(), &mint).await;
        let balance = get_token_balance(&mut context.banks_client, account).await;

        assert_eq!(balance, expect);
    }

    pub async fn klend_init_user_metadata(&self) {
        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();

        let (user_metadata, _) = Pubkey::find_program_address(
            &[b"user_meta", &self.user.pubkey().to_bytes()],
            &KLEND_PROGRAM_ID,
        );

        let clock = get_sysvar_clock(&mut context.banks_client).await;

        let (_, user_lookup_table) =
            solana_address_lookup_table_program::instruction::create_lookup_table(
                self.user.pubkey(),
                self.user.pubkey(),
                clock.slot,
            );

        let instruction = compose_klend_init_user_metadata_ix(
            &self.user.pubkey(),
            &self.user.pubkey(),
            &user_metadata,
            &user_lookup_table,
        );

        process_instructions(context, &self.user, &vec![instruction]).await;
    }

    pub async fn klend_init_obligation(&self, market: &Pubkey, tag: u8, id: u8) -> Pubkey {
        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();

        let seed_account = Pubkey::default();
        let (obligation, _) = Pubkey::find_program_address(
            &[
                &[tag],
                &[id],
                &self.user.pubkey().to_bytes(),
                &market.to_bytes(),
                &seed_account.to_bytes(),
                &seed_account.to_bytes(),
            ],
            &KLEND_PROGRAM_ID,
        );

        let (user_metadata, _) = Pubkey::find_program_address(
            &[b"user_meta", &self.user.pubkey().to_bytes()],
            &KLEND_PROGRAM_ID,
        );

        let instruction = compose_klend_init_obligation_ix(
            &self.user.pubkey(),
            &self.user.pubkey(),
            &obligation,
            market,
            &seed_account,
            &seed_account,
            &user_metadata,
            tag,
            id,
        );

        process_instructions(context, &self.user, &vec![instruction]).await;

        obligation
    }

    pub async fn klend_refresh_reserve(&self, reserve: &Pubkey) {
        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();

        let instruction = compose_klend_refresh_reserve_ix(reserve, &MAIN_MARKET);

        process_instructions(context, &self.user, &vec![instruction]).await;
    }

    pub async fn klend_refresh_obligation(&self, obligation: &Pubkey, reserves: &Vec<Pubkey>) {
        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();

        let instruction = compose_klend_refresh_obligation_ix(obligation, &MAIN_MARKET, reserves);

        process_instructions(context, &self.user, &vec![instruction]).await;
    }

    pub async fn klend_deposit_reserve_jitosol_liquidity(&self, liquidity_amount: u64) {
        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();

        let (lending_market_authority, _) =
            Pubkey::find_program_address(&[b"lma", &MAIN_MARKET.to_bytes()], &KLEND_PROGRAM_ID);

        println!("Main market authority: {}", lending_market_authority);

        let user_source_liquidity = get_or_create_associated_token_address(
            context,
            &self.user,
            &self.user.pubkey(),
            &JITOSOL_MINT,
        )
        .await;
        let user_destination_collateral = get_or_create_associated_token_address(
            context,
            &self.user,
            &self.user.pubkey(),
            &RESERVE_JITOSOL_COLLATERAL_MINT,
        )
        .await;

        let instruction = compose_klend_deposit_reserve_liquidity_ix(
            &self.user.pubkey(),
            &RESERVE_JITOSOL_STATE,
            &MAIN_MARKET,
            &lending_market_authority,
            &JITOSOL_MINT,
            &RESERVE_JITOSOL_LIQUIDITY_SUPPLY_VAULT,
            &RESERVE_JITOSOL_COLLATERAL_MINT,
            &user_source_liquidity,
            &user_destination_collateral,
            liquidity_amount,
        );

        process_instructions(context, &self.user, &vec![instruction]).await;
    }

    pub async fn klend_deposit_reserve_sol_liquidity(&self, liquidity_amount: u64) {
        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();

        let (lending_market_authority, _) =
            Pubkey::find_program_address(&[b"lma", &MAIN_MARKET.to_bytes()], &KLEND_PROGRAM_ID);

        println!("Main market authority: {}", lending_market_authority);

        let user_wsol_acc = Keypair::new();

        create_token_account(
            context,
            &self.user,
            &user_wsol_acc,
            &spl_token::native_mint::id(),
            &self.user.pubkey(),
            liquidity_amount,
        )
        .await
        .unwrap();

        let user_destination_collateral = get_or_create_associated_token_address(
            context,
            &self.user,
            &self.user.pubkey(),
            &RESERVE_SOL_COLLATERAL_MINT,
        )
        .await;

        let mut instructions: Vec<Instruction> = vec![];

        instructions.push(compose_klend_refresh_reserve_ix(
            &RESERVE_SOL_STATE,
            &MAIN_MARKET,
        ));

        instructions.push(compose_klend_deposit_reserve_liquidity_ix(
            &self.user.pubkey(),
            &RESERVE_SOL_STATE,
            &MAIN_MARKET,
            &lending_market_authority,
            &RESERVE_SOL_LIQUIDITY_MINT,
            &RESERVE_SOL_LIQUIDITY_SUPPLY_VAULT,
            &RESERVE_SOL_COLLATERAL_MINT,
            &user_wsol_acc.pubkey(),
            &user_destination_collateral,
            liquidity_amount,
        ));

        let close_wsol_account_ix = spl_token::instruction::close_account(
            &spl_token::id(),
            &user_wsol_acc.pubkey(),
            &self.user.pubkey(),
            &self.user.pubkey(),
            &[&self.user.pubkey()],
        )
        .unwrap();

        instructions.push(close_wsol_account_ix);

        process_instructions(context, &self.user, &instructions).await;
    }

    pub async fn klend_deposit_obligation_collateral(&self, obligation: &Pubkey) {
        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();

        let user_source_collateral =
            get_associated_token_address(&self.user.pubkey(), &RESERVE_JITOSOL_COLLATERAL_MINT)
                .await;
        let collateral_amount =
            get_token_balance(&mut context.banks_client, user_source_collateral).await;

        println!(
            "deposit obligation collateral amount: {}",
            collateral_amount
        );

        let mut instructions: Vec<Instruction> = vec![];
        instructions.push(compose_klend_refresh_reserve_ix(
            &RESERVE_JITOSOL_STATE,
            &MAIN_MARKET,
        ));

        instructions.push(compose_klend_refresh_obligation_ix(
            obligation,
            &MAIN_MARKET,
            &vec![],
        ));

        instructions.push(compose_klend_deposit_obligation_collateral_ix(
            &self.user.pubkey(),
            obligation,
            &MAIN_MARKET,
            &RESERVE_JITOSOL_STATE,
            &RESERVE_JITOSOL_COLLATERAL_SUPPLY_VAULT,
            &user_source_collateral,
            collateral_amount,
        ));

        process_instructions(context, &self.user, &instructions).await;

        let collateral_amount =
            get_token_balance(&mut context.banks_client, user_source_collateral).await;
        assert_eq!(collateral_amount, 0);
    }

    pub async fn klend_borrow_obligation_liquidity(
        &self,
        obligation: &Pubkey,
        mint: &str,
        liquidity_amount: u64,
    ) {
        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();

        let mut instructions: Vec<Instruction> = vec![];
        instructions.push(compose_klend_refresh_reserve_ix(
            &RESERVE_JITOSOL_STATE,
            &MAIN_MARKET,
        ));

        match mint {
            "SOL" => {
                instructions.push(compose_klend_refresh_reserve_ix(
                    &RESERVE_SOL_STATE,
                    &MAIN_MARKET,
                ));
            }
            "USDC" => {
                instructions.push(compose_klend_refresh_reserve_ix(
                    &RESERVE_USDC_STATE,
                    &MAIN_MARKET,
                ));
            }
            _ => panic!("not support"),
        }

        instructions.push(compose_klend_refresh_obligation_ix(
            obligation,
            &MAIN_MARKET,
            &vec![RESERVE_JITOSOL_STATE],
        ));

        match mint {
            "SOL" => {
                let user_destination_liquidity = get_or_create_associated_token_address(
                    context,
                    &self.user,
                    &self.user.pubkey(),
                    &RESERVE_SOL_LIQUIDITY_MINT,
                )
                .await;

                instructions.push(compose_klend_borrow_obligation_liquidity_ix(
                    &self.user.pubkey(),
                    obligation,
                    &MAIN_MARKET,
                    &MAIN_MARKET_AUTHORITY,
                    &RESERVE_SOL_STATE,
                    &RESERVE_SOL_LIQUIDITY_MINT,
                    &RESERVE_SOL_LIQUIDITY_SUPPLY_VAULT,
                    &RESERVE_SOL_LIQUIDITY_FEE_VAULT,
                    &user_destination_liquidity,
                    liquidity_amount,
                ));
            }
            "USDC" => {
                let user_destination_liquidity = get_or_create_associated_token_address(
                    context,
                    &self.user,
                    &self.user.pubkey(),
                    &RESERVE_USDC_LIQUIDITY_MINT,
                )
                .await;

                instructions.push(compose_klend_borrow_obligation_liquidity_ix(
                    &self.user.pubkey(),
                    obligation,
                    &MAIN_MARKET,
                    &MAIN_MARKET_AUTHORITY,
                    &RESERVE_USDC_STATE,
                    &RESERVE_USDC_LIQUIDITY_MINT,
                    &RESERVE_USDC_LIQUIDITY_SUPPLY_VAULT,
                    &RESERVE_USDC_LIQUIDITY_FEE_VAULT,
                    &user_destination_liquidity,
                    liquidity_amount,
                ));
            }
            _ => panic!("not support"),
        }

        process_instructions(context, &self.user, &instructions).await;
    }

    pub async fn klend_init_obligation_farms_for_reserve(
        &self,
        obligation: &Pubkey,
        reserve_name: &str,
    ) {
        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();

        let (reserve, reserve_farm_state) = match reserve_name {
            "SOL" => (&RESERVE_SOL_STATE, &RESERVE_SOL_FARM_STATE),
            // "USDC" => (&RESERVE_USDC_STATE, &RESERVE_USDC_FARM_STATE),
            // "JITOSOL" => (&RESERVE_JITOSOL_STATE, &RESERVE_JITOSOL_FARM_STATE),
            _ => panic!("not support"),
        };

        let (obligation_farm, _) = Pubkey::find_program_address(
            &[
                b"user",
                &reserve_farm_state.to_bytes(),
                &obligation.to_bytes(),
            ],
            &KLEND_PROGRAM_ID,
        );

        let instruction = compose_klend_init_obligation_farms_for_reserve_ix(
            &self.user.pubkey(),
            &self.user.pubkey(),
            obligation,
            &MAIN_MARKET_AUTHORITY,
            reserve,
            reserve_farm_state,
            &obligation_farm,
            &MAIN_MARKET,
            0,
        );

        process_instructions(context, &self.user, &vec![instruction]).await;
    }
}
