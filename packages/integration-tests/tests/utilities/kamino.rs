#![allow(dead_code)]

use anchor_client::solana_sdk::{instruction::Instruction, native_token::LAMPORTS_PER_SOL, sysvar};
use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::hash::hash;
use anchor_lang::solana_program::pubkey;
use anchor_lang::{prelude::AccountMeta, system_program};
use anchor_spl::token::spl_token;
use borsh::BorshDeserialize;
use fixed::types::U68F60 as Fraction;
use klend::state::Reserve;
use solana_program_test::ProgramTest;

use crate::utilities::helper::read_account_data;

pub const KLEND_PROGRAM_ID: Pubkey = pubkey!("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD");
pub const KFARM_PROGRAM_ID: Pubkey = pubkey!("FarmsPZpWu9i7Kky8tPN37rs2TpmMrAZrC7S7vJa91Hr");
pub const KLEND_SCOPE_PRICES_PROGRAM_ID: Pubkey =
    pubkey!("HFn8GnPADiny6XqUoWE8uRPPxb29ikn4yTuPa9MF2fWJ");

pub const MAIN_MARKET: Pubkey = pubkey!("7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF");
pub const MAIN_MARKET_AUTHORITY: Pubkey = pubkey!("9DrvZvyWh1HuAoZxvYWMvkf2XCzryCpGgHqrMjyDWpmo");
pub const ALTCOINS_MARKET: Pubkey = pubkey!("ByYiZxp8QrdN9qbdtaAiePN8AAr3qvTPppNJDpf5DVJ5");
pub const KAMINO_SCOPE_PRICES: Pubkey = pubkey!("3NJYftD5sjVfxSnUdZ1wVML8f3aC6mp1CXCL6L7TnU8C");

pub const JITOSOL_MINT: Pubkey = pubkey!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"); // Mint

// Reserve JITOSOL state
pub const RESERVE_JITOSOL_STATE: Pubkey = pubkey!("EVbyPKrHG6WBfm4dLxLMJpUDY43cCAcHSpV3KYjKsktW");
pub const RESERVE_JITOSOL_COLLATERAL_MINT: Pubkey =
    pubkey!("9ucQp7thL38MDDTSER5ou24QnVSTZFLevDsZC1cAFkKy"); // Mint
pub const RESERVE_JITOSOL_LIQUIDITY_SUPPLY_VAULT: Pubkey =
    pubkey!("6sga1yRArgQRqa8Darhm54EBromEpV3z8iDAvMTVYXB3"); // Token Account
pub const RESERVE_JITOSOL_COLLATERAL_SUPPLY_VAULT: Pubkey =
    pubkey!("7y5Nko765HcZiTd2gFtxorELuJZcbQqmrmTbUVoiwGyS"); // Token Account
pub const RESERVE_JITOSOL_FARM_STATE: Pubkey = pubkey!("11111111111111111111111111111111"); // ???

// Reserve SOL state
pub const RESERVE_SOL_STATE: Pubkey = pubkey!("d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q");
pub const RESERVE_SOL_LIQUIDITY_MINT: Pubkey =
    pubkey!("So11111111111111111111111111111111111111112"); // Mint
pub const RESERVE_SOL_LIQUIDITY_SUPPLY_VAULT: Pubkey =
    pubkey!("GafNuUXj9rxGLn4y79dPu6MHSuPWeJR6UtTWuexpGh3U"); // Token Account
pub const RESERVE_SOL_LIQUIDITY_FEE_VAULT: Pubkey =
    pubkey!("3JNof8s453bwG5UqiXBLJc77NRQXezYYEBbk3fqnoKph"); // Token Account
pub const RESERVE_SOL_COLLATERAL_MINT: Pubkey =
    pubkey!("2UywZrUdyqs5vDchy7fKQJKau2RVyuzBev2XKGPDSiX1"); // Mint
pub const RESERVE_SOL_FARM_STATE: Pubkey = pubkey!("955xWFhSDcDiUgUr4sBRtCpTLiMd4H5uZLAmgtP3R3sX");

// Reserve USDC state
pub const RESERVE_USDC_STATE: Pubkey = pubkey!("D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59");
pub const RESERVE_USDC_LIQUIDITY_MINT: Pubkey =
    pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // Mint
pub const RESERVE_USDC_LIQUIDITY_SUPPLY_VAULT: Pubkey =
    pubkey!("Bgq7trRgVMeq33yt235zM2onQ4bRDBsY5EWiTetF4qw6"); // Token Account
