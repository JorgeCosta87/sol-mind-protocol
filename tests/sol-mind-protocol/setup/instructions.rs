use litesvm::{types::TransactionResult, LiteSVM};
use sol_mind_protocol_client::generated::{
    instructions::{
        CreateProjectBuilder, InitializeProtocolBuilder, ProjectFeesTransferBuilder,
        ProtocolFeesTransferBuilder, UpdateFeesBuilder, UpdateSingleFeeBuilder,
    },
    types::{Fee, FeesStructure, Operation},
};
use solana_pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID;

use crate::setup::accounts::AccountHelper;

pub struct Instructions;

impl Instructions {
    pub fn initialize_protocol(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        admins: Vec<Pubkey>,
        whitelist_transfer_addrs: Vec<Pubkey>,
        fees: FeesStructure,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda(program_id);

        let instruction = InitializeProtocolBuilder::new()
            .payer(payer)
            .protocol_config(protocol_config_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .admins(admins)
            .whitelist_transfer_addrs(whitelist_transfer_addrs)
            .fees(fees)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn create_project(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        project_id: u64,
        name: String,
        description: String,
        owner: Pubkey,
        authorities: Vec<Pubkey>,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (project_config_pda, _) =
            AccountHelper::find_project_pda(program_id, &owner, project_id);
        let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda(program_id);

        let instruction = CreateProjectBuilder::new()
            .owner(owner)
            .project_config(project_config_pda)
            .protocol_config(protocol_config_pda)
            .system_program(SYSTEM_PROGRAM_ID)
            .project_id(project_id)
            .name(name)
            .description(description)
            .authorities(authorities)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn update_fees(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        fees: FeesStructure,
        admin: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda(program_id);

        let instruction = UpdateFeesBuilder::new()
            .admin(admin)
            .protocol_config(protocol_config_pda)
            .fees(fees)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn update_single_fee(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        operation: Operation,
        fee: Fee,
        admin: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda(program_id);

        let instruction = UpdateSingleFeeBuilder::new()
            .admin(admin)
            .protocol_config(protocol_config_pda)
            .operation(operation)
            .fee(fee)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn project_fees_transfer(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        amount: u64,
        owner: Pubkey,
        to: Pubkey,
        project_id: u64,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (project_config_pda, _) =
            AccountHelper::find_project_pda(program_id, &owner, project_id);

        let instruction = ProjectFeesTransferBuilder::new()
            .owner(owner)
            .to(to)
            .project_config(project_config_pda)
            .amount(amount)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }

    pub fn protocol_fees_transfer(
        svm: &mut LiteSVM,
        program_id: &Pubkey,
        amount: u64,
        admin: Pubkey,
        to: Pubkey,
        payer: Pubkey,
        signing_keypairs: &[&Keypair],
    ) -> TransactionResult {
        let (protocol_config_pda, _) = AccountHelper::find_protocol_config_pda(program_id);

        let instruction = ProtocolFeesTransferBuilder::new()
            .admin(admin)
            .to(to)
            .protocol_config(protocol_config_pda)
            .amount(amount)
            .instruction();

        utils::send_transaction(svm, &[instruction], &payer, signing_keypairs)
    }
}
