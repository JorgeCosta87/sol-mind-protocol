// Load environment variables from .env file
import 'dotenv/config'
import prompts from 'prompts'

// Solana Client SDK
import { Address, createSolanaClient, getMonikerFromGenesisHash, isAddress, lamportsToSol } from 'gill'
// Solana Client SDK (Node.js)
import { loadKeypairSignerFromFile } from 'gill/node'

import { DacManagerClient } from './dacManagerClient.js'
import { TaskStatus, ComputeNodeStatus } from './generated/dac-manager/index.js'

// Get the Solana RPC endpoint from the environment variable or default to devnet
const urlOrMoniker = process.env.SOLANA_RPC_ENDPOINT || 'devnet'
const client = createSolanaClient({ urlOrMoniker })

console.log(urlOrMoniker)
// Load the keypair from the .env file or use the default (~/.config/solana/id.json)
const signer = await loadKeypairSignerFromFile(process.env.SOLANA_SIGNER_PATH)

// Get the balance of the provided address and print it to the console
async function showBalance(address: Address) {
  const balance = await client.rpc.getBalance(address).send()
  console.log(`Address : ${address}`)
  console.log(`Balance : ${lamportsToSol(balance.value)} SOL`)
}

// Welcome message
console.log('Gm! Say hi to your new Solana script!')

// Show the endpoint and cluster
console.log(`Endpoint: ${urlOrMoniker.split('?')[0]}`)
const cluster = getMonikerFromGenesisHash(await client.rpc.getGenesisHash().send())
console.log(`Cluster : ${cluster}`)

// Show the signer's address and balance
console.log('Signer Keypair')
await showBalance(signer.address)

// Initialize DAC Manager Client
const dacManager = new DacManagerClient(client);

// Compute node configuration
const nodePubkey = '6DgNU6x3TF8TL4VWYcA4SP36pvszjj71MSwaoD52LnHQ' as Address;
const nodeInfoCid = 'QmExample123'; // Replace with actual CID
const owner = signer.address;

