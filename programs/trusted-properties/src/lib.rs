use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use std::convert::Into;

#[program]
mod trusted_properties {
	use super::*;

	/* ==========================================================================
								Instructions
	===========================================================================*/



	pub fn deposit_security(
		ctx: Context<DepositSecurity>,
		security_deposit_amount: u64
	) -> ProgramResult {

		msg!("TrustedProperties: DepositSecurity start");

		let rent_data = &mut ctx.accounts.rent_agreement_account;

		// msg!("SECURITY DEPOSIT::: {}", security_deposit_amount);

		if !rent_data.is_security_deposit_pending() {
			msg!("[TrustedProperties] ERROR: Security already deposited");
			return Err(ErrorCode::SecurityAlreadyDeposited.into());
		}

		rent_data.remaining_security_deposit = security_deposit_amount;
		rent_data.status = AgreementStatus::Active as u8;

		msg!("TrustedProperties: DepositSecurity state set. Now starting transfer");

		token::transfer(ctx.accounts.into(), security_deposit_amount)?;

		msg!("TrustedProperties: DepositSecurity end");

		Ok(())
	}
}



/* ==========================================================================
							Accounts for Instructions
===========================================================================*/

#[derive(Accounts)]
pub struct InitializeRentContract<'info> {
	#[account(zero)]	// (init, payer = owner, space = 1 + 32 + 32 + 8 + 8 + 1 + 1 + 1 + 2 + 1 + 8)]		// (zero)
	pub rent_agreement_account: ProgramAccount<'info, RentAgreementAccount>,
	pub owner: AccountInfo<'info>,
	pub tenant: AccountInfo<'info>,
	pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct DepositSecurity<'info> {
	#[account(mut)]
	pub rent_agreement_account: ProgramAccount<'info, RentAgreementAccount>,
	#[account(mut,signer)]
	pub tenant: AccountInfo<'info>,
	// #[account(mut,signer)]
	// pub tenant_authority: AccountInfo<'info>,

	// Misc.
    // #[account(constraint = token_program.key == &token::ID)]
    token_program: AccountInfo<'info>,
}

impl<'a, 'b, 'c, 'info> From<&mut DepositSecurity<'info>>
    for CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>
{
    fn from(accounts: &mut DepositSecurity<'info>) -> CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
		msg!("TrustedProperties: Account -> CpiContext start");
        let cpi_accounts = Transfer {
            from: accounts.tenant.clone(),
            to: accounts.rent_agreement_account.to_account_info(),
            authority: accounts.tenant.clone(),
        };
        let cpi_program = accounts.token_program.clone();

		msg!("TrustedProperties: Account -> CpiContext end");
        CpiContext::new(cpi_program, cpi_accounts)
    }
}



/* ==========================================================================
							Account States (Data)
===========================================================================*/

#[account]
pub struct RentAgreementAccount {

	/// Agreement status (active, complete, terminated, etc)
	pub status: u8,

	/// Property owner account's public-key
	pub owner_pubkey: Pubkey,

	/// Tenant account's public-key
	pub tenant_pubkey: Pubkey,

	/// Security-deposit escrow account's public-key
	// pub security_escrow_pubkey: Pubkey,

	/// Minimum security deposit (in Lamports) to be made by the tenant before the contract begins
	pub security_deposit: u64,

	/// Rent amount per month (in Lamports)
	pub rent_amount: u64,

	/// Duration of the agreement (in months)
	pub duration: u8,

	/// Count of monthly payments due
	pub remaining_payments: u8,

	/// Count of monthly payments due
	pub remaining_security_deposit: u64,

	/// Contract start month (1-12)
	pub start_month: u8,

	/// Contract start year (eg: 2021)
	pub start_year: u16,

	/// Duration (in months) for contract extension requested by Tenant
	pub duration_extension_request: u8
}

impl RentAgreementAccount {

	/// Is initial security_deposit pending by the tenant?
	pub fn is_security_deposit_pending(&self) -> bool {
		self.status == AgreementStatus::DepositPending as u8
	}

	/// Is the rent-agreement complete (i.e, all payments done for the agreed duration)?
	pub fn is_completed(&self) -> bool {
		self.status == AgreementStatus::Completed as u8
	}

	/// Is the rent-agreement terminated?
	pub fn is_terminated(&self) -> bool {
		self.status == AgreementStatus::Terminated as u8
	}
}


#[derive(Copy, Clone)]
pub enum AgreementStatus {
	Uninitialized = 0,
	DepositPending,
	Active,
	Completed,
	Terminated,
}


/* ==========================================================================
							Error Types
===========================================================================*/
#[error]
pub enum ErrorCode {
	#[msg("Invalid Instruction")]
	InvalidInstruction,

	#[msg("Incorrect Payment Amount")]
	IncorrectPaymentAmount,

	#[msg("Full Rent Already Paid")]
	RentAlreadyFullyPaid,

	#[msg("Security Amount Already Deposited")]
	SecurityAlreadyDeposited,

	#[msg("Rent Agreement Already Terminated")]
	RentAgreementTerminated,

	#[msg("Invalid Agreement Status")]
	InvalidAgreementStatus,

	#[msg("Invalid Instruction Parameter")]
	InvalidInstructionParameter,
}