pub const RESERVE_USDC_LIQUIDITY_FEE_VAULT: Pubkey =
    pubkey!("BbDUrk1bVtSixgQsPLBJFZEF7mwGstnD5joA1WzYvYFX"); // Token Account
pub const RESERVE_USDC_FARM_STATE: Pubkey = pubkey!("11111111111111111111111111111111"); // ???

pub fn load_kamino_fixtures(pt: &mut ProgramTest) {
    pt.add_program("klend", KLEND_PROGRAM_ID, None);
    pt.add_program("kfarm", KFARM_PROGRAM_ID, None);
    pt.add_program("klend_refresh_price", KLEND_SCOPE_PRICES_PROGRAM_ID, None);

    // Main Market
    pt.add_account_with_file_data(
        MAIN_MARKET,
        LAMPORTS_PER_SOL,
        KLEND_PROGRAM_ID,
        "7u3HeHxYDLhnCoErrtycNokbQYbWGzLs6JSDqGAv5PfF.bin",
    );

    pt.add_account_with_file_data(
        MAIN_MARKET_AUTHORITY,
        LAMPORTS_PER_SOL * 10,
        system_program::ID,
        "9DrvZvyWh1HuAoZxvYWMvkf2XCzryCpGgHqrMjyDWpmo.bin",
    );

    // Altcoins Market
    // pt.add_account_with_file_data(
    //     ALTCOINS_MARKET,
    //     LAMPORTS_PER_SOL,
    //     KLEND_PROGRAM_ID,
    //     "ByYiZxp8QrdN9qbdtaAiePN8AAr3qvTPppNJDpf5DVJ5.bin",
    // );

    // Reserve JITOSOL state
    pt.add_account_with_file_data(
        RESERVE_JITOSOL_STATE,
        LAMPORTS_PER_SOL,
        KLEND_PROGRAM_ID,
        "EVbyPKrHG6WBfm4dLxLMJpUDY43cCAcHSpV3KYjKsktW.bin",
    );

    pt.add_account_with_file_data(
        RESERVE_JITOSOL_LIQUIDITY_SUPPLY_VAULT,
        LAMPORTS_PER_SOL,
        spl_token::id(),
        "6sga1yRArgQRqa8Darhm54EBromEpV3z8iDAvMTVYXB3.bin",
    );

    pt.add_account_with_file_data(
        RESERVE_JITOSOL_COLLATERAL_SUPPLY_VAULT,
        LAMPORTS_PER_SOL,
        spl_token::id(),
        "7y5Nko765HcZiTd2gFtxorELuJZcbQqmrmTbUVoiwGyS.bin",
    );

    // Reserve SOL state
    pt.add_account_with_file_data(
        RESERVE_SOL_STATE,
        LAMPORTS_PER_SOL,
        KLEND_PROGRAM_ID,
        "d4A2prbA2whesmvHaL88BH6Ewn5N4bTSU2Ze8P6Bc4Q.bin",
    );

    pt.add_account_with_file_data(
        RESERVE_SOL_LIQUIDITY_SUPPLY_VAULT,
        LAMPORTS_PER_SOL,
        spl_token::id(),
        "GafNuUXj9rxGLn4y79dPu6MHSuPWeJR6UtTWuexpGh3U.bin",
    );

    pt.add_account_with_file_data(
        RESERVE_SOL_LIQUIDITY_FEE_VAULT,
        LAMPORTS_PER_SOL,
        spl_token::id(),
        "3JNof8s453bwG5UqiXBLJc77NRQXezYYEBbk3fqnoKph.bin",
    );

    pt.add_account_with_file_data(
        RESERVE_SOL_FARM_STATE,
        LAMPORTS_PER_SOL,
        KFARM_PROGRAM_ID,
        "955xWFhSDcDiUgUr4sBRtCpTLiMd4H5uZLAmgtP3R3sX.bin",
    );

    // Reserve USDC state
    pt.add_account_with_file_data(
        RESERVE_USDC_STATE,
        LAMPORTS_PER_SOL,
        KLEND_PROGRAM_ID,
        "D6q6wuQSrifJKZYpR1M8R4YawnLDtDsMmWM1NbBmgJ59.bin",
    );

    pt.add_account_with_file_data(
        RESERVE_USDC_LIQUIDITY_SUPPLY_VAULT,
        LAMPORTS_PER_SOL,
        spl_token::id(),
        "Bgq7trRgVMeq33yt235zM2onQ4bRDBsY5EWiTetF4qw6.bin",
    );

    pt.add_account_with_file_data(
        RESERVE_USDC_LIQUIDITY_FEE_VAULT,
        LAMPORTS_PER_SOL,
        spl_token::id(),
        "BbDUrk1bVtSixgQsPLBJFZEF7mwGstnD5joA1WzYvYFX.bin",
    );

    // Kamino scope prices
    pt.add_account_with_file_data(
        KAMINO_SCOPE_PRICES,
        LAMPORTS_PER_SOL,
        KLEND_SCOPE_PRICES_PROGRAM_ID,
        "3NJYftD5sjVfxSnUdZ1wVML8f3aC6mp1CXCL6L7TnU8C.bin",
    );

    println!("Load kamino fixtures.")
}

