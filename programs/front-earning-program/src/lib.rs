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
    invest_gap,
    swap_stable,
    withdraw_investment,
    execute_payment,
};