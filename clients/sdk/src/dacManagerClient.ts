import {
  type Address,
  getProgramDerivedAddress,
  getAddressEncoder,
  getU64Encoder,
  getBytesEncoder,
  type TransactionSigner,
  type SolanaClient,
} from 'gill';
import { sendTransaction } from './utils.js';
import { getCreateAgentInstructionAsync } from './generated/dac-manager/instructions/createAgent.js';
import { getSubmitTaskInstructionAsync } from './generated/dac-manager/instructions/submitTask.js';
import { getClaimTaskInstructionAsync } from './generated/dac-manager/instructions/claimTask.js';
import { getSubmitTaskResultInstructionAsync } from './generated/dac-manager/instructions/submitTaskResult.js';
import { getRegisterComputeNodeInstructionAsync } from './generated/dac-manager/instructions/registerComputeNode.js';
import { CLAIM_NODE_DISCRIMINATOR } from './generated/dac-manager/instructions/claimNode.js';
import { fetchAgent, fetchMaybeAgent } from './generated/dac-manager/accounts/agent.js';
import { fetchMaybeTaskData } from './generated/dac-manager/accounts/taskData.js';
import { fetchMaybeComputeNode } from './generated/dac-manager/accounts/computeNode.js';
import { DAC_MANAGER_PROGRAM_ADDRESS } from './generated/dac-manager/programs/dacManager.js';

export class DacManagerClient {
  constructor(
    private readonly client: SolanaClient,
    private readonly programAddress: Address = DAC_MANAGER_PROGRAM_ADDRESS
  ) {}

  async deriveAgentAddress(owner: Address, agentId: bigint): Promise<Address> {
    const [agentAddress] = await getProgramDerivedAddress({
      programAddress: this.programAddress,
      seeds: [
        Buffer.from('agent'),
        getAddressEncoder().encode(owner),
        Buffer.from(getU64Encoder().encode(agentId)),
      ],
    });
    return agentAddress;
  }

  async getAgent(owner: Address, agentId: bigint) {
    const agentAddress = await this.deriveAgentAddress(owner, agentId);
    return await fetchMaybeAgent(this.client.rpc, agentAddress);
  }

  async getAgentByAddress(agentAddress: Address) {
    return await fetchAgent(this.client.rpc, agentAddress);
  }

  async deriveTaskDataAddress(agentAddress: Address): Promise<Address> {
    const [taskDataAddress] = await getProgramDerivedAddress({
      programAddress: this.programAddress,
      seeds: [
        Buffer.from('task_data'),
        getAddressEncoder().encode(agentAddress),
      ],
    });
    return taskDataAddress;
  }

  async getTaskData(agentAddress: Address) {
    const taskDataAddress = await this.deriveTaskDataAddress(agentAddress);
    return await fetchMaybeTaskData(this.client.rpc, taskDataAddress);
  }

  async deriveComputeNodeAddress(owner: Address, nodePubkey: Address): Promise<Address> {
    const [computeNodeAddress] = await getProgramDerivedAddress({
      programAddress: this.programAddress,
      seeds: [
        Buffer.from('compute_node'),
        getAddressEncoder().encode(owner),
        getAddressEncoder().encode(nodePubkey),
      ],
    });
    return computeNodeAddress;
  }

  async getComputeNode(owner: Address, nodePubkey: Address) {
    const computeNodeAddress = await this.deriveComputeNodeAddress(owner, nodePubkey);
    return await fetchMaybeComputeNode(this.client.rpc, computeNodeAddress);
  }

  async getComputeNodeByAddress(computeNodeAddress: Address) {
    return await fetchMaybeComputeNode(this.client.rpc, computeNodeAddress);
  }

