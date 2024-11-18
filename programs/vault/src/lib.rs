use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer, Token};

declare_id!("DNjm4GTfWG7NuYPLMwnHDCeh62KURd9R3D6q3wqdnuNN");

#[error_code]
pub enum ErrorCode {
    #[msg("Vault is empty")]
    VaultEmpty,
    
    #[msg("Wallet is not authorized to claim")]
    UnauthorizedWallet,

    #[msg("Wallet has already claimed tokens")]
    AlreadyClaimed,

    #[msg("Insufficient claimable amount")]
    InsufficientAmount,

    #[msg("Only the vault owner can modify the whitelist")]
    NotVaultOwner,
}

#[program]
pub mod vault_claim {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.amount = amount;
        vault.whitelist = Vec::new();
        vault.payer = ctx.accounts.user.key();  // Store the payer as the vault owner
        
        msg!("Vault initialized at: {}", ctx.accounts.vault.key());
        Ok(())
    }

    pub fn add_to_whitelist(ctx: Context<ModifyWhitelist>, wallet: Pubkey, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        // Check if the caller is the vault owner (payer)
        if vault.payer != ctx.accounts.user.key() {
            return Err(ErrorCode::NotVaultOwner.into());
        }

        if let Some(entry) = vault.whitelist.iter_mut().find(|(w, _)| *w == wallet) {
            entry.1 = amount;
        } else {
            vault.whitelist.push((wallet, amount));
        }
        Ok(())
    }

    pub fn remove_from_whitelist(ctx: Context<ModifyWhitelist>, wallet: Pubkey) -> Result<()> {
        let vault = &mut ctx.accounts.vault;

        // Check if the caller is the vault owner (payer)
        if vault.payer != ctx.accounts.user.key() {
            return Err(ErrorCode::NotVaultOwner.into());
        }

        vault.whitelist.retain(|(w, _)| *w != wallet);
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let user_account = &ctx.accounts.user_account;

        // Check if the user is on the whitelist and retrieve the claimable amount
        let claimable_amount = vault.whitelist
            .iter()
            .find(|(w, _)| *w == user_account.key())
            .map(|(_, amount)| *amount)
            .unwrap_or(0);

        if claimable_amount == 0 {
            return Err(ErrorCode::UnauthorizedWallet.into());
        }

        if vault.amount == 0 {
            return Err(ErrorCode::VaultEmpty.into());
        }

        let claimed_amount = ctx.accounts.claimed_amount.claimed_amount;
        if claimed_amount >= claimable_amount {
            return Err(ErrorCode::AlreadyClaimed.into());
        }

        let remaining_claimable = claimable_amount - claimed_amount;

        if vault.amount < remaining_claimable {
            return Err(ErrorCode::InsufficientAmount.into());
        }

        // Transfer the tokens to the user
        let vault_authority = vault.to_account_info();  // Separate reference for authority
        let cpi_accounts = Transfer {
            from: ctx.accounts.vault_token_account.to_account_info(),
            to: user_account.to_account_info(),
            authority: vault_authority.clone(),  // Use separate immutable reference
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, remaining_claimable)?;

        // Update vault and claimed amount state
        vault.amount -= remaining_claimable;
        ctx.accounts.claimed_amount.claimed_amount += remaining_claimable;
        
        Ok(())
    }
}

#[account]
pub struct Vault {
    pub amount: u64,
    pub whitelist: Vec<(Pubkey, u64)>,
    pub payer: Pubkey,  // Store the payer's public key (vault owner)
}

#[account]
pub struct ClaimedAmount {
    pub wallet: Pubkey,
    pub claimed_amount: u64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 8 + 32 + 8 + 32)]  // Include space for payer
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ModifyWhitelist<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut, constraint = vault.payer == user.key())]  // Verify user is the vault owner
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    /// CHECK: This is a token account
    pub vault_token_account: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK: This is a token account
    pub user_account: AccountInfo<'info>,
    #[account(mut)]
    pub claimed_amount: Account<'info, ClaimedAmount>,
    pub token_program: Program<'info, Token>,
}
