use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::{
    self, CloseAccount, Mint, MintTo, SetAuthority, Token, TokenAccount, Transfer,
};

declare_id!("7aPUXEXQv7bQi96SRwitQAbPt1H58986NzdAamdV7XFT");

#[program]
pub mod solmint {
    use super::*;

    pub fn mint_nft(
        ctx: Context<MintNft>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        require!(name.len() <= 32, SolMintError::NameTooLong);
        require!(symbol.len() <= 10, SolMintError::SymbolTooLong);
        require!(uri.len() <= 200, SolMintError::UriTooLong);

        let record = &mut ctx.accounts.record;
        record.creator = ctx.accounts.creator.key();
        record.mint = ctx.accounts.mint.key();
        record.name = name;
        record.symbol = symbol;
        record.uri = uri;
        record.bump = ctx.bumps.record;

        token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.creator_token.to_account_info(),
                    authority: ctx.accounts.creator.to_account_info(),
                },
            ),
            1,
        )?;

        // A one-of-one showcase NFT must not remain mintable or freezeable by its creator.
        token::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                SetAuthority {
                    current_authority: ctx.accounts.creator.to_account_info(),
                    account_or_mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            AuthorityType::MintTokens,
            None,
        )?;
        token::set_authority(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                SetAuthority {
                    current_authority: ctx.accounts.creator.to_account_info(),
                    account_or_mint: ctx.accounts.mint.to_account_info(),
                },
            ),
            AuthorityType::FreezeAccount,
            None,
        )
    }

    pub fn list_for_sale(ctx: Context<ListForSale>, price_lamports: u64) -> Result<()> {
        require!(price_lamports > 0, SolMintError::InvalidPrice);
        require!(
            ctx.accounts.seller_token.amount == 1,
            SolMintError::MissingNft
        );

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.seller_token.to_account_info(),
                    to: ctx.accounts.vault.to_account_info(),
                    authority: ctx.accounts.seller.to_account_info(),
                },
            ),
            1,
        )?;

        let listing = &mut ctx.accounts.listing;
        listing.mint = ctx.accounts.mint.key();
        listing.seller = ctx.accounts.seller.key();
        listing.price_lamports = price_lamports;
        listing.bump = ctx.bumps.listing;
        Ok(())
    }

    pub fn buy(ctx: Context<Buy>) -> Result<()> {
        let price_lamports = ctx.accounts.listing.price_lamports;
        require!(
            ctx.accounts.vault.amount == 1,
            SolMintError::InvalidVaultBalance
        );

        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.key(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.seller.to_account_info(),
                },
            ),
            price_lamports,
        )?;

        let mint_key = ctx.accounts.mint.key();
        let bump = [ctx.accounts.listing.bump];
        let signer_seeds: &[&[u8]] = &[b"listing", mint_key.as_ref(), &bump];
        let signer = &[signer_seeds];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.buyer_token.to_account_info(),
                    authority: ctx.accounts.listing.to_account_info(),
                },
                signer,
            ),
            1,
        )?;
        token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            CloseAccount {
                account: ctx.accounts.vault.to_account_info(),
                destination: ctx.accounts.seller.to_account_info(),
                authority: ctx.accounts.listing.to_account_info(),
            },
            signer,
        ))
    }

    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        require!(
            ctx.accounts.vault.amount == 1,
            SolMintError::InvalidVaultBalance
        );
        let mint_key = ctx.accounts.mint.key();
        let bump = [ctx.accounts.listing.bump];
        let signer_seeds: &[&[u8]] = &[b"listing", mint_key.as_ref(), &bump];
        let signer = &[signer_seeds];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                Transfer {
                    from: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.seller_token.to_account_info(),
                    authority: ctx.accounts.listing.to_account_info(),
                },
                signer,
            ),
            1,
        )?;
        token::close_account(CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            CloseAccount {
                account: ctx.accounts.vault.to_account_info(),
                destination: ctx.accounts.seller.to_account_info(),
                authority: ctx.accounts.listing.to_account_info(),
            },
            signer,
        ))
    }
}

#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(
        init,
        payer = creator,
        mint::decimals = 0,
        mint::authority = creator,
        mint::freeze_authority = creator
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = creator,
        space = 8 + MintRecord::INIT_SPACE,
        seeds = [b"metadata", mint.key().as_ref()],
        bump
    )]
    pub record: Account<'info, MintRecord>,
    #[account(
        init,
        payer = creator,
        associated_token::mint = mint,
        associated_token::authority = creator
    )]
    pub creator_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ListForSale<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(mut, token::mint = mint, token::authority = seller)]
    pub seller_token: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = seller,
        space = 8 + Listing::INIT_SPACE,
        seeds = [b"listing", mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        init,
        payer = seller,
        associated_token::mint = mint,
        associated_token::authority = listing
    )]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    #[account(mut)]
    pub seller: SystemAccount<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"listing", mint.key().as_ref()],
        bump = listing.bump,
        has_one = mint,
        has_one = seller,
        close = seller
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = listing
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = buyer
    )]
    pub buyer_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    pub mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"listing", mint.key().as_ref()],
        bump = listing.bump,
        has_one = mint,
        has_one = seller,
        close = seller
    )]
    pub listing: Account<'info, Listing>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = listing
    )]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut, token::mint = mint, token::authority = seller)]
    pub seller_token: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[account]
#[derive(InitSpace)]
pub struct MintRecord {
    pub creator: Pubkey,
    pub mint: Pubkey,
    #[max_len(32)]
    pub name: String,
    #[max_len(10)]
    pub symbol: String,
    #[max_len(200)]
    pub uri: String,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub mint: Pubkey,
    pub seller: Pubkey,
    pub price_lamports: u64,
    pub bump: u8,
}

#[error_code]
pub enum SolMintError {
    #[msg("The NFT name exceeds 32 bytes.")]
    NameTooLong,
    #[msg("The NFT symbol exceeds 10 bytes.")]
    SymbolTooLong,
    #[msg("The metadata URI exceeds 200 bytes.")]
    UriTooLong,
    #[msg("The listing price must be greater than zero.")]
    InvalidPrice,
    #[msg("The seller token account does not hold the NFT.")]
    MissingNft,
    #[msg("The listing vault must hold exactly one NFT.")]
    InvalidVaultBalance,
}
