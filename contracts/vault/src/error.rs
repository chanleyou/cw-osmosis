use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("ZeroBalance")]
    ZeroBalance {},

    #[error("CompoundFailed")]
    CompoundFailed {},

    #[error("UnknownReplyId: {id:?}")]
    UnknownReplyId { id: u64 },
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
