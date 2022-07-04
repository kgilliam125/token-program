
use borsh::{BorshDeserialize};
use solana_program::{
   account_info::{next_account_info, AccountInfo},
   entrypoint::ProgramResult,
   msg,
   program_error::ProgramError,
   pubkey::Pubkey,
};

use crate::instruction::TokenInstruction;
use crate::state::{Token, TokenAccount};

pub struct Processor {}

impl Processor {
   pub fn process_instruction(
       _program_id: &Pubkey,
       accounts: &[AccountInfo],
       instruction_data: &[u8],
   ) -> ProgramResult {
       let instruction = TokenInstruction::try_from_slice(instruction_data)
           .map_err(|_| ProgramError::InvalidInstructionData)?;
       let accounts_iter = &mut accounts.iter();
       msg!("Instruction: {:?}",instruction);
       match instruction {
           TokenInstruction::CreateToken => {
               msg!("Instruction: Create Token");
               let token_master_account = 
next_account_info(accounts_iter)?;
                let token_authority = next_account_info(accounts_iter)?;
                let mut token = 
Token::load_unchecked(token_master_account)?;

                token.authority = *token_authority.key;
                token.supply = 0;
                token.save(token_master_account)?
           }

           TokenInstruction::CreateTokenAccount => {
               msg!("Instruction: Create Token Account");

               let token_account_acct = next_account_info(accounts_iter)?;
               let token_master_account = next_account_info(accounts_iter)?;
               let owner = next_account_info(accounts_iter)?;
               let mut token_account = TokenAccount::load_unchecked(token_account_acct)?;

               token_account.owner = *owner.key;
               token_account.token = *token_master_account.key;
               token_account.amount = 0;
               token_account.save(token_account_acct)?;
           }

           TokenInstruction::Mint { amount } => {
               msg!("Instruction: Mint");

               let token_account_acct = next_account_info(accounts_iter)?;
               let token_master_account = next_account_info(accounts_iter)?;
               let mut token_account = TokenAccount::load_unchecked(token_account_acct)?;
               let mut token = Token::load(token_master_account)?;

               let token_authority = next_account_info(accounts_iter)?;
               if !token_authority.is_signer {
                   msg!("Only the token owner can mint tokens.");
                   return Err(ProgramError::MissingRequiredSignature);
               }

               token.supply += amount;
               token_account.amount += amount;

               token_account.save(token_account_acct)?;
               token.save(token_master_account)?;
           }
           TokenInstruction::Transfer { amount } => {
               msg!("Instruction: Transfer");
               
               //get account info for from and to token accounts, as well as master token account
               let from_token_acct = next_account_info(accounts_iter)?;
               let to_token_acct = next_account_info(accounts_iter)?;
               let owner = next_account_info(accounts_iter)?;
               let mut src_token_account = TokenAccount::load(from_token_acct)?;
               let mut dst_token_account = TokenAccount::load(to_token_acct)?;

               if src_token_account.amount <= amount {
                   msg!("Not enough tokens to transer");
                   return Err(ProgramError::InsufficientFunds);
               }

               if !owner.is_signer {
                   msg!("Not the token owner signing the transaction");
                   return Err(ProgramError::MissingRequiredSignature);
               }

                //ensure the owner passed in is the actual owner of the token account
                if !(src_token_account.owner == *owner.key) {
                    msg!("Not the token account owner signing the transaction");
                    return Err(ProgramError::MissingRequiredSignature);
                }
    
                //update values in from and to accounts, then save new contents of both accounts
                src_token_account.amount -= amount;
                dst_token_account.amount += amount;
                src_token_account.save(from_token_acct)?;
                dst_token_account.save(to_token_acct)?;
           }
       }
       Ok(())
   }
}