try {
  // Step 1: Register Compute Node
  console.log('\n=== Step 1: Registering Compute Node ===');
  const computeNodeAddress = await dacManager.deriveComputeNodeAddress(nodePubkey);
  let computeNode = await dacManager.getComputeNode(nodePubkey);
  
  if (computeNode.exists) {
    console.log('Compute node already registered');
    console.log(`Compute Node Address: ${computeNodeAddress}`);
    console.log(`Status: ${ComputeNodeStatus[computeNode.data.status]}`);
    console.log(`Owner: ${computeNode.data.owner}`);
    console.log(`Node Pubkey: ${computeNode.data.nodePubkey}`);
  } else {
    console.log('Compute node does not exist, registering...');
    const { signature, computeNodeAddress: registeredAddress } = await dacManager.registerComputeNode({
      payer: signer,
      owner: signer,
      nodePubkey,
    });
    console.log(`Transaction sent: ${signature}`);
    console.log(`Compute Node address: ${registeredAddress}`);
    console.log('Transaction confirmed!');
    computeNode = await dacManager.getComputeNode(nodePubkey);
  }
  
  if (!computeNode.exists) {
    throw new Error('Failed to register or fetch compute node');
  }

  // Step 2: Wait for Compute Node to Claim
  console.log('\n=== Step 2: Waiting for Compute Node to Claim ===');
  console.log(`Current status: ${ComputeNodeStatus[computeNode.data.status]}`);
  
  if (computeNode.data.status === ComputeNodeStatus.Approved) {
    console.log('✅ Compute node is already approved!');
  } else {
    console.log('⏳ Waiting for compute node to claim...');
    console.log(`Monitoring compute node: ${computeNodeAddress}`);
    
    await new Promise<void>((resolve, reject) => {
      // Also poll periodically as a fallback
      const pollInterval = setInterval(async () => {
        const currentNode = await dacManager.getComputeNode(nodePubkey);
        if (currentNode.exists && currentNode.data.status === ComputeNodeStatus.Approved) {
          clearInterval(pollInterval);
          console.log('✅ Compute node approved (detected via polling)');
          resolve();
        }
      }, 5000);
      
      // Timeout after 5 minutes
      setTimeout(() => {
        clearInterval(pollInterval);
        reject(new Error('Timeout waiting for compute node to claim'));
      }, 5 * 60 * 1000);
    });
  }

  // Step 3: Create Agent (using the approved compute node)
  console.log('\n=== Step 3: Creating Agent ===');
  const agentId = 1n;
  const isPublic = false;
  const agentAddress = await dacManager.deriveAgentAddress(signer.address, agentId);
  let agent = await dacManager.getAgent(signer.address, agentId);
  
  if (agent.exists) {
    console.log('Agent already exists');
  } else {
    console.log('Agent does not exist, creating...');
    const { signature, agentAddress: createdAddress } = await dacManager.createAgent({
      payer: signer,
      owner: signer,
      agentId,
      computeNodeOwner: owner,
      computeNodePubkey: nodePubkey,
      public: isPublic,
    });
    console.log(`Transaction sent: ${signature}`);
    console.log(`Agent address: ${createdAddress}`);
    console.log('Transaction confirmed!');
    agent = await dacManager.getAgent(signer.address, agentId);
  }
  
  if (!agent.exists) {
    throw new Error('Failed to create or fetch agent');
  }
  
  // Print agent info
  console.log('\n--- Agent Info ---');
  console.log(`Agent Address: ${agentAddress}`);
  console.log(`Agent ID: ${agent.data.agentId}`);
  console.log(`Agent Owner: ${agent.data.owner}`);
  console.log(`Agent Compute Node: ${agent.data.computeNode}`);
  console.log(`Agent Public: ${agent.data.public}`);
  const taskDataAddress = await dacManager.deriveTaskDataAddress(agentAddress);
  console.log(`Task Data Address: ${taskDataAddress}`);
  const taskData = await dacManager.getTaskData(agentAddress);
  if (taskData.exists) {
    console.log(`Task Status: ${TaskStatus[taskData.data.status]}`);
  } else {
    console.log('Task data does not exist');
  }
 
  // Step 4: Submit Task
  console.log('\n=== Step 4: Submitting Task ===');
  const taskDataBytes = new Uint8Array([1, 2, 2, 4, 5]);
  
  if (taskData.exists && taskData.data.status !== TaskStatus.Ready) {
    console.log('Task not ready, cannot submit');
  } else {
    console.log('Submitting task...');

    const agentInfo = await dacManager.getAgent(signer.address, agentId);
    if (agentInfo.exists) {
      console.log(`Agent is ${agentInfo.data.public ? 'public' : 'private'}`);
      if (!agentInfo.data.public) {
        console.log('Agent is private - only owner can submit tasks');
      }
    }
    
    const taskSignature = await dacManager.submitTask({
      payer: signer,
      submitter: signer,
      agent: agentAddress,
      data: taskDataBytes,
    });
    console.log(`Task submitted! Transaction signature: ${taskSignature}`);
  }
  
  const updatedTaskData = await dacManager.getTaskData(agentAddress);
  if (updatedTaskData.exists) {
    console.log('\n--- Task Data Info ---');
    console.log(`Task Data Address: ${updatedTaskData.address}`);
    const statusNames = ['Ready', 'Pending', 'Processing'];
    console.log(`Task Status: ${TaskStatus[updatedTaskData.data.status]}`);
    console.log(`Task Data: ${Array.from(updatedTaskData.data.data).join(', ')}`);
    console.log(`Compute Node: ${updatedTaskData.data.computeNode}`);
  }

  // Step 5: Claim Task (by compute node)
  console.log('\n=== Step 5: Claiming Task ===');
  const taskDataForClaim = await dacManager.getTaskData(agentAddress);
  
  if (taskDataForClaim.exists) {
    const statusNames = ['Ready', 'Pending', 'Processing'];
    const currentStatus = taskDataForClaim.data.status;
    console.log(`Current task status: ${statusNames[currentStatus] || currentStatus}`);
    
    if (currentStatus === TaskStatus.Pending) {
      console.log('Claiming task...');
      const claimSignature = await dacManager.claimTask({
        payer: signer,
        computeNode: signer,
        agent: agentAddress,
      });
      console.log(`Task claimed! Transaction signature: ${claimSignature}`);
    } else if (currentStatus === TaskStatus.Processing) {
      console.log('Task already claimed and is being processed');
    } else {
      console.log(`⚠️  Task status is ${statusNames[currentStatus]}, cannot claim. Expected status: Pending (1)`);
    }
  }

  // Step 6: Submit Task Result
  console.log('\n=== Step 6: Submitting Task Result ===');
  const taskDataForResult = await dacManager.getTaskData(agentAddress);
  const resultData = new Uint8Array([10, 20, 30, 40, 50]);
  
  if (taskDataForResult.exists) {
    const currentStatus = taskDataForResult.data.status;
    console.log(`Current task status: ${TaskStatus[currentStatus]}`);
    
    if (currentStatus === TaskStatus.Processing) { // Processing = 2
        console.log('Submitting task result...');
        try {
          const resultSignature = await dacManager.submitTaskResult({
            payer: signer,
            computeNode: signer,
            agent: agentAddress,
            result: resultData,
          });
          console.log(`Task result submitted! Transaction signature: ${resultSignature}`);
        } catch (err: any) {
          if (err?.cause?.context?.code === 101) {
            console.error('Error: Instruction not found on chain.');
            console.error('The program needs to be rebuilt and redeployed.');
            console.error('Run: anchor build && anchor deploy --program-name dac-manager');
          } else {
            throw err;
          }
        }
      }
    } else {
      console.log(`Task status must be "Processing" to submit result`);
    }
  
  // Print final result info
  const finalTaskData = await dacManager.getTaskData(agentAddress);
  if (finalTaskData.exists) {
    console.log('\n--- Final Task Data Info ---');
    const statusNames = ['Ready', 'Pending', 'Processing'];
    console.log(`Task Status: ${statusNames[finalTaskData.data.status] || finalTaskData.data.status}`);
    console.log(`Task Data: ${Array.from(finalTaskData.data.data).join(', ')}`);
    console.log(`Result Data: ${finalTaskData.data.result.length > 0 ? Array.from(finalTaskData.data.result).join(', ') : 'None'}`);
    console.log(`Compute Node: ${finalTaskData.data.computeNode}`);
  }
} catch (err) {
  console.error('Error:', err);
}


