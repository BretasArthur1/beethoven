use pinocchio::{
    address::address_eq, cpi::Signer, error::ProgramError, AccountView, ProgramResult,
};

/// Core trait for swap operations across different DEX protocols.
///
/// Each protocol implements this trait with its specific account requirements,
/// instruction data format, and CPI logic.
pub trait Swap<'info> {
    /// Protocol-specific accounts required for the swap CPI
    type Accounts;

    /// Protocol-specific instruction data beyond in_amount and minimum_out_amount
    type Data;

    /// Execute a swap with PDA signing capability
    ///
    /// # Arguments
    /// * `ctx` - Protocol-specific account context
    /// * `in_amount` - Amount of input tokens to swap
    /// * `minimum_out_amount` - Minimum acceptable output amount (slippage protection)
    /// * `data` - Protocol-specific additional instruction data
    /// * `signer_seeds` - Seeds for PDA signing
    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult;

    /// Execute a swap without signing (user is direct signer)
    ///
    /// # Arguments
    /// * `ctx` - Protocol-specific account context
    /// * `in_amount` - Amount of input tokens to swap
    /// * `minimum_out_amount` - Minimum acceptable output amount (slippage protection)
    /// * `data` - Protocol-specific additional instruction data
    fn swap(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
    ) -> ProgramResult;
}

