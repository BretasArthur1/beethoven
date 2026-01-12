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

pub const HEAVEN_PROGRAM_ID: Address = Address::new_from_array(five8_const::decode_32_const(
    "HEAVENoP2qxoeuF8Dj2oT1GHEnu49U5mJYkdeC8BAX2o",
));

// Anchor discriminator for swap instruction
const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

/// Heaven DEX integration
pub struct Heaven;

/// Protocol-specific swap data for Heaven
/// The event field is typically empty but can be set for custom event logging
#[derive(Default)]
pub struct HeavenSwapData<'a> {
    pub event: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for HeavenSwapData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self { event: data })
    }
}

/// Account context for Heaven's swap instruction.
///
/// # Account Order
/// 0. heaven_program - Target program (for detection)
/// 1. token_a_owner
/// 2. token_b_owner
/// 3. ata_program
/// 4. system_program
/// 5. pool_state - writable
/// 6. user - signer
/// 7. token_a_mint
/// 8. token_b_mint
/// 9. user_token_a_account - writable
/// 10. user_token_b_account - writable
/// 11. pool_token_a_account - writable
/// 12. pool_token_b_account - writable
/// 13. protocol_config - writable
/// 14. ix_sysvar
/// 15. chainlink_id
/// 16. chainlink_sol_usd_feed
pub struct HeavenSwapAccounts<'info> {
    pub heaven_program: &'info AccountView,
    pub token_a_owner: &'info AccountView,
    pub token_b_owner: &'info AccountView,
    pub ata_program: &'info AccountView,
    pub system_program: &'info AccountView,
    pub pool_state: &'info AccountView,
    pub user: &'info AccountView,
    pub token_a_mint: &'info AccountView,
    pub token_b_mint: &'info AccountView,
    pub user_token_a_account: &'info AccountView,
    pub user_token_b_account: &'info AccountView,
    pub pool_token_a_account: &'info AccountView,
    pub pool_token_b_account: &'info AccountView,
    pub protocol_config: &'info AccountView,
    pub ix_sysvar: &'info AccountView,
    pub chainlink_id: &'info AccountView,
    pub chainlink_sol_usd_feed: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for HeavenSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 17 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [
            heaven_program,
            token_a_owner,
            token_b_owner,
            ata_program,
            system_program,
            pool_state,
            user,
            token_a_mint,
            token_b_mint,
            user_token_a_account,
            user_token_b_account,
            pool_token_a_account,
            pool_token_b_account,
            protocol_config,
            ix_sysvar,
            chainlink_id,
            chainlink_sol_usd_feed,
            ..,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(HeavenSwapAccounts {
            heaven_program,
            token_a_owner,
            token_b_owner,
            ata_program,
            system_program,
            pool_state,
            user,
            token_a_mint,
            token_b_mint,
            user_token_a_account,
            user_token_b_account,
            pool_token_a_account,
            pool_token_b_account,
            protocol_config,
            ix_sysvar,
            chainlink_id,
            chainlink_sol_usd_feed,
        })
    }
}

impl<'info> Swap<'info> for Heaven {
    type Accounts = HeavenSwapAccounts<'info>;
    type Data = HeavenSwapData<'info>;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly(ctx.token_a_owner.address()),
            InstructionAccount::readonly(ctx.token_b_owner.address()),
            InstructionAccount::readonly(ctx.ata_program.address()),
            InstructionAccount::readonly(ctx.system_program.address()),
            InstructionAccount::writable(ctx.pool_state.address()),
            InstructionAccount::readonly_signer(ctx.user.address()),
            InstructionAccount::readonly(ctx.token_a_mint.address()),
            InstructionAccount::readonly(ctx.token_b_mint.address()),
            InstructionAccount::writable(ctx.user_token_a_account.address()),
            InstructionAccount::writable(ctx.user_token_b_account.address()),
            InstructionAccount::writable(ctx.pool_token_a_account.address()),
            InstructionAccount::writable(ctx.pool_token_b_account.address()),
            InstructionAccount::writable(ctx.protocol_config.address()),
            InstructionAccount::readonly(ctx.ix_sysvar.address()),
            InstructionAccount::readonly(ctx.chainlink_id.address()),
            InstructionAccount::readonly(ctx.chainlink_sol_usd_feed.address()),
        ];

        let account_infos = [
            ctx.token_a_owner,
            ctx.token_b_owner,
            ctx.ata_program,
            ctx.system_program,
            ctx.pool_state,
            ctx.user,
            ctx.token_a_mint,
            ctx.token_b_mint,
            ctx.user_token_a_account,
            ctx.user_token_b_account,
            ctx.pool_token_a_account,
            ctx.pool_token_b_account,
            ctx.protocol_config,
            ctx.ix_sysvar,
            ctx.chainlink_id,
            ctx.chainlink_sol_usd_feed,
        ];

        // Instruction data: discriminator(8) + in_amount(8) + minimum_amount_out(8) + event_string
        // Borsh string encoding: 4 bytes length prefix + string bytes
        let event_len = data.event.len();
        let instruction_data_len = 8 + 8 + 8 + 4 + event_len;

        // For empty event (most common case), use stack allocation
        if event_len == 0 {
            let mut instruction_data = MaybeUninit::<[u8; 28]>::uninit();
            unsafe {
                let ptr = instruction_data.as_mut_ptr() as *mut u8;
                core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
                core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
                core::ptr::copy_nonoverlapping(
                    minimum_out_amount.to_le_bytes().as_ptr(),
                    ptr.add(16),
                    8,
                );
                // Empty string: length = 0
                core::ptr::copy_nonoverlapping(0u32.to_le_bytes().as_ptr(), ptr.add(24), 4);
            }

            let instruction = InstructionView {
                program_id: &HEAVEN_PROGRAM_ID,
                accounts: &accounts,
                data: unsafe {
                    core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 28)
                },
            };

            return invoke_signed(&instruction, &account_infos, signer_seeds);
        }

        // For non-empty event, we need dynamic allocation
        // Since we're no_std, we'll use a fixed max size buffer
        const MAX_EVENT_LEN: usize = 256;
        if event_len > MAX_EVENT_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut instruction_data = MaybeUninit::<[u8; 28 + MAX_EVENT_LEN]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            // String length prefix
            core::ptr::copy_nonoverlapping(
                (event_len as u32).to_le_bytes().as_ptr(),
                ptr.add(24),
                4,
            );
            // String bytes
            core::ptr::copy_nonoverlapping(data.event.as_ptr(), ptr.add(28), event_len);
        }

        let instruction = InstructionView {
            program_id: &HEAVEN_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(
                    instruction_data.as_ptr() as *const u8,
                    instruction_data_len,
                )
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