/*
 // Step 5: Claim Task (by compute node)
  console.log('\n=== Step 5: Claiming Task ===');
  const taskDataForClaim = await dacManager.getTaskData(agentAddress);
  
  if (taskDataForClaim.exists) {
    const statusNames = ['Ready', 'Pending', 'Processing'];
    const currentStatus = taskDataForClaim.data.status;
    console.log(`Current task status: ${statusNames[currentStatus] || currentStatus}`);
    
    if (currentStatus === TaskStatus.Pending) {
      console.log('Claiming task...');
      const claimSignature = await dacManager.claimTask({
        payer: signer,
        computeNode: signer,
        agent: agentAddress,
      });
      console.log(`Task claimed! Transaction signature: ${claimSignature}`);
    } else if (currentStatus === TaskStatus.Processing) {
      console.log('Task already claimed and is being processed');
    } else {
      console.log(`⚠️  Task status is ${statusNames[currentStatus]}, cannot claim. Expected status: Pending (1)`);
    }
  }

  // Step 6: Submit Task Result
  console.log('\n=== Step 6: Submitting Task Result ===');
  const taskDataForResult = await dacManager.getTaskData(agentAddress);
  const resultData = new Uint8Array([10, 20, 30, 40, 50]);
  
  if (taskDataForResult.exists) {
    const currentStatus = taskDataForResult.data.status;
    console.log(`Current task status: ${TaskStatus[currentStatus]}`);
    
    if (currentStatus === TaskStatus.Processing) { // Processing = 2
        console.log('Submitting task result...');
        try {
          const resultSignature = await dacManager.submitTaskResult({
            payer: signer,
            computeNode: signer,
            agent: agentAddress,
            result: resultData,
          });
          console.log(`Task result submitted! Transaction signature: ${resultSignature}`);
        } catch (err: any) {
          if (err?.cause?.context?.code === 101) {
            console.error('Error: Instruction not found on chain.');
            console.error('The program needs to be rebuilt and redeployed.');
            console.error('Run: anchor build && anchor deploy --program-name dac-manager');
          } else {
            throw err;
          }
        }
      }
    } else {
      console.log(`Task status must be "Processing" to submit result`);
    }
  
  // Print final result info
  const finalTaskData = await dacManager.getTaskData(agentAddress);
  if (finalTaskData.exists) {
    console.log('\n--- Final Task Data Info ---');
    const statusNames = ['Ready', 'Pending', 'Processing'];
    console.log(`Task Status: ${statusNames[finalTaskData.data.status] || finalTaskData.data.status}`);
    console.log(`Task Data: ${Array.from(finalTaskData.data.data).join(', ')}`);
    console.log(`Result Data: ${finalTaskData.data.result.length > 0 ? Array.from(finalTaskData.data.result).join(', ') : 'None'}`);
    console.log(`Compute Node: ${finalTaskData.data.computeNode}`);
  }

      // Subscribe to program logs to detect register_compute_node instructions
    let ws_url = config.rpc_websocket_url.clone();
    tokio::spawn(async move {
        let ws_client = PubsubClient::new(&ws_url).await.unwrap();
        let (mut stream, _) = ws_client
            .logs_subscribe(
                solana_client::rpc_config::RpcTransactionLogsFilter::Mentions(vec![
                    DAC_MANAGER_ID.to_string()
                ]),
                solana_client::rpc_config::RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::confirmed()),
                },
            )
            .await
            .expect("Failed to subscribe to program logs");

        println!("Monitoring DAC Manager program logs for register_compute_node events...");

        while let Some(log_notification) = stream.next().await {
            for log in &log_notification.value.logs {
                if log.contains("register_compute_node") || log.contains("RegisterComputeNode") {
                    println!("Register compute node event detected!");
                    println!(
                        "Transaction signature: {}",
                        log_notification.value.signature
                    );
                    println!("Log: {}", log);
                }
            }
        }
    });
*/