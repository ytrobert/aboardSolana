use solana_program::{
    msg,
    program_error::ProgramError,
    program_error::ProgramError::InvalidInstructionData,
    pubkey::Pubkey,
};
use std::str;
use std::convert::TryInto;
// use crate::{
//     error::PerpError, 
//     error::PerpError::InvalidInstruction};

//yt: program API, (de)serializing instruction data
//<'a>named lifetime parameter have to be added
pub enum PerpetualInstruction {
    /// Initializes an perpetual account
    /// Accounts expected:
    /// 0. `[signer]` The admin of the perpetual account.
    ///    check signer
    /// 1. `[writable]` The perpetual account
    ///    new created, 
    ///    not initialized or signer is data.admin, account address is pda(unique)
    /// 2. `[]` system account for create_account cpi
    /// Safety:
    /// 1.The perpetual account is the only pda of the program.
    ///   1st initialise can be done by anyone that can be admin
    ///   fake perpetual account is owned by other program, because
    ///   create_account need new created account's signature 
    /// 2.admin can reconfigure the perpetual account, including change admin
    /// 3.Perpetual::unpack ensure initial
    InitPerpetual {
        /// The signer eth public key to check signature
        secp256k1_pubkey: [u8; 64],
        /// The gateway to send trades and withdraw
        gateway: Pubkey, //not used currently
        /// The admin of the Perpetual account
        admin: Pubkey,
    },

    /// Set Token supported map
    /// 0. `[signer]` The admin account to update the token map
    ///    check signer
    /// 1. `[writable]` The perpetual account
    ///    check signer is admin, initialized
    /// 2. `[]` program token account
    ///    check token owner is pda, owner is spl::token
    /// Safety:
    /// 1.only admin
    /// 2.program_token_account owner is pda(perpetual account address)
    /// 3.if program token account is not owned by spl, remove the token
    SetTokenMap {
        /// account type
        account_type: u8,
        /// token symbol
        symbol: String,
    },

    /// Initializes a new perpetual user account to hold user's trading data in the perpetual exchange
    /// 
    /// Accounts expected:
    /// 0. `[signer]` user
    /// 1. `[writable]` The perpetual user account to initialize.
    ///    check rent exempt, not initialized, address is pda
    /// 2. `[]` system account for create_account cpi
    /// Safety:
    /// 1. user account is pda
    /// 2. once
    InitAccount,

    /// Deposit
    /// Accounts expected:
    /// 0. `[signer]` The token owner of depositor's token account
    ///    check signer
    /// 1. `[writable]` The depositor's token account
    ///    check mint is in perpetual user account's btreemap
    /// 2. `[writable]` The program token account
    ///    check address is in perpetual account's btreemap
    /// 3. `[]` The perpetual user account
    ///    check owner, user is signer, initialized, to avoid can't withdraw
    /// 4. `[]` The perpetual account
    ///    check owner, initialized
    /// 5. `[]` The token program
    /// Safety:
    /// 1.fake depositor's token account v
    /// 2.fake program token account v
    /// 3.fake user account v
    /// 4.fake perpetual account v 
    /// 5.fake token program v
    /// 6.incorrect account type or symbol v
    /// 7.incorrect amount v
    Deposit {
        /// account type
        account_type: u8,
        /// token symbol
        symbol: String,
        /// transfer amount, because token Transfer is u64
        amount: u64,
    },

    /// Withdraw
    /// Accounts expected:
    /// 0. `[signer]` user account
    ///    check signer
    /// 1. `[writable]` The user token account
    ///    check mint, initialized, token owner is signer
    /// 2. `[writable]` The program token account
    ///    check mint, intialized, address is in perpetual accout btreemap
    /// 3. `[writable]` The perpetual user account
    ///    check initialized, user is signer, owner is program
    /// 4. `[]` The perpetual account
    ///    check initialized, owner is programid
    /// 5. `[]` The token program
    ///     check spl::id
    /// Safety:
    /// 1.fake user token account 
    /// 2.fake program token account 
    /// 3.fake user account 
    /// 4.fake perpetual account 
    /// 5.fake token program  v
    /// 6.incorrect account type or symbol 
    /// 7.incorrect amount
    /// 8.incorrect signature
    Withdraw {
        /// account type
        account_type: u8,
        /// token symbol
        symbol: String,
        /// withdraw amount
        amount: u64,
        /// withdraw id
        withdrawid: u64,
        /// time
        timestamp: u64,
        /// recovery id
        recovery_id: u8,
        /// signature
        signature: [u8; 64], //64
    },

}

