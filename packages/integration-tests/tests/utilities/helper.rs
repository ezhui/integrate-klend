#![allow(dead_code)]
use std::cell::RefCell;

use super::kamino::{
    load_kamino_fixtures, JITOSOL_MINT, RESERVE_JITOSOL_COLLATERAL_MINT,
    RESERVE_SOL_COLLATERAL_MINT, RESERVE_USDC_LIQUIDITY_MINT,
};
use anchor_client::solana_sdk::program_option::COption;
use anchor_lang::prelude::Clock;
use anchor_lang::prelude::Pubkey;
use anchor_spl::token::{spl_token, spl_token::state::Mint};
use spl_associated_token_account;

use solana_program_test::{find_file, read_file, BanksClient, ProgramTest, ProgramTestContext};
use std::time::{SystemTime, UNIX_EPOCH};

use anchor_lang::solana_program::pubkey;

use {
    // anchor client must be imported with "async" feature
    anchor_client::{
        solana_sdk::{
            account::Account, commitment_config::CommitmentConfig, instruction::Instruction,
            program_pack::Pack, signature::read_keypair_file, signature::Keypair,
            signature::Signer, system_instruction, sysvar, transaction::Transaction,
            transport::TransportError,
        },
        Client, Cluster, Program,
    },
    std::rc::Rc,
};

pub const SSOL_MINT: Pubkey = pubkey!("sSo14endRuUbvQaJS3dq36Q829a3A6BEfoeeRGJywEh");
pub const ENDOAVS_SSOL_VAULT: Pubkey = pubkey!("Bc7hj6aFhBRihZ8dYp8qXWbuDBXYMya4dzFGmHezLnB7");
pub const ENDOAVS_SONIC: Pubkey = pubkey!("HBkJwH6rjUUBK1wNhBuYgo9Wnk1iCx2phduyxWCQj6uk");
pub const SONIC_SSOL: Pubkey = pubkey!("sonickAJFiVLcYXx25X9vpF293udaWqDMUCiGtk7dg2");

pub const ENDOAVS_PROGRAM_ID: Pubkey = pubkey!("endoLNCKTqDn8gSVnN2hDdpgACUPWHZTwoYnnMybpAT");

pub fn get_program(pid: Pubkey) -> Program<Rc<Keypair>> {
    let client = Client::new_with_options(
        Cluster::Debug,
        Rc::new(Keypair::new()),
        CommitmentConfig::processed(),
    );
    client.program(pid).unwrap()
}

pub fn add_mint(
    pt: &mut ProgramTest,
    mint_address: Pubkey,
    mint_account_file: &str,
    mint_authority: Option<Pubkey>,
) {
    let mint_data = read_file(find_file(mint_account_file).unwrap_or_else(|| {
        panic!("Unable to load {}", mint_account_file);
    }));

    let mut mint = Mint::unpack_from_slice(&mint_data).unwrap();
    if let Some(auth) = mint_authority {
        mint.mint_authority = COption::Some(auth);
    }
    let mut fixed_data: Vec<u8> = vec![0; 82];
    Mint::pack(mint, &mut fixed_data).unwrap();

    pt.add_account(
        mint_address,
        Account {
            lamports: 1461600,
            data: fixed_data,
            owner: spl_token::id(),
            executable: false,
            rent_epoch: 0,
        },
    );
}

pub async fn get_context() -> Rc<RefCell<ProgramTestContext>> {
    let mut pt = ProgramTest::default();
    pt.add_program("endoavs", ENDOAVS_PROGRAM_ID, None);

    let admin = read_keypair_file("tests/fixtures/admin.json").unwrap();

    add_mint(
        &mut pt,
        JITOSOL_MINT,
        "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn.bin",
        Some(admin.pubkey()), // take over the mint authority
    );
    add_mint(
        &mut pt,
        RESERVE_JITOSOL_COLLATERAL_MINT,
        "9ucQp7thL38MDDTSER5ou24QnVSTZFLevDsZC1cAFkKy.bin",
        None,
    );
    add_mint(
        &mut pt,
        RESERVE_USDC_LIQUIDITY_MINT,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v.bin",
        None,
    );
    add_mint(
        &mut pt,
        RESERVE_SOL_COLLATERAL_MINT,
        "2UywZrUdyqs5vDchy7fKQJKau2RVyuzBev2XKGPDSiX1.bin",
        None,
    );

    load_kamino_fixtures(&mut pt);

    let context = pt.start_with_context().await;

    Rc::new(RefCell::new(context))
}

pub async fn get_keypair(file_path: &str) -> Keypair {
    read_keypair_file(file_path).unwrap()
}

pub async fn create_payer_from_file(context: &mut ProgramTestContext, file_path: &str) -> Keypair {
    let keypair = read_keypair_file(file_path).unwrap();

    transfer(context, &keypair.pubkey(), 100_000_000_000).await;

    keypair
}

pub async fn create_user(context: &mut ProgramTestContext) -> Keypair {
    let keypair = Keypair::new();

    transfer(context, &keypair.pubkey(), 100_000_000_000).await;

    keypair
}

pub async fn transfer(context: &mut ProgramTestContext, recipient: &Pubkey, amount: u64) {
    let transaction = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &context.payer.pubkey(),
            recipient,
            amount,
        )],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.banks_client.get_latest_blockhash().await.unwrap(),
    );

    context
        .banks_client
        .process_transaction_with_preflight(transaction)
        .await
        .unwrap();
}