/// Typed context for swap operations, discriminated by protocol.
///
/// This enum contains the protocol-specific account structures after parsing
/// and discrimination. Users can pattern match on this to perform custom
/// validation before executing the swap.
pub enum SwapContext<'info> {
    #[cfg(feature = "perena-swap")]
    Perena(crate::programs::swap::perena::PerenaSwapAccounts<'info>),

    #[cfg(feature = "solfi-swap")]
    SolFi(crate::programs::swap::solfi::SolFiSwapAccounts<'info>),

    #[cfg(feature = "solfi_v2-swap")]
    SolFiV2(crate::programs::swap::solfi_v2::SolFiV2SwapAccounts<'info>),

    #[cfg(feature = "manifest-swap")]
    Manifest(crate::programs::swap::manifest::ManifestSwapAccounts<'info>),

    #[cfg(feature = "heaven-swap")]
    Heaven(crate::programs::swap::heaven::HeavenSwapAccounts<'info>),

    #[cfg(feature = "aldrin-swap")]
    Aldrin(crate::programs::swap::aldrin::AldrinSwapAccounts<'info>),

    #[cfg(feature = "aldrin_v2-swap")]
    AldrinV2(crate::programs::swap::aldrin_v2::AldrinV2SwapAccounts<'info>),

    #[cfg(feature = "futarchy-swap")]
    Futarchy(crate::programs::swap::futarchy::FutarchySwapAccounts<'info>),

    #[cfg(feature = "gamma-swap")]
    Gamma(crate::programs::swap::gamma::GammaSwapAccounts<'info>),
}

/// Protocol-specific swap data enum for use with SwapContext
pub enum SwapData<'a> {
    #[cfg(feature = "perena-swap")]
    Perena(crate::programs::swap::perena::PerenaSwapData),

    #[cfg(feature = "solfi-swap")]
    SolFi(crate::programs::swap::solfi::SolFiSwapData),

    #[cfg(feature = "solfi_v2-swap")]
    SolFiV2(crate::programs::swap::solfi_v2::SolFiV2SwapData),

    #[cfg(feature = "manifest-swap")]
    Manifest(crate::programs::swap::manifest::ManifestSwapData),

    #[cfg(feature = "heaven-swap")]
    Heaven(crate::programs::swap::heaven::HeavenSwapData<'a>),

    #[cfg(feature = "aldrin-swap")]
    Aldrin(crate::programs::swap::aldrin::AldrinSwapData),

    #[cfg(feature = "aldrin_v2-swap")]
    AldrinV2(crate::programs::swap::aldrin_v2::AldrinV2SwapData),

    #[cfg(feature = "futarchy-swap")]
    Futarchy(crate::programs::swap::futarchy::FutarchySwapData),
}

impl<'a> SwapContext<'a> {
    /// Parse protocol-specific swap data from raw bytes based on the detected protocol.
    ///
    /// This allows parsing the extra instruction data after you've already detected
    /// the protocol from accounts via `try_from_swap_context`.
    pub fn try_from_swap_data(&self, data: &'a [u8]) -> Result<SwapData<'a>, ProgramError> {
        match self {
            #[cfg(feature = "perena-swap")]
            SwapContext::Perena(_) => Ok(SwapData::Perena(
                crate::programs::swap::perena::PerenaSwapData::try_from(data)?,
            )),

            #[cfg(feature = "solfi-swap")]
            SwapContext::SolFi(_) => Ok(SwapData::SolFi(
                crate::programs::swap::solfi::SolFiSwapData::try_from(data)?,
            )),

            #[cfg(feature = "solfi_v2-swap")]
            SwapContext::SolFiV2(_) => Ok(SwapData::SolFiV2(
                crate::programs::swap::solfi_v2::SolFiV2SwapData::try_from(data)?,
            )),

            #[cfg(feature = "manifest-swap")]
            SwapContext::Manifest(_) => Ok(SwapData::Manifest(
                crate::programs::swap::manifest::ManifestSwapData::try_from(data)?,
            )),

            #[cfg(feature = "heaven-swap")]
            SwapContext::Heaven(_) => Ok(SwapData::Heaven(
                crate::programs::swap::heaven::HeavenSwapData::try_from(data)?,
            )),

            #[cfg(feature = "aldrin-swap")]
            SwapContext::Aldrin(_) => Ok(SwapData::Aldrin(
                crate::programs::swap::aldrin::AldrinSwapData::try_from(data)?,
            )),

            #[cfg(feature = "aldrin_v2-swap")]
            SwapContext::AldrinV2(_) => Ok(SwapData::AldrinV2(
                crate::programs::swap::aldrin_v2::AldrinV2SwapData::try_from(data)?,
            )),

            #[cfg(feature = "futarchy-swap")]
            SwapContext::Futarchy(_) => Ok(SwapData::Futarchy(
                crate::programs::swap::futarchy::FutarchySwapData::try_from(data)?,
            )),

            #[allow(unreachable_patterns)]
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}

impl<'a> Swap<'a> for SwapContext<'a> {
    type Accounts = Self;
    type Data = SwapData<'a>;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        match (ctx, data) {
            #[cfg(feature = "perena-swap")]
            (SwapContext::Perena(accounts), SwapData::Perena(d)) => {
                crate::programs::swap::perena::Perena::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "solfi-swap")]
            (SwapContext::SolFi(accounts), SwapData::SolFi(d)) => {
                crate::programs::swap::solfi::SolFi::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "solfi_v2-swap")]
            (SwapContext::SolFiV2(accounts), SwapData::SolFiV2(d)) => {
                crate::programs::swap::solfi_v2::SolFiV2::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "manifest-swap")]
            (SwapContext::Manifest(accounts), SwapData::Manifest(d)) => {
                crate::programs::swap::manifest::Manifest::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "heaven-swap")]
            (SwapContext::Heaven(accounts), SwapData::Heaven(d)) => {
                crate::programs::swap::heaven::Heaven::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "aldrin-swap")]
            (SwapContext::Aldrin(accounts), SwapData::Aldrin(d)) => {
                crate::programs::swap::aldrin::Aldrin::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "aldrin_v2-swap")]
            (SwapContext::AldrinV2(accounts), SwapData::AldrinV2(d)) => {
                crate::programs::swap::aldrin_v2::AldrinV2::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "futarchy-swap")]
            (SwapContext::Futarchy(accounts), SwapData::Futarchy(d)) => {
                crate::programs::swap::futarchy::Futarchy::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "gamma-swap")]
            (SwapContext::Gamma(accounts), _) => crate::programs::swap::gamma::Gamma::swap_signed(
                accounts,
                in_amount,
                minimum_out_amount,
                &(),
                signer_seeds,
            ),

            #[allow(unreachable_patterns)]
            _ => Err(ProgramError::InvalidAccountData),
        }
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

/// Parses accounts and discriminates the protocol based on the first account (program ID).
///
/// This function returns a typed `SwapContext` that allows users to:
/// - Pattern match on the protocol type
/// - Access typed account fields for custom validation
/// - Inspect account properties before executing the swap
///
/// # Arguments
/// * `accounts` - Slice of accounts where the first account determines the protocol
///
/// # Returns
/// * `Ok(SwapContext)` - Typed context for the detected protocol
/// * `Err(ProgramError::NotEnoughAccountKeys)` - Empty account slice provided
/// * `Err(ProgramError::InvalidAccountData)` - No matching protocol found
pub fn try_from_swap_context<'info>(
    accounts: &'info [AccountView],
) -> Result<SwapContext<'info>, ProgramError> {
    let detector_account = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    #[cfg(feature = "perena-swap")]
    if address_eq(
        detector_account.address(),
        &crate::programs::swap::perena::PERENA_PROGRAM_ID,
    ) {
        let ctx = crate::programs::swap::perena::PerenaSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Perena(ctx));
    }

    #[cfg(feature = "solfi-swap")]
    if address_eq(
        detector_account.address(),
        &crate::programs::swap::solfi::SOLFI_PROGRAM_ID,
    ) {
        let ctx = crate::programs::swap::solfi::SolFiSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::SolFi(ctx));
    }

    #[cfg(feature = "solfi_v2-swap")]
    if address_eq(
        detector_account.address(),
        &crate::programs::swap::solfi_v2::SOLFI_V2_PROGRAM_ID,
    ) {
        let ctx = crate::programs::swap::solfi_v2::SolFiV2SwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::SolFiV2(ctx));
    }

    #[cfg(feature = "manifest-swap")]
    if address_eq(
        detector_account.address(),
        &crate::programs::swap::manifest::MANIFEST_PROGRAM_ID,
    ) {
        let ctx = crate::programs::swap::manifest::ManifestSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Manifest(ctx));
    }

    #[cfg(feature = "heaven-swap")]
    if address_eq(
        detector_account.address(),
        &crate::programs::swap::heaven::HEAVEN_PROGRAM_ID,
    ) {
        let ctx = crate::programs::swap::heaven::HeavenSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Heaven(ctx));
    }

    #[cfg(feature = "aldrin-swap")]
    if address_eq(
        detector_account.address(),
        &crate::programs::swap::aldrin::ALDRIN_PROGRAM_ID,
    ) {
        let ctx = crate::programs::swap::aldrin::AldrinSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Aldrin(ctx));
    }

    #[cfg(feature = "aldrin_v2-swap")]
    if address_eq(
        detector_account.address(),
        &crate::programs::swap::aldrin_v2::ALDRIN_V2_PROGRAM_ID,
    ) {
        let ctx = crate::programs::swap::aldrin_v2::AldrinV2SwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::AldrinV2(ctx));
    }

    #[cfg(feature = "futarchy-swap")]
    if address_eq(
        detector_account.address(),
        &crate::programs::swap::futarchy::FUTARCHY_PROGRAM_ID,
    ) {
        let ctx = crate::programs::swap::futarchy::FutarchySwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Futarchy(ctx));
    }

    #[cfg(feature = "gamma-swap")]
    if address_eq(
        detector_account.address(),
        &crate::programs::swap::gamma::GAMMA_PROGRAM_ID,
    ) {
        let ctx = crate::programs::swap::gamma::GammaSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Gamma(ctx));
    }

    Err(ProgramError::InvalidAccountData)
}