pub fn dump_reserve(address: &Pubkey) {
    let filename = format!("{}.bin", address);
    let data = read_account_data(&filename);
    let reserve = Reserve::try_from_slice(&data[8..]).unwrap(); // Skip discriminator !

    // println!("lending market {:?}", reserve.lending_market);
    // println!("reserve {:#?}", reserve.liquidity);

    // let reserve_liquidity_supply = Fraction::from(reserve.liquidity.available_amount)
    //     + Fraction::from_bits(reserve.liquidity.borrowed_amount_sf)
    //     - Fraction::from_bits(reserve.liquidity.accumulated_protocol_fees_sf)
    //     - Fraction::from_bits(reserve.liquidity.accumulated_referrer_fees_sf)
    //     - Fraction::from_bits(reserve.liquidity.pending_referrer_fees_sf);

    // println!("reserve_liquidity_supply = {}", reserve_liquidity_supply);

    let reserve_liquidity_borrowed_f = Fraction::from_bits(reserve.liquidity.borrowed_amount_sf);
    println!(
        "reserve_liquidity_borrowed_f = {}, limit = {}",
        reserve_liquidity_borrowed_f, reserve.config.borrow_limit
    );
}

pub fn compose_klend_init_user_metadata_ix(
    owner: &Pubkey,
    fee_payer: &Pubkey,
    user_metadata: &Pubkey,
    user_lookup_table: &Pubkey,
) -> Instruction {
    let hash = hash(b"global:init_user_metadata");

    let mut data: Vec<u8> = Vec::new();
    data.extend_from_slice(&hash.as_ref()[0..8].to_vec());
    data.extend_from_slice(&user_lookup_table.to_bytes());

    Instruction {
        program_id: KLEND_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*fee_payer, true),
            AccountMeta::new(*user_metadata, false),
            AccountMeta::new_readonly(KLEND_PROGRAM_ID, false), // no referrer
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    }
}

pub fn compose_klend_init_obligation_ix(
    owner: &Pubkey,
    fee_payer: &Pubkey,
    obligation: &Pubkey,
    lending_market: &Pubkey,
    seed1_account: &Pubkey,
    seed2_account: &Pubkey,
    user_metadata: &Pubkey,
    tag: u8,
    id: u8,
) -> Instruction {
    let mut data: Vec<u8> = hash(b"global:init_obligation").as_ref()[0..8].to_vec();
    data.push(tag);
    data.push(id);

    Instruction {
        program_id: KLEND_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*fee_payer, true),
            AccountMeta::new(*obligation, false),
            AccountMeta::new_readonly(*lending_market, false),
            AccountMeta::new_readonly(*seed1_account, false),
            AccountMeta::new_readonly(*seed2_account, false),
            AccountMeta::new_readonly(*user_metadata, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    }
}

pub fn compose_klend_refresh_reserve_ix(reserve: &Pubkey, market: &Pubkey) -> Instruction {
    let data = hash(b"global:refresh_reserve").as_ref()[0..8].to_vec();

    Instruction {
        program_id: KLEND_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*reserve, false),
            AccountMeta::new_readonly(*market, false),
            AccountMeta::new_readonly(KLEND_PROGRAM_ID, false),
            AccountMeta::new_readonly(KLEND_PROGRAM_ID, false),
            AccountMeta::new_readonly(KLEND_PROGRAM_ID, false),
            AccountMeta::new_readonly(KAMINO_SCOPE_PRICES, false),
        ],
        data,
    }
}

pub fn compose_klend_refresh_obligation_ix(
    obligation: &Pubkey,
    market: &Pubkey,
    reserves: &Vec<Pubkey>,
) -> Instruction {
    let data = hash(b"global:refresh_obligation").as_ref()[0..8].to_vec();

    let mut accounts: Vec<AccountMeta> = vec![
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new(*obligation, false),
    ];

    for reserve in reserves {
        accounts.push(AccountMeta::new(*reserve, false));
    }

    Instruction {
        program_id: KLEND_PROGRAM_ID,
        accounts,
        data,
    }
}