  async registerComputeNode(params: {
    payer: TransactionSigner;
    owner: TransactionSigner;
    nodePubkey: Address;
    nodeInfoCid: string;
  }): Promise<{ signature: string; computeNodeAddress: Address }> {
    const instruction = await getRegisterComputeNodeInstructionAsync({
      payer: params.payer as any,
      owner: params.owner as any,
      nodePubkey: params.nodePubkey,
      nodeInfoCid: params.nodeInfoCid,
    });

    const signature = await sendTransaction(this.client, params.payer, [instruction]);
    const computeNodeAddress = await this.deriveComputeNodeAddress(
      params.owner.address,
      params.nodePubkey
    );

    return { signature, computeNodeAddress };
  }

  async createAgent(params: {
    payer: TransactionSigner;
    owner: TransactionSigner;
    agentId: bigint;
    computeNodeOwner: Address;
    computeNodePubkey: Address;
    public: boolean;
  }): Promise<{ signature: string; agentAddress: Address }> {
    const computeNodeAddress = await this.deriveComputeNodeAddress(
      params.computeNodeOwner,
      params.computeNodePubkey
    );

    const instruction = await getCreateAgentInstructionAsync({
      payer: params.payer as any,
      owner: params.owner as any,
      agentId: params.agentId,
      computeNode: computeNodeAddress,
      public: params.public,
    });

    const signature = await sendTransaction(this.client, params.payer, [instruction]);
    const agentAddress = await this.deriveAgentAddress(
      params.owner.address,
      params.agentId
    );

    return { signature, agentAddress };
  }

  subscribeToClaimNodeEvents(
    computeNodeAddress: Address,
    callback: (data: { signature: string; slot: bigint }) => void
  ) {
    if (!this.client.rpcSubscriptions) {
      throw new Error('RPC subscriptions not available. Use createSolanaClient with subscriptions enabled.');
    }

    const subscriptionRequest = this.client.rpcSubscriptions.accountNotifications(computeNodeAddress, {
      commitment: 'confirmed',
      encoding: 'base64',
    });

    // Handle the subscription asynchronously
    (async () => {
      try {
    const subscription = await subscriptionRequest;
    // @ts-ignore - PendingRpcSubscriptionsRequest structure may vary
    const stream = await subscription.value();
        
        for await (const notification of stream) {
          const logs = notification.value.logs || [];
          for (const log of logs) {
            // Check if log contains claim node instruction discriminator
            // The discriminator bytes might appear in the logs
            const discriminatorHex = Buffer.from(CLAIM_NODE_DISCRIMINATOR).toString('hex');
            if (log.includes(discriminatorHex) || log.includes('claim_node') || log.includes('ClaimNode')) {
              callback({
                signature: notification.value.signature,
                slot: notification.context.slot,
              });
              break;
            }
          }
        }
      } catch (error: unknown) {
        console.error('Error in claim node subscription:', error);
      }
    })();

    return subscriptionRequest;
  }

  async submitTask(params: {
    payer: TransactionSigner;
    user: TransactionSigner;
    agent: Address;
    data: Uint8Array;
  }): Promise<string> {
    const instruction = await getSubmitTaskInstructionAsync({
      payer: params.payer as any,
      user: params.user as any,
      agent: params.agent,
      data: params.data,
    });

    return await sendTransaction(this.client, params.payer, [instruction]);
  }

  async claimTask(params: {
    payer: TransactionSigner;
    computeNode: TransactionSigner;
    agent: Address;
  }): Promise<string> {
    const instruction = await getClaimTaskInstructionAsync({
      payer: params.payer as any,
      computeNode: params.computeNode as any,
      agent: params.agent,
    });

    return await sendTransaction(this.client, params.payer, [instruction]);
  }

  async submitTaskResult(params: {
    payer: TransactionSigner;
    computeNode: TransactionSigner;
    agent: Address;
    result: Uint8Array;
  }): Promise<string> {
    const instruction = await getSubmitTaskResultInstructionAsync({
      payer: params.payer as any,
      computeNode: params.computeNode as any,
      agent: params.agent,
      result: params.result,
    });

    return await sendTransaction(this.client, params.payer, [instruction]);
  }
}
