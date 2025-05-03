use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Invalid basis point")]
    InvalidBasisPoint,
    #[msg("Invalid state for this operation")]
    InvalidState,
    #[msg("Deal not yet mature")]
    NotMature,
    #[msg("Unsupported token")]
    UnsupportedToken,
    #[msg("Input amount too small")]
    InputTooSmall,
}