impl PerpetualInstruction {
    /// Unpacks a byte buffer into a [PerpetualInstruction](enum.PerpetualInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(InvalidInstructionData)?;
        Ok(match tag {
            0 => {
                msg!("Perpetual instuction InitPerpetual");
                let (value, rest ) = rest.split_at(64);
                let secp256k1_pubkey= value.to_vec().try_into().unwrap();
                //let (signer_eth_pubkey, rest) = Self::unpack_pubkey(rest)?;
                let (gateway, rest) = Self::unpack_pubkey(rest)?;
                let (admin, _rest) = Self::unpack_pubkey(rest)?;
                Self::InitPerpetual{
                    secp256k1_pubkey,
                    gateway,
                    admin,
                }
            },
            1 => {
                msg!("Perpetual instuction SetTokenMap");
                let (&account_type, rest) = rest.split_first().ok_or(InvalidInstructionData)?;
                let (&symbol_len, rest) = rest.split_first().ok_or(InvalidInstructionData)?;
                let (symbol_raw, _rest) = rest.split_at(symbol_len as usize);
                let symbol = str::from_utf8(symbol_raw).unwrap().to_string();
                Self::SetTokenMap{
                    account_type,
                    symbol,
                }
            }
            2 => {
                msg!("Perpetual instuction InitAccount");
                //let (user, _rest) = Self::unpack_pubkey(rest)?;
                Self::InitAccount
            },
            3 => {
                msg!("Perpetual instuction Deposit");
                let (&account_type, rest) = rest.split_first().ok_or(InvalidInstructionData)?;
                let (&symbol_len, rest) = rest.split_first().ok_or(InvalidInstructionData)?;
                let (symbol_raw, rest) = rest.split_at(symbol_len as usize);
                let symbol = str::from_utf8(symbol_raw).unwrap().to_string();
                let amount = Self::unpack_u64(rest)?;
                Self::Deposit{
                    account_type,
                    symbol,
                    amount,
                }
            },
            4 => {
                msg!("Perpetual instuction Withdraw");
                let (&account_type, rest) = rest.split_first().ok_or(InvalidInstructionData)?;
                let (&symbol_len, rest) = rest.split_first().ok_or(InvalidInstructionData)?;
                let (symbol_raw, rest) = rest.split_at(symbol_len as usize);
                let symbol = str::from_utf8(symbol_raw).unwrap().to_string();
                let (value, rest) = rest.split_at(8);
                let amount = Self::unpack_u64(value)?;
                let (value, rest) = rest.split_at(8);
                let withdrawid = Self::unpack_u64(value)?;
                let (value, rest) = rest.split_at(8);
                let timestamp = Self::unpack_u64(value)?;
                let (&recovery_id, rest) = rest.split_first().ok_or(InvalidInstructionData)?;
                let (value,_rest ) = rest.split_at(64);
                let signature= value.to_vec().try_into().unwrap();
                Self::Withdraw{
                    account_type,
                    symbol,
                    amount,
                    withdrawid,
                    timestamp,
                    recovery_id,
                    signature,
                }
            },
            _ => return Err(InvalidInstructionData),
        })
    }

    pub fn unpack_bool(input: &u8) -> Result<bool, ProgramError> {
        let result = match input {
            0 => false,
            1 => true,
            _ => return Err(InvalidInstructionData),
        };
        Ok(result)
    }

    // pub fn unpack_amount(input: &[u8]) -> Result<u128, ProgramError> {
    //     let amount = input
    //         .get(..16)
    //         .and_then(|slice| slice.try_into().ok())
    //         .map(u128::from_le_bytes)
    //         .ok_or(InvalidInstructionData)?;
    //     Ok(amount)
    // }

    pub fn unpack_u64(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstructionData)?;
        Ok(amount)
    }

    pub fn unpack_u16(input: &[u8]) -> Result<u16, ProgramError> {
        let amount = input
            .get(..2)
            .and_then(|slice| slice.try_into().ok())
            .map(u16::from_le_bytes)
            .ok_or(InvalidInstructionData)?;
        Ok(amount)
    }

    pub fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        if input.len() >= 32 {
            let (key, rest) = input.split_at(32);
            let pk = Pubkey::try_from(key).unwrap();
            Ok((pk, rest))
        } else {
            Err(InvalidInstructionData)
        }
    }
}
