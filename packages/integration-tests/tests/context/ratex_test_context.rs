#![allow(clippy::too_many_arguments)]
#![allow(dead_code)]

use anchor_client::solana_sdk::signature::Keypair;
use anchor_client::solana_sdk::signature::Signer;
use anchor_lang::prelude::{Clock, Pubkey};
use solana_program_test::ProgramTestContext;
use std::{cell::RefCell, rc::Rc};

use crate::utilities::helper::{
    create_payer_from_file, get_context, get_or_create_associated_token_address, get_sysvar_clock,
    spl_token_mint, transfer,
};

use crate::utilities::kamino::JITOSOL_MINT;
use anchor_spl::token::spl_token;

use super::UserTestContext;

pub enum MintType {
    JitoSol,
}

pub struct RateXTestContext {
    pub context: Rc<RefCell<ProgramTestContext>>,
    pub admin: Keypair,
    pub users: Vec<UserTestContext>,
}

#[allow(dead_code)]
impl RateXTestContext {
    pub async fn new() -> RateXTestContext {
        let context = get_context().await;

        let admin =
            create_payer_from_file(&mut context.borrow_mut(), "tests/fixtures/admin.json").await;

        // Initialize users
        let mut users: Vec<UserTestContext> = vec![];
        for _ in 0..8 {
            let user = UserTestContext::new(context.clone()).await;
            users.push(user);
        }

        RateXTestContext {
            context,
            admin,
            users,
        }
    }

    pub async fn get_clock(&self) -> Clock {
        let context = &mut self.context.borrow_mut();

        get_sysvar_clock(&mut context.banks_client).await
    }

    pub async fn warp_to_slot(&self) {
        let clock: Clock = self.get_clock().await;

        self.context
            .borrow_mut()
            .warp_to_slot(clock.slot + 1)
            .unwrap();
    }

    pub async fn mint_token(
        &self,
        mint: &Pubkey,
        mint_authority: &Keypair,
        user: &Keypair,
        amount: u64,
    ) {
        if *mint == spl_token::native_mint::id() {
            transfer(&mut self.context.borrow_mut(), &user.pubkey(), amount).await;
            return;
        }

        let context: &mut ProgramTestContext = &mut self.context.borrow_mut();
        let user_mint_acc =
            get_or_create_associated_token_address(context, &user, &user.pubkey(), mint).await;

        spl_token_mint(
            context,
            &self.admin,
            mint,
            &user_mint_acc,
            mint_authority,
            amount,
        )
        .await
        .unwrap();
    }

    pub async fn mint_token_by_type(
        &self,
        utc: &UserTestContext,
        amount: u64,
        mint_type: MintType,
    ) {
        let mint_address = match mint_type {
            MintType::JitoSol => &JITOSOL_MINT,
        };

        self.mint_token(mint_address, &self.admin, &utc.user, amount)
            .await;
    }

    pub async fn set_sysvar_clock(&self, time: i64) {
        let mut clock: Clock = get_sysvar_clock(&mut self.context.borrow_mut().banks_client).await;
        // println!("clock: {:?}", clock);
        // println!("time: {}", now());

        clock.epoch_start_timestamp = time;
        clock.unix_timestamp = time;
        clock.slot = time as u64;

        self.context.borrow_mut().set_sysvar::<Clock>(&clock);
    }
}
