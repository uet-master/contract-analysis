use solana_program::{
    msg,
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::collections::HashMap;

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let contract_account = next_account_info(accounts_iter)?;

    let mut balances: HashMap<Pubkey, u64> = HashMap::new();
    let instruction = instruction_data[0];
    let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
    match instruction {
        0 => {
            msg!("User deposits {} lamports", amount);
            deposit(&mut balances, amount, user_account, contract_account)?;
        }
        1 => {
            msg!("User withdraws {} lamports", amount);
            withdraw(&mut balances, amount, user_account, contract_account)?;
        }
        _ => {
            msg!("Invalid action");
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    let balance = get_balance(&mut balances, user_account.key);
    msg!(
        "User {} has a remaining balance of {} lamports",
        user_account.key,
        balance
    );

    Ok(())
}

pub fn deposit(
    balances: &mut HashMap<Pubkey, u64>, 
    amount: u64,
    user_account: &AccountInfo,
    contract_account: &AccountInfo
) -> Result<(), ProgramError>  {
    if !user_account.is_signer {
        msg!("User account must sign the transaction");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let entry = balances.entry(*user_account.key).or_insert(0);
    *entry += amount;
    
    **user_account.try_borrow_mut_lamports()? -= amount;
    **contract_account.try_borrow_mut_lamports()? += amount;
    Ok(())
}

pub fn withdraw(
    balances: &mut HashMap<Pubkey, u64>,  
    amount: u64,
    user_account: &AccountInfo,
    contract_account: &AccountInfo
) -> Result<(), ProgramError> {
    if !contract_account.is_signer {
        msg!("Contract account must sign the transaction");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let balance = balances.get_mut(user_account.key).ok_or(ProgramError::InvalidAccountData)?;
    if *balance < amount {
        msg!("Insufficient balance for withdrawal");
        return Err(ProgramError::InsufficientFunds);
    }
    **contract_account.try_borrow_mut_lamports()? -= amount;
    **user_account.try_borrow_mut_lamports()? += amount;

    *balance -= amount;
    Ok(())
}

pub fn get_balance(balances: &mut HashMap<Pubkey, u64>, user: &Pubkey) -> u64 {
    *balances.get(user).unwrap_or(&0)
}