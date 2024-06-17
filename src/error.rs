use thiserror::Error;  //can use #[error("Invalid Instruction")]
//yt: program specific errors
use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum PerpError {
    /// Signature Mismatch
    #[error("Signature Mismatch")]
    SignatureMismatch,
    /// Withdraw ID Fail
    #[error("Withdraw ID Fail")]
    WithdrawIdFail,
    /// Account Not Empty
    #[error("Account Not Empty")]
    AccountNotEmpty,
    /// User Already In Use
    #[error("User Already In Use")]
    UserAlreadyInUse,
    /// Incorrect Admin
    #[error("Incorrect Admin")]
    IncorrectAdmin,
}

//yt: From trait to covert PerpError to ProgramError
impl From<PerpError> for ProgramError {
    fn from(e: PerpError) -> Self {
        //convert PerpError to ProgramError
        //can print: custom program error: 0x0
        ProgramError::Custom(e as u32)
    }
}