pub fn compose_klend_deposit_reserve_liquidity_ix(
    owner: &Pubkey,
    reserve: &Pubkey,
    lending_market: &Pubkey,
    lending_market_authority: &Pubkey,
    reserve_liquidity_mint: &Pubkey,
    reserve_liquidity_supply: &Pubkey,
    reserve_collateral_mint: &Pubkey,
    user_source_liquidity: &Pubkey,
    user_destination_collateral: &Pubkey,
    liquidity_amount: u64,
) -> Instruction {
    let mut data = hash(b"global:deposit_reserve_liquidity").as_ref()[0..8].to_vec();
    data.extend_from_slice(&liquidity_amount.to_le_bytes());

    Instruction {
        program_id: KLEND_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*reserve, false),
            AccountMeta::new_readonly(*lending_market, false),
            AccountMeta::new(*lending_market_authority, false),
            AccountMeta::new(*reserve_liquidity_mint, false),
            AccountMeta::new(*reserve_liquidity_supply, false),
            AccountMeta::new(*reserve_collateral_mint, false),
            AccountMeta::new(*user_source_liquidity, false),
            AccountMeta::new(*user_destination_collateral, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::instructions::ID, false),
        ],
        data,
    }
}

pub fn compose_klend_deposit_obligation_collateral_ix(
    owner: &Pubkey,
    obligation: &Pubkey,
    lending_market: &Pubkey,
    deposit_reserve: &Pubkey,
    reserve_destination_collateral: &Pubkey,
    user_source_collateral: &Pubkey,
    collateral_amount: u64,
) -> Instruction {
    let mut data = hash(b"global:deposit_obligation_collateral").as_ref()[0..8].to_vec();
    data.extend_from_slice(&collateral_amount.to_le_bytes());

    Instruction {
        program_id: KLEND_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*obligation, false),
            AccountMeta::new_readonly(*lending_market, false),
            AccountMeta::new(*deposit_reserve, false),
            AccountMeta::new(*reserve_destination_collateral, false),
            AccountMeta::new(*user_source_collateral, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::instructions::ID, false),
        ],
        data,
    }
}

pub fn compose_klend_borrow_obligation_liquidity_ix(
    owner: &Pubkey,
    obligation: &Pubkey,
    lending_market: &Pubkey,
    lending_market_authority: &Pubkey,
    borrow_reserve: &Pubkey,
    borrow_reserve_liquidity_mint: &Pubkey,
    reserve_source_liquidity: &Pubkey,
    borrow_reserve_liquidity_fee_receiver: &Pubkey,
    user_destination_liquidity: &Pubkey,
    liquidity_amount: u64,
) -> Instruction {
    let mut data = hash(b"global:borrow_obligation_liquidity").as_ref()[0..8].to_vec();
    data.extend_from_slice(&liquidity_amount.to_le_bytes());

    Instruction {
        program_id: KLEND_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*owner, true),
            AccountMeta::new(*obligation, false),
            AccountMeta::new_readonly(*lending_market, false),
            AccountMeta::new_readonly(*lending_market_authority, false),
            AccountMeta::new(*borrow_reserve, false),
            AccountMeta::new(*borrow_reserve_liquidity_mint, false),
            AccountMeta::new(*reserve_source_liquidity, false),
            AccountMeta::new(*borrow_reserve_liquidity_fee_receiver, false),
            AccountMeta::new(*user_destination_liquidity, false),
            AccountMeta::new(KLEND_PROGRAM_ID, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(sysvar::instructions::ID, false),
        ],
        data,
    }
}

pub fn compose_klend_init_obligation_farms_for_reserve_ix(
    payer: &Pubkey,
    owner: &Pubkey,
    obligation: &Pubkey,
    lending_market_authority: &Pubkey,
    reserve: &Pubkey,
    reserve_farm_state: &Pubkey,
    obligation_farm: &Pubkey,
    lending_market: &Pubkey,
    mode: u8,
) -> Instruction {
    let mut data = hash(b"global:init_obligation_farms_for_reserve").as_ref()[0..8].to_vec();
    data.push(mode);

    Instruction {
        program_id: KLEND_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*owner, false),
            AccountMeta::new(*obligation, false),
            AccountMeta::new(*lending_market_authority, false),
            AccountMeta::new(*reserve, false),
            AccountMeta::new(*reserve_farm_state, false),
            AccountMeta::new(*obligation_farm, false),
            AccountMeta::new_readonly(*lending_market, false),
            AccountMeta::new(KFARM_PROGRAM_ID, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    }
}
