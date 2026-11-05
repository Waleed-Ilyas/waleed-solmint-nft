use anchor_lang::solana_program::program_option::COption;
use anchor_lang::solana_program::program_pack::Pack;
use anchor_lang::{AccountSerialize, InstructionData, Space, ToAccountMetas};
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::spl_token::state::{Account as SplAccount, AccountState, Mint};
use anchor_spl::token::ID as TOKEN_PROGRAM_ID;
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::account::Account;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;

fn solmint_processor<'program, 'accounts, 'info, 'data>(
    program_id: &'program Pubkey,
    accounts: &'accounts [anchor_lang::solana_program::account_info::AccountInfo<'info>],
    instruction_data: &'data [u8],
) -> anchor_lang::solana_program::entrypoint::ProgramResult {
    // SAFETY: Anchor's entrypoint handles the account infos synchronously and does not retain them.
    let (accounts, instruction_data) = unsafe {
        (
            std::mem::transmute::<
                &'accounts [anchor_lang::solana_program::account_info::AccountInfo<'info>],
                &'program [anchor_lang::solana_program::account_info::AccountInfo<'program>],
            >(accounts),
            std::mem::transmute::<&'data [u8], &'program [u8]>(instruction_data),
        )
    };
    solmint::entry(program_id, accounts, instruction_data)
}

fn token_account(mint: Pubkey, owner: Pubkey, amount: u64) -> Account {
    let state = SplAccount {
        mint,
        owner,
        amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };
    let mut data = vec![0; SplAccount::LEN];
    SplAccount::pack(state, &mut data).unwrap();
    Account {
        lamports: 1_000_000,
        data,
        owner: TOKEN_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    }
}

async fn send_instruction(
    context: &mut ProgramTestContext,
    instruction: Instruction,
    additional_signers: &[&Keypair],
) -> Result<(), Box<dyn std::error::Error>> {
    let blockhash = context.banks_client.get_latest_blockhash().await?;
    let mut signers = vec![&context.payer];
    signers.extend_from_slice(additional_signers);
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &signers,
        blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await?;
    Ok(())
}

fn register_token_programs(test: &mut ProgramTest) {
    test.add_program(
        "spl_token",
        TOKEN_PROGRAM_ID,
        processor!(spl_token::processor::Processor::process),
    );
    test.add_program(
        "spl_associated_token_account",
        anchor_spl::associated_token::ID,
        processor!(spl_associated_token_account::processor::process_instruction),
    );
}