/// Convenience function: Parses accounts, discriminates protocol, and executes swap with PDA signing.
///
/// This is equivalent to calling `try_from_swap_context` followed by `SwapContext::swap_signed`.
/// For custom validation, use those functions separately instead.
///
/// # Arguments
/// * `accounts` - Slice of accounts where the first account determines the protocol
/// * `in_amount` - Amount of input tokens to swap
/// * `minimum_out_amount` - Minimum acceptable output amount (slippage protection)
/// * `data` - Protocol-specific additional instruction data
/// * `signer_seeds` - Seeds for PDA signing
///
/// # Returns
/// * `Ok(())` - Swap executed successfully
/// * `Err(ProgramError)` - Parsing, discrimination, or CPI failed
pub fn swap_signed(
    accounts: &[AccountView],
    in_amount: u64,
    minimum_out_amount: u64,
    data: &SwapData<'_>,
    signer_seeds: &[Signer],
) -> ProgramResult {
    let ctx = try_from_swap_context(accounts)?;
    SwapContext::swap_signed(&ctx, in_amount, minimum_out_amount, data, signer_seeds)
}

/// Convenience function: Parses accounts, discriminates protocol, and executes swap.
///
/// This is equivalent to calling `try_from_swap_context` followed by `SwapContext::swap`.
/// For custom validation, use those functions separately instead.
///
/// # Arguments
/// * `accounts` - Slice of accounts where the first account determines the protocol
/// * `in_amount` - Amount of input tokens to swap
/// * `minimum_out_amount` - Minimum acceptable output amount (slippage protection)
/// * `data` - Protocol-specific additional instruction data
///
/// # Returns
/// * `Ok(())` - Swap executed successfully
/// * `Err(ProgramError)` - Parsing, discrimination, or CPI failed
pub fn swap(
    accounts: &[AccountView],
    in_amount: u64,
    minimum_out_amount: u64,
    data: &SwapData<'_>,
) -> ProgramResult {
    swap_signed(accounts, in_amount, minimum_out_amount, data, &[])
}
