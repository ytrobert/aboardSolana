use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    system_instruction,
    msg,
    secp256k1_recover,
    keccak,
    log::sol_log_compute_units,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, clock::Clock, Sysvar},
};
use spl_token::state::Account as TokenAccount;
use spl_token::ID as TokenProgramId;
//use std::str; //convert::TryInto,
use crate::{
    error::PerpError, 
    instruction::PerpetualInstruction,
    state::{Perpetual, Account, TypeSymbol, MintProgram},

};



pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey, //not use. yt: need check owner?
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = PerpetualInstruction::unpack(instruction_data)?;
        //msg!("hello perpetual entrypoint");
        match instruction {
            PerpetualInstruction::InitPerpetual {
                secp256k1_pubkey,
                gateway,
                admin,
            } => {
                Self::process_init_perpetual(accounts, secp256k1_pubkey, gateway, admin, program_id)
            }
            PerpetualInstruction::SetTokenMap {
                account_type,
                symbol,
            } => {
                Self::process_set_tokenmap(accounts, account_type, symbol)
            }
            PerpetualInstruction::InitAccount => {
                Self::process_init_account(accounts, program_id)
            }
            PerpetualInstruction::Deposit {
                account_type,
                symbol,
                amount,
            } => {
                Self::process_deposit(accounts, account_type, symbol, amount, program_id)
            }
            PerpetualInstruction::Withdraw {
                account_type,
                symbol,
                amount,
                withdrawid,
                timestamp,
                recovery_id,
                signature,
            } => {
                Self::process_withdraw(accounts, account_type, symbol, amount, withdrawid, timestamp, recovery_id, signature, program_id)
            }
        }
    }

    fn process_init_perpetual(
        accounts: &[AccountInfo],
        secp256k1_pubkey: [u8; 64],
        gateway: Pubkey,
        admin: Pubkey,
        program_id: &Pubkey,
    ) -> ProgramResult {
        //msg!("process instuction 0");
        let account_info_iter = &mut accounts.iter();
        
        //1.signer account
        let admin_info = next_account_info(account_info_iter)?;
        //check signer
        if !admin_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        //2.perpetual account
        let perpetual_info = next_account_info(account_info_iter)?;
        if perpetual_info.data_is_empty() {
            let lamports_required = (Rent::get()?).minimum_balance(Perpetual::LEN);
            let (_pda, bump_seed) = Pubkey::find_program_address(&[b"perpetual"], program_id);
            //3.system account
            let system_account = next_account_info(account_info_iter)?;
            //create account with pda as publickey must be cpi, because signature needed
            invoke_signed(
                &system_instruction::create_account(
                    admin_info.key,
                    perpetual_info.key,
                    lamports_required,
                    Perpetual::LEN as u64,
                    program_id,
                ),
                &[
                    admin_info.clone(),
                    perpetual_info.clone(),
                    system_account.clone(),
                ],
                &[&[&b"perpetual"[..], &[bump_seed]]],
            )?;
            let mut perpetual = Perpetual::unpack_unchecked(&perpetual_info.data.borrow())?;
            perpetual.is_initialized = true;
            perpetual.secp256k1_pubkey = secp256k1_pubkey;
            perpetual.gateway = gateway;
            perpetual.admin = admin;
            perpetual.bump_seed = bump_seed;
            msg!("Perpetual initial info:{:?}", perpetual);
            Perpetual::pack(perpetual, &mut perpetual_info.data.borrow_mut())?;
        } else {
            //data unpack
            let mut perpetual = Perpetual::unpack(&perpetual_info.data.borrow())?;
            if perpetual.admin != *admin_info.key {
                msg!("Perpetual incorrect admin:{:?}", perpetual.admin);
                return Err(ProgramError::InvalidAccountData);
            }
            perpetual.secp256k1_pubkey = secp256k1_pubkey;
            perpetual.gateway = gateway;
            perpetual.admin = admin;
            msg!("Perpetual info:{:?}", perpetual);
            Perpetual::pack(perpetual, &mut perpetual_info.data.borrow_mut())?;
        }
        //msg!("perpetual account initialize:{}", perpetual_info.key);
        Ok(())
    }

    fn process_set_tokenmap (
        accounts: &[AccountInfo],
        account_type: u8,
        symbol: String,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        //1.signer account
        let admin_info = next_account_info(account_info_iter)?;
        //check signer
        if !admin_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        //2.perpetual account
        let perpetual_info = next_account_info(account_info_iter)?;
        //data unpack
        let mut perpetual = Perpetual::unpack(&perpetual_info.data.borrow())?;
        //check admin
        if *admin_info.key != perpetual.admin {
            msg!("Perpetual incorrect admin:{:?}", perpetual.admin);
            return Err(ProgramError::InvalidAccountData);
        }

        //3.program token account
        let program_token_account_info = next_account_info(account_info_iter)?;
        //check account owner
        if *program_token_account_info.owner == TokenProgramId {
            //data unpack
            let token_info = TokenAccount::unpack(&program_token_account_info.try_borrow_data()?)?;
            //check token owner
            if *perpetual_info.key != token_info.owner {
                msg!("Perpetual incorrect owner:{:?}", token_info.owner);
                return Err(ProgramError::InvalidAccountData);
            }
            perpetual.token_map.insert(TypeSymbol{account_type, symbol}, MintProgram{mint: token_info.mint, program_token_account: *program_token_account_info.key});         
        } else {
            //remove
            perpetual.token_map.remove(&TypeSymbol{account_type, symbol});
        }
        msg!("Perpetual info:{:?}", perpetual);
        Perpetual::pack(perpetual, &mut perpetual_info.data.borrow_mut())?;
        Ok(())
    }

    fn process_init_account(
        accounts: &[AccountInfo],
        //user: Pubkey,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        //1.user signer account
        let user_info = next_account_info(account_info_iter)?;
        //check signer
        if !user_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        //2.perpetual user account
        let account_info = next_account_info(account_info_iter)?;
        if account_info.data_is_empty() {
            let lamports_required = (Rent::get()?).minimum_balance(Account::LEN);
            let (_pda, bump_seed) = Pubkey::find_program_address(&[b"perpetual", user_info.key.as_ref()], program_id);
            let system_account = next_account_info(account_info_iter)?;
            invoke_signed(
                &system_instruction::create_account(
                    user_info.key,
                    account_info.key,
                    lamports_required,
                    Account::LEN as u64,
                    program_id,
                ),
                &[
                    user_info.clone(),
                    account_info.clone(),
                    system_account.clone(),
                ],
                &[&[&b"perpetual"[..], user_info.key.as_ref(), &[bump_seed]]],
            )?;
            let mut account = Account::unpack_unchecked(&account_info.data.borrow())?;
            account.is_initialized = true;
            account.user = *user_info.key;
            msg!("Perpetual account:{:?}", account);
            Account::pack(account, &mut account_info.data.borrow_mut())?;
            Ok(())
        } else {
            return Err(ProgramError::AccountAlreadyInitialized);
        }   
    }

    fn process_deposit(
        accounts: &[AccountInfo],
        account_type: u8,
        symbol: String,
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        
        let account_info_iter = &mut accounts.iter();

        //1.signer account
        let token_owner_info = next_account_info(account_info_iter)?;
        //check sign
        if !token_owner_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        
        //2.user token account
        let user_token_account_info = next_account_info(account_info_iter)?;
        //check owner is spl::token
        // if *user_token_account_info.owner != TokenProgramId {
        //     return Err(ProgramError::InvalidAccountData);
        // }
        //unpack
        let token_info = TokenAccount::unpack(&user_token_account_info.try_borrow_data()?)?;

        //3.program token account
        let program_token_account_info = next_account_info(account_info_iter)?;
        
        //4.perpetual user account
        let account_info = next_account_info(account_info_iter)?;
        //check owner is programid
        if account_info.owner != program_id {
            msg!("Perpetual incorrect user account:{}", account_info.owner);
            return Err(ProgramError::IncorrectProgramId);
        }
        //unpack
        let account = Account::unpack(&account_info.data.borrow())?;
        //check account's user is the signer
        if account.user != *token_owner_info.key {
            msg!("Perpetual incorrect user:{}", account.user);
            return Err(ProgramError::InvalidAccountData);
        }
        
        //5.perpetual account
        let admin_info = next_account_info(account_info_iter)?;
        //unpack
        let admin_data = Perpetual::unpack(&admin_info.try_borrow_data()?)?;
        //check owner is programid
        if admin_info.owner != program_id {
            msg!("Perpetual incorrect perpetual account:{}", admin_info.owner);
            return Err(ProgramError::IncorrectProgramId);
        }
        let mint_program = admin_data.token_map[&TypeSymbol{account_type, symbol:symbol.clone()}];
        //check mint is token account's mint
        if token_info.mint != mint_program.mint {
            msg!("Perpetual incorrect mint:{}", mint_program.mint);
            return Err(ProgramError::InvalidAccountData);
        }
        //check address is program token account address
        if mint_program.program_token_account != *program_token_account_info.key {
            msg!("Perpetual incorrect program token account:{}", mint_program.program_token_account);
            return Err(ProgramError::InvalidAccountData);
        }

        //6.token program account
        let token_program_info = next_account_info(account_info_iter)?;
        //check token programId
        if *token_program_info.key != TokenProgramId {
            return Err(ProgramError::IncorrectProgramId);
        }
        
        msg!("Perpetual deposit CPI");
        //transfer token from user_token_account_info to program_token_account_info
        let transfer_to_program_ix = spl_token::instruction::transfer(
            token_program_info.key,
            user_token_account_info.key,
            program_token_account_info.key,
            token_owner_info.key,
            &[&token_owner_info.key],
            amount,
        )?;
        invoke(
            &transfer_to_program_ix,
            &[
                user_token_account_info.clone(),
                program_token_account_info.clone(),
                token_owner_info.clone(),
                token_program_info.clone(),
            ],
        )?;

        msg!("Perpetual deposit account:{} type:{} symbol:{} amount:{}", 
              account.user, account_type, symbol, amount);
        msg!("Perpetual user account:{:?}", account);
        Ok(())
    }

    fn process_withdraw(
        accounts: &[AccountInfo],
        account_type: u8,
        symbol: String,
        amount: u64,
        withdrawid: u64,
        timestamp: u64,
        recovery_id: u8,
        signature: [u8;64],
        program_id: &Pubkey,
    ) -> ProgramResult {
        sol_log_compute_units();
        
        let account_info_iter = &mut accounts.iter();

        //1.signer account
        let signer_info = next_account_info(account_info_iter)?;
        //check signer
        if !signer_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        //2.user token account
        let dest_token_account_info = next_account_info(account_info_iter)?;
        //unpack
        let dest_token_account_data = TokenAccount::unpack(&dest_token_account_info.try_borrow_data()?)?;
        //check owner is signer
        if *signer_info.key != dest_token_account_data.owner {
            msg!("Perpetual incorrect token owner:{}", dest_token_account_data.owner);
            return Err(ProgramError::InvalidAccountData);
        }
        
        //3.program token account
        let program_token_account_info = next_account_info(account_info_iter)?;

        //4.perpetual user account
        let account_info = next_account_info(account_info_iter)?;
        //unpack
        let mut account = Account::unpack(&account_info.data.borrow())?;
        //check user is signer
        if account.user != *signer_info.key {
            msg!("Perpetual incorrect user:{}", account.user);
            return Err(ProgramError::InvalidAccountData);
        }

        //5.perpetual account
        let admin_info = next_account_info(account_info_iter)?;
        if admin_info.owner != program_id {
            msg!("Perpetual incorrect perpetual account:{}", admin_info.owner);
            return Err(ProgramError::IncorrectProgramId);
        }
        //unpack
        let admin_data = Perpetual::unpack(&admin_info.try_borrow_data()?)?;
        let mint_program = admin_data.token_map[&TypeSymbol{account_type, symbol: symbol.clone()}];
        //check mint is token account's mint
        if dest_token_account_data.mint != mint_program.mint {
            msg!("Perpetual incorrect mint:{}", mint_program.mint);
            return Err(ProgramError::InvalidAccountData);
        }
        //check address is program token account address
        if mint_program.program_token_account != *program_token_account_info.key {
            msg!("Perpetual incorrect program token account:{}", mint_program.program_token_account);
            return Err(ProgramError::InvalidAccountData);
        }

        //6.token program account
        let token_program_info = next_account_info(account_info_iter)?;
        //check token programId
        if *token_program_info.key != TokenProgramId {
            return Err(ProgramError::IncorrectProgramId);
        }

        //check withdrawid
        let account_withdraw_id: u64 = match account.withdraw_id.get(&account_type) {
            Some(&wdid) => wdid,
            None => 0,
        };
        if account_withdraw_id >= withdrawid {
            msg!("Perpetual incorrect withdrawId:{}", account_withdraw_id);
            return Err(PerpError::WithdrawIdFail.into());
        }

        //check timestamp
        let now_timestamp = Clock::get()?.unix_timestamp;
        if timestamp <= now_timestamp.try_into().unwrap() {
            msg!("Perpetual incorrect timestamp:{}", now_timestamp);
            return Err(ProgramError::InvalidInstructionData);
        }
        
        //check signature
        //maybe vec if length
        //msg!("user_account:{}", user_account);
        let user_account_string = account.user.to_string();
        let user_account_bytes = user_account_string.as_bytes();
        //msg!("user_account_bytes:{}", user_account_bytes.len());
        let symbol_bytes = symbol.as_bytes();
        let mut dst_data = vec![];
        let mut data_offset = user_account_bytes.len();
        dst_data.resize(
            data_offset
            .saturating_add(symbol.len())
            .saturating_add(1+8+8+8),
        0,
        );
        dst_data[..data_offset].copy_from_slice(user_account_bytes);
        dst_data[data_offset..data_offset+1].copy_from_slice(&account_type.to_be_bytes());
        data_offset = data_offset + 1;
        dst_data[data_offset..data_offset+symbol.len()].copy_from_slice(symbol_bytes);
        data_offset = data_offset + symbol.len();
        dst_data[data_offset..data_offset+8].copy_from_slice(&amount.to_be_bytes());
        data_offset = data_offset + 8;
        dst_data[data_offset..data_offset+8].copy_from_slice(&withdrawid.to_be_bytes());
        data_offset = data_offset + 8;
        dst_data[data_offset..data_offset+8].copy_from_slice(&timestamp.to_be_bytes());
        //msg!("dst_data:{:?}", dst_data);
        let hash = keccak::hash(&dst_data); //may need try_into()
        //msg!("hash:{:?} recovery_id:{} signature:{:?}", hash.to_bytes(), recovery_id, signature);
        //check signature
        let pubkey_secp256k1 = secp256k1_recover::secp256k1_recover(&hash.to_bytes(), recovery_id, &signature).unwrap();
        //msg!("recovery pubkey_secp256k1:{:?}", pubkey_secp256k1.to_bytes());
        //msg!("input pubkey_secp256k1:{:?}", admin_data.secp256k1_pubkey);
        if pubkey_secp256k1.to_bytes() != admin_data.secp256k1_pubkey {
            msg!("Perpetual signature mismatch");
            return Err(PerpError::SignatureMismatch.into());
        }

        //cpi with pda
        msg!("Perpetual withdraw CPI");
        let transfer_to_dest_ix = spl_token::instruction::transfer(
            token_program_info.key,
            program_token_account_info.key,
            dest_token_account_info.key,
            &admin_info.key,
            &[&admin_info.key],
            amount,
        )?;
        invoke_signed(
            &transfer_to_dest_ix,
            &[
                program_token_account_info.clone(),
                dest_token_account_info.clone(),
                admin_info.clone(),
                token_program_info.clone(),
            ],
            &[&[&b"perpetual"[..], &[admin_data.bump_seed]]],
        )?;

        //update account
        account.withdraw_id.insert(account_type, withdrawid);
        msg!("Perpetual user account:{:?}", account);
        Account::pack(account, &mut account_info.data.borrow_mut())?;
        msg!("Perpetual withdraw account:{} type:{} symbol:{} amount:{} withdrawid:{}", 
              signer_info.key, account_type, symbol, amount, withdrawid);
        Ok(())

    }

}

