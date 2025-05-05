use anchor_lang::prelude::*;

pub mod error;
pub mod states;
pub mod instructions;

pub use error::CustomError;
pub use instructions::*;

declare_id!("CCxyEA1iGXgbDTseF52h3vPK867Gjm2JiPaA15xH86gY");

pub use instructions::{
    init_config,
    initialize_payment,
    execute_payment,
    withdraw_liquidity,
    deposit_liquidity,
    update_config,
    settle,
    // allocate_gap is internal
};