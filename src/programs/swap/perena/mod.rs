use {
    crate::Swap,
    core::mem::MaybeUninit,
    pinocchio::{
        AccountView, Address, ProgramResult,
        cpi::{Signer, invoke_signed},
        error::ProgramError,
        instruction::{InstructionAccount, InstructionView},
    },
};

pub const PERENA_PROGRAM_ID: Address = Address::new_from_array(five8_const::decode_32_const(
    "NUMERUNsFCP3kuNmWZuXtm1AaQCPj9uw6Guv2Ekoi5P",
));

// Anchor discriminator for swap instruction
const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

/// Perena DEX integration
pub struct Perena;

/// Protocol-specific swap data for Perena
pub struct PerenaSwapData {
    pub in_index: u8,
    pub out_index: u8,
}

impl TryFrom<&[u8]> for PerenaSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 2 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            in_index: data[0],
            out_index: data[1],
        })
    }
}

/// Account context for Perena's swap instruction.
///
/// # Account Order
/// 0. perena_program - Target program (for detection)
/// 1. pool - writable
/// 2. in_mint - writable
/// 3. out_mint - writable
/// 4. in_trader - writable
/// 5. out_trader - writable
/// 6. in_vault - writable
/// 7. out_vault - writable
/// 8. numeraire_config
/// 9. payer - signer
/// 10. token_program
/// 11. token_2022_program
pub struct PerenaSwapAccounts<'info> {
    pub perena_program: &'info AccountView,
    pub pool: &'info AccountView,
    pub in_mint: &'info AccountView,
    pub out_mint: &'info AccountView,
    pub in_trader: &'info AccountView,
    pub out_trader: &'info AccountView,
    pub in_vault: &'info AccountView,
    pub out_vault: &'info AccountView,
    pub numeraire_config: &'info AccountView,
    pub payer: &'info AccountView,
    pub token_program: &'info AccountView,
    pub token_2022_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for PerenaSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 12 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [
            perena_program,
            pool,
            in_mint,
            out_mint,
            in_trader,
            out_trader,
            in_vault,
            out_vault,
            numeraire_config,
            payer,
            token_program,
            token_2022_program,
            ..,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(PerenaSwapAccounts {
            perena_program,
            pool,
            in_mint,
            out_mint,
            in_trader,
            out_trader,
            in_vault,
            out_vault,
            numeraire_config,
            payer,
            token_program,
            token_2022_program,
        })
    }
}

impl<'info> Swap<'info> for Perena {
    type Accounts = PerenaSwapAccounts<'info>;
    type Data = PerenaSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable(ctx.pool.address()),
            InstructionAccount::writable(ctx.in_mint.address()),
            InstructionAccount::writable(ctx.out_mint.address()),
            InstructionAccount::writable(ctx.in_trader.address()),
            InstructionAccount::writable(ctx.out_trader.address()),
            InstructionAccount::writable(ctx.in_vault.address()),
            InstructionAccount::writable(ctx.out_vault.address()),
            InstructionAccount::readonly(ctx.numeraire_config.address()),
            InstructionAccount::writable_signer(ctx.payer.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.token_2022_program.address()),
        ];

        let account_infos = [
            ctx.pool,
            ctx.in_mint,
            ctx.out_mint,
            ctx.in_trader,
            ctx.out_trader,
            ctx.in_vault,
            ctx.out_vault,
            ctx.numeraire_config,
            ctx.payer,
            ctx.token_program,
            ctx.token_2022_program,
        ];

        // Instruction data: discriminator(8) + in_index(1) + out_index(1) + exact_amount_in(8) + min_amount_out(8) = 26 bytes
        let mut instruction_data = MaybeUninit::<[u8; 26]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::write(ptr.add(8), data.in_index);
            core::ptr::write(ptr.add(9), data.out_index);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(10), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(18),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &PERENA_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 26)
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