#[tokio::test]
async fn buy_rejects_a_listing_whose_vault_does_not_hold_the_nft(
) -> Result<(), Box<dyn std::error::Error>> {
    let buyer = Keypair::new();
    let seller = Keypair::new().pubkey();
    let mint = Keypair::new().pubkey();
    let (listing, bump) = Pubkey::find_program_address(&[b"listing", mint.as_ref()], &solmint::ID);
    let vault = get_associated_token_address(&listing, &mint);
    let buyer_token = get_associated_token_address(&buyer.pubkey(), &mint);

    let mint_state = Mint {
        mint_authority: COption::None,
        supply: 1,
        decimals: 0,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    let mut mint_data = vec![0; Mint::LEN];
    Mint::pack(mint_state, &mut mint_data)?;

    let mut listing_data = Vec::new();
    solmint::Listing {
        mint,
        seller,
        price_lamports: 1_000_000,
        bump,
    }
    .try_serialize(&mut listing_data)?;
    listing_data.resize(8 + solmint::Listing::INIT_SPACE, 0);

    let mut test = ProgramTest::new("solmint", solmint::ID, processor!(solmint_processor));
    register_token_programs(&mut test);
    test.add_account(
        mint,
        Account {
            lamports: 1_000_000,
            data: mint_data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        listing,
        Account {
            lamports: 1_000_000,
            data: listing_data,
            owner: solmint::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(vault, token_account(mint, listing, 0));
    test.add_account(buyer_token, token_account(mint, buyer.pubkey(), 0));
    test.add_account(
        seller,
        Account {
            lamports: 1_000_000,
            data: Vec::new(),
            owner: anchor_lang::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut context = test.start_with_context().await;
    let instruction = Instruction {
        program_id: solmint::ID,
        accounts: solmint::accounts::Buy {
            buyer: buyer.pubkey(),
            seller,
            mint,
            listing,
            vault,
            buyer_token,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
        data: solmint::instruction::Buy {}.data(),
    };

    let error = send_instruction(&mut context, instruction, &[&buyer])
        .await
        .expect_err("an empty vault must reject a purchase");
    assert!(format!("{error:?}").contains("Custom(6005)"));

    Ok(())
}

#[tokio::test]
async fn buy_transfers_the_nft_and_listing_price() -> Result<(), Box<dyn std::error::Error>> {
    let buyer = Keypair::new();
    let mint = Keypair::new().pubkey();
    let seller = Pubkey::new_unique();
    let (listing, bump) = Pubkey::find_program_address(&[b"listing", mint.as_ref()], &solmint::ID);
    let vault = get_associated_token_address(&listing, &mint);
    let buyer_token = get_associated_token_address(&buyer.pubkey(), &mint);
    let price = 2_000_000;

    let mint_state = Mint {
        mint_authority: COption::None,
        supply: 1,
        decimals: 0,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    let mut mint_data = vec![0; Mint::LEN];
    Mint::pack(mint_state, &mut mint_data)?;
    let mut listing_data = Vec::new();
    solmint::Listing {
        mint,
        seller,
        price_lamports: price,
        bump,
    }
    .try_serialize(&mut listing_data)?;
    listing_data.resize(8 + solmint::Listing::INIT_SPACE, 0);

    let mut test = ProgramTest::new("solmint", solmint::ID, processor!(solmint_processor));
    register_token_programs(&mut test);
    test.add_account(
        mint,
        Account {
            lamports: 1_000_000,
            data: mint_data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        listing,
        Account {
            lamports: 1_000_000,
            data: listing_data,
            owner: solmint::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(vault, token_account(mint, listing, 1));
    test.add_account(buyer_token, token_account(mint, buyer.pubkey(), 0));
    test.add_account(
        seller,
        Account {
            lamports: 1_000_000,
            data: Vec::new(),
            owner: anchor_lang::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        buyer.pubkey(),
        Account {
            lamports: 10_000_000,
            data: Vec::new(),
            owner: anchor_lang::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut context = test.start_with_context().await;
    let buyer_balance_before = context.banks_client.get_balance(buyer.pubkey()).await?;
    let instruction = Instruction {
        program_id: solmint::ID,
        accounts: solmint::accounts::Buy {
            buyer: buyer.pubkey(),
            seller,
            mint,
            listing,
            vault,
            buyer_token,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: anchor_spl::associated_token::ID,
            system_program: anchor_lang::system_program::ID,
        }
        .to_account_metas(None),
        data: solmint::instruction::Buy {}.data(),
    };
    send_instruction(&mut context, instruction, &[&buyer]).await?;

    let buyer_balance_after = context.banks_client.get_balance(buyer.pubkey()).await?;
    assert_eq!(buyer_balance_before - buyer_balance_after, price);
    let buyer_token_state = context
        .banks_client
        .get_account(buyer_token)
        .await?
        .expect("buyer token account should remain");
    assert_eq!(SplAccount::unpack(&buyer_token_state.data)?.amount, 1);
    assert!(context.banks_client.get_account(listing).await?.is_none());
    assert!(context.banks_client.get_account(vault).await?.is_none());
    Ok(())
}

#[tokio::test]
async fn seller_can_cancel_a_listing_and_reclaim_the_nft() -> Result<(), Box<dyn std::error::Error>>
{
    let mint = Keypair::new().pubkey();
    let seller = Keypair::new();
    let seller_pubkey = seller.pubkey();
    let (listing, bump) = Pubkey::find_program_address(&[b"listing", mint.as_ref()], &solmint::ID);
    let vault = get_associated_token_address(&listing, &mint);
    let seller_token = get_associated_token_address(&seller_pubkey, &mint);

    let mint_state = Mint {
        mint_authority: COption::None,
        supply: 1,
        decimals: 0,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    let mut mint_data = vec![0; Mint::LEN];
    Mint::pack(mint_state, &mut mint_data)?;
    let mut listing_data = Vec::new();
    solmint::Listing {
        mint,
        seller: seller_pubkey,
        price_lamports: 2_000_000,
        bump,
    }
    .try_serialize(&mut listing_data)?;
    listing_data.resize(8 + solmint::Listing::INIT_SPACE, 0);

    let mut test = ProgramTest::new("solmint", solmint::ID, processor!(solmint_processor));
    register_token_programs(&mut test);
    test.add_account(
        mint,
        Account {
            lamports: 1_000_000,
            data: mint_data,
            owner: TOKEN_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(
        listing,
        Account {
            lamports: 1_000_000,
            data: listing_data,
            owner: solmint::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    test.add_account(vault, token_account(mint, listing, 1));
    test.add_account(seller_token, token_account(mint, seller_pubkey, 0));
    test.add_account(
        seller_pubkey,
        Account {
            lamports: 1_000_000,
            data: Vec::new(),
            owner: anchor_lang::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let mut context = test.start_with_context().await;
    let instruction = Instruction {
        program_id: solmint::ID,
        accounts: solmint::accounts::CancelListing {
            seller: seller_pubkey,
            mint,
            listing,
            vault,
            seller_token,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: anchor_spl::associated_token::ID,
        }
        .to_account_metas(None),
        data: solmint::instruction::CancelListing {}.data(),
    };
    send_instruction(&mut context, instruction, &[&seller]).await?;

    let seller_token_state = context
        .banks_client
        .get_account(seller_token)
        .await?
        .expect("seller token account should remain");
    assert_eq!(SplAccount::unpack(&seller_token_state.data)?.amount, 1);
    assert!(context.banks_client.get_account(listing).await?.is_none());
    assert!(context.banks_client.get_account(vault).await?.is_none());
    Ok(())
}
