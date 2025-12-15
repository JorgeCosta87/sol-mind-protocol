mod setup;

use sol_mind_protocol_client::dac_manager::types::{AgentStatus, ComputeNodeStatus, TaskStatus};
use solana_sdk::signature::Signer;

use crate::setup::test_data::*;
use setup::{AccountHelper, Instructions, TestFixture};

#[test]
fn test_register_compute_node() {
    let mut fixture = TestFixture::new();
    let node_pubkey = fixture.compute_node.pubkey();

    let result = Instructions::register_compute_node(
        &mut fixture.svm,
        node_pubkey,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        &[
            &fixture.payer.insecure_clone(),
            &fixture.project_owner.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);
            let compute_node_info =
                AccountHelper::get_compute_node_info(&fixture.svm, &node_pubkey);

            assert_eq!(compute_node_info.owner, fixture.project_owner.pubkey());
            assert_eq!(compute_node_info.node_pubkey, node_pubkey);
            assert_eq!(compute_node_info.status, ComputeNodeStatus::Pending);
            assert_eq!(compute_node_info.node_info_cid, None);
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_claim_compute_node() {
    let mut fixture = TestFixture::new();
    let node_pubkey = fixture.compute_node.pubkey();
    let node_info_cid = "QmExample123".to_string();

    Instructions::register_compute_node(
        &mut fixture.svm,
        node_pubkey,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        &[
            &fixture.payer.insecure_clone(),
            &fixture.project_owner.insecure_clone(),
        ],
    )
    .expect("Failed to register compute node");

    let result = Instructions::claim_compute_node(
        &mut fixture.svm,
        node_pubkey,
        node_info_cid.clone(),
        fixture.payer.pubkey(),
        &[
            &fixture.payer.insecure_clone(),
            &fixture.compute_node.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);
            let compute_node_info =
                AccountHelper::get_compute_node_info(&fixture.svm, &node_pubkey);

            assert_eq!(compute_node_info.owner, fixture.project_owner.pubkey());
            assert_eq!(compute_node_info.node_pubkey, node_pubkey);
            assert_eq!(compute_node_info.status, ComputeNodeStatus::Approved);
            assert_eq!(compute_node_info.node_info_cid, Some(node_info_cid));
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_create_agent() {
    let mut fixture = TestFixture::new()
        .with_register_compute_node()
        .with_claim_compute_node(None);

    let compute_node_info_pda =
        AccountHelper::find_compute_node_info_pda(&fixture.compute_node.pubkey()).0;

    let result = Instructions::create_agent(
        &mut fixture.svm,
        AGENT_ID,
        true,
        compute_node_info_pda,
        fixture.project_owner.pubkey(),
        fixture.payer.pubkey(),
        &[
            &fixture.payer.insecure_clone(),
            &fixture.project_owner.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);
            let agent_pda =
                AccountHelper::find_agent_pda(&fixture.project_owner.pubkey(), AGENT_ID).0;
            let agent =
                AccountHelper::get_agent(&fixture.svm, &fixture.project_owner.pubkey(), AGENT_ID);
            let task_data = AccountHelper::get_task_data(&fixture.svm, &agent_pda);

            assert_eq!(agent.owner, fixture.project_owner.pubkey());
            assert_eq!(agent.agent_id, AGENT_ID);
            assert_eq!(agent.compute_node, fixture.compute_node.pubkey());
            assert_eq!(task_data.compute_node, fixture.compute_node.pubkey());
            assert_eq!(task_data.data, Vec::<u8>::new());
            assert_eq!(task_data.status, TaskStatus::Ready);
            assert_eq!(task_data.result, Vec::<u8>::new());
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_submit_task() {
    let mut fixture = TestFixture::new()
        .with_register_compute_node()
        .with_claim_compute_node(None)
        .with_create_agent();

    let agent_pda = AccountHelper::find_agent_pda(&fixture.project_owner.pubkey(), AGENT_ID).0;
    let task_data = vec![1, 2, 3, 4];

    let result = Instructions::submit_task(
        &mut fixture.svm,
        task_data.clone(),
        agent_pda,
        fixture.compute_node.pubkey(),
        fixture.payer.pubkey(),
        &[
            &fixture.payer.insecure_clone(),
            &fixture.compute_node.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);

            let task_data_account = AccountHelper::get_task_data(&fixture.svm, &agent_pda);

            assert_eq!(task_data_account.data, task_data);
            assert_eq!(
                task_data_account.compute_node,
                fixture.compute_node.pubkey()
            );
            assert_eq!(task_data_account.status, TaskStatus::Pending);
            assert_eq!(task_data_account.result, Vec::<u8>::new());
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}

#[test]
fn test_activate_agent() {
    let mut fixture = TestFixture::new()
        .with_register_compute_node()
        .with_claim_compute_node(None)
        .with_create_agent();

    let agent_before = AccountHelper::get_agent(&fixture.svm, &fixture.project_owner.pubkey(), AGENT_ID);

    assert_eq!(agent_before.status, AgentStatus::Pending);

    let result = Instructions::activate_agent(
        &mut fixture.svm,
        AGENT_ID,
        fixture.project_owner.pubkey(),
        fixture.compute_node.pubkey(),
        fixture.payer.pubkey(),
        &[
            &fixture.payer.insecure_clone(),
            &fixture.compute_node.insecure_clone(),
        ],
    );

    match result {
        Ok(result) => {
            utils::print_transaction_logs(&result);
            let agent_after = AccountHelper::get_agent(&fixture.svm, &fixture.project_owner.pubkey(), AGENT_ID);

            assert_eq!(agent_after.owner, fixture.project_owner.pubkey());
            assert_eq!(agent_after.agent_id, AGENT_ID);
            assert_eq!(agent_after.compute_node, fixture.compute_node.pubkey());
            assert_eq!(agent_after.status, AgentStatus::Active);
        }
        Err(e) => {
            panic!("Transaction failed: {:?}", e);
        }
    }
}