pub fn encode_string_to_bytes(s: &str) -> [u8; 32] {
    // Skip checking size
    let mut bytes: [u8; 32] = [b' '; 32];
    let s_as_bytes = s.as_bytes();
    bytes[..s_as_bytes.len()].copy_from_slice(s_as_bytes);

    bytes
}

pub async fn process_instructions(
    context: &mut ProgramTestContext,
    admin: &Keypair,
    instructions: &Vec<Instruction>,
) {
    let mut signers: Vec<&Keypair> = vec![];
    signers.push(admin);

    let transaction = Transaction::new_signed_with_payer(
        instructions,
        Some(&admin.pubkey()),
        &signers,
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction_with_commitment(
            transaction,
            anchor_client::solana_sdk::commitment_config::CommitmentLevel::Finalized,
        )
        .await
        .unwrap();

    let clock = get_sysvar_clock(&mut context.banks_client).await;
    context.warp_to_slot(clock.slot + 1).unwrap();
}

pub async fn create_token_account(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    account: &Keypair,
    mint: &Pubkey,
    owner: &Pubkey,
    extra_lamports: u64,
) -> Result<(), TransportError> {
    let rent = context.banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(spl_token::state::Account::LEN);

    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                account_rent + extra_lamports,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &account.pubkey(),
                mint,
                owner,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[payer, account],
        context.banks_client.get_latest_blockhash().await.unwrap(),
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .map_err(|e| e.into())
}

pub async fn create_account(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    account: &Keypair,
    size: usize,
    owner: &Pubkey,
) -> Result<(), TransportError> {
    let rent = context.banks_client.get_rent().await.unwrap();
    let account_rent = rent.minimum_balance(size);

    let mut transaction = Transaction::new_with_payer(
        &[system_instruction::create_account(
            &payer.pubkey(),
            &account.pubkey(),
            account_rent,
            size as u64,
            owner,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, account], context.last_blockhash);

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .map_err(|e| e.into())
}

pub async fn create_mint(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    mint: &Keypair,
    decimals: u8,
    owner: &Pubkey,
) -> Result<(), TransportError> {
    let rent = context.banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                mint_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                owner,
                Some(&payer.pubkey()),
                decimals,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[payer, mint], context.last_blockhash);
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .map_err(|e| e.into())
}

pub async fn create_mint_with_authorities(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    mint: &Keypair,
    decimals: u8,
    mint_authority: &Pubkey,
    freeze_authority: &Pubkey,
) -> Result<(), TransportError> {
    let rent = context.banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                mint_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint2(
                &spl_token::id(),
                &mint.pubkey(),
                mint_authority,
                Some(freeze_authority),
                decimals,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[payer, mint], context.last_blockhash);
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .map_err(|e| e.into())
}

pub async fn create_associated_token_account(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    user: &Pubkey,
    mint: &Pubkey,
) {
    let transaction = Transaction::new_signed_with_payer(
        &[
            spl_associated_token_account::instruction::create_associated_token_account(
                &payer.pubkey(),
                user,
                mint,
                &spl_token::id(),
            ),
        ],
        Some(&payer.pubkey()),
        &[payer],
        context.banks_client.get_latest_blockhash().await.unwrap(),
    );
    // transaction.sign(&[payer], context.last_blockhash);

    context
        .banks_client
        .process_transaction_with_preflight(transaction)
        .await
        .unwrap();
}

pub async fn get_associated_token_address(user: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address(user, mint)
}

pub async fn get_or_create_associated_token_address(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    user: &Pubkey,
    mint: &Pubkey,
) -> Pubkey {
    let associate_account = spl_associated_token_account::get_associated_token_address(user, mint);
    match context.banks_client.get_account(associate_account).await {
        Ok(None) => create_associated_token_account(context, payer, user, mint).await,
        Ok(Some(_)) => {} // Do nothing if already exists
        Err(_) => panic!("Got error when getting associated account"),
    }

    associate_account
}

pub async fn spl_token_mint(
    context: &mut ProgramTestContext,
    payer: &Keypair,
    mint: &Pubkey,
    account: &Pubkey,
    mint_authority: &Keypair,
    amount: u64,
) -> Result<(), TransportError> {
    let transaction = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            account,
            &mint_authority.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&payer.pubkey()),
        &[payer, mint_authority],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .map_err(|e| e.into())
}

pub async fn get_sysvar_clock(banks_client: &mut BanksClient) -> Clock {
    let clock_account = banks_client
        .get_account(sysvar::clock::id())
        .await
        .unwrap()
        .unwrap();

    let clock: Clock = bincode::deserialize(&clock_account.data).unwrap();

    clock
}

pub async fn get_account(banks_client: &mut BanksClient, address: Pubkey) -> Option<Account> {
    banks_client.get_account(address).await.unwrap()
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

pub async fn get_mint(banks_client: &mut BanksClient, address: Pubkey) -> Mint {
    let token_account = banks_client.get_account(address).await.unwrap().unwrap();
    Mint::unpack_from_slice(token_account.data.as_slice()).unwrap()
}

pub async fn get_token_balance(banks_client: &mut BanksClient, address: Pubkey) -> u64 {
    let token_account = banks_client.get_account(address).await.unwrap().unwrap();
    let account_info: spl_token::state::Account =
        spl_token::state::Account::unpack_from_slice(token_account.data.as_slice()).unwrap();
    account_info.amount
}

pub async fn get_token_account(
    banks_client: &mut BanksClient,
    address: Pubkey,
) -> spl_token::state::Account {
    let token_account = banks_client.get_account(address).await.unwrap().unwrap();
    spl_token::state::Account::unpack_from_slice(token_account.data.as_slice()).unwrap()
}
