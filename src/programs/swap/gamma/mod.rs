use {
    crate::Swap,
    core::mem::MaybeUninit,
    pinocchio::{
        cpi::{invoke_signed, Signer},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
        AccountView, Address, ProgramResult,
    },
};

pub const GAMMA_PROGRAM_ID: Address = Address::new_from_array(five8_const::decode_32_const(
    "GAMMA7meSFWaBXF25oSUgmGRwaW6sCMFLmBNiMSdbHVT",
));

// Anchor discriminator for swap instruction
const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

/// Gamma DEX integration
pub struct Gamma;

/// Account context for Gamma's swap instruction.
///
/// # Account Order
/// 0. gamma_program - Target program (for detection)
/// 1. payer - signer
/// 2. authority
/// 3. amm_config
/// 4. pool_state - writable
/// 5. input_token_account - writable
/// 6. output_token_account - writable
/// 7. input_vault - writable
/// 8. output_vault - writable
/// 9. input_token_program
/// 10. output_token_program
/// 11. input_token_mint
/// 12. output_token_mint
/// 13. observation_state - writable
pub struct GammaSwapAccounts<'info> {
    pub gamma_program: &'info AccountView,
    pub payer: &'info AccountView,
    pub authority: &'info AccountView,
    pub amm_config: &'info AccountView,
    pub pool_state: &'info AccountView,
    pub input_token_account: &'info AccountView,
    pub output_token_account: &'info AccountView,
    pub input_vault: &'info AccountView,
    pub output_vault: &'info AccountView,
    pub input_token_program: &'info AccountView,
    pub output_token_program: &'info AccountView,
    pub input_token_mint: &'info AccountView,
    pub output_token_mint: &'info AccountView,
    pub observation_state: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for GammaSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 14 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [gamma_program, payer, authority, amm_config, pool_state, input_token_account, output_token_account, input_vault, output_vault, input_token_program, output_token_program, input_token_mint, output_token_mint, observation_state, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(GammaSwapAccounts {
            gamma_program,
            payer,
            authority,
            amm_config,
            pool_state,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            input_token_program,
            output_token_program,
            input_token_mint,
            output_token_mint,
            observation_state,
        })
    }
}

impl<'info> Swap<'info> for Gamma {
    type Accounts = GammaSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly_signer(ctx.payer.address()),
            InstructionAccount::readonly(ctx.authority.address()),
            InstructionAccount::readonly(ctx.amm_config.address()),
            InstructionAccount::writable(ctx.pool_state.address()),
            InstructionAccount::writable(ctx.input_token_account.address()),
            InstructionAccount::writable(ctx.output_token_account.address()),
            InstructionAccount::writable(ctx.input_vault.address()),
            InstructionAccount::writable(ctx.output_vault.address()),
            InstructionAccount::readonly(ctx.input_token_program.address()),
            InstructionAccount::readonly(ctx.output_token_program.address()),
            InstructionAccount::readonly(ctx.input_token_mint.address()),
            InstructionAccount::readonly(ctx.output_token_mint.address()),
            InstructionAccount::writable(ctx.observation_state.address()),
        ];

        let account_infos = [
            ctx.payer,
            ctx.authority,
            ctx.amm_config,
            ctx.pool_state,
            ctx.input_token_account,
            ctx.output_token_account,
            ctx.input_vault,
            ctx.output_vault,
            ctx.input_token_program,
            ctx.output_token_program,
            ctx.input_token_mint,
            ctx.output_token_mint,
            ctx.observation_state,
        ];

        // Instruction data: discriminator(8) + amount_in(8) + min_amount_out(8) = 24 bytes
        let mut instruction_data = MaybeUninit::<[u8; 24]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &GAMMA_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 24)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }

    fn swap(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
    ) -> ProgramResult {
        Self::swap_signed(ctx, in_amount, minimum_out_amount, data, &[])
    }
}
