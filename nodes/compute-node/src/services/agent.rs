use anyhow::Result;
use tokio::sync::{Mutex, mpsc};
use crate::adapters::SolanaAdapter;
use solana_sdk::pubkey::Pubkey;
use sol_mind_protocol_client::dac_manager::{accounts::Agent, types::AgentStatus};
use std::{collections::HashMap, sync::Arc};

pub struct AgentService {
    adapter: Arc<SolanaAdapter>,
    agents_registry: Arc<Mutex<HashMap<Pubkey, Agent>>>,
}

impl AgentService {
    pub fn new(adapter: Arc<SolanaAdapter>) -> Self {
        Self { adapter, agents_registry: Arc::new(Mutex::new(HashMap::new()))}
    }

    pub async fn monitor_and_register_agents(
        &mut self,
        node_pubkey: &Pubkey,
    ) -> Result<()> {
        self.start_watching_agent_accounts(node_pubkey, Arc::clone(&self.agents_registry));
        
        println!("Checking and registering pending agents for node: {:?}", node_pubkey);
        if let Some(agents) = self.adapter.get_agents_accounts(node_pubkey).await? {
            for agent in agents {
                if agent.1.status == AgentStatus::Pending {
                    println!("Activating pending agent: {:?}", agent.0);
                    if let Err(e) = self.adapter.activate_agent(&agent.1).await {
                        eprintln!("Failed to activate agent {:?}: {}", agent.0, e);
                    } else {
                        self.add_agent_by_address(&agent.0, agent.1.clone()).await?;
                        println!("Successfully activated agent and added to registry: {:?}", agent.0);
                    }
                }
                else if agent.1.status == AgentStatus::Active { //TODO: this also will be tracked on ifps!
                    self.add_agent_by_address(&agent.0, agent.1.clone()).await?;
                    println!("Successfully added agent to registry: {:?}", agent.0);
                }
            }
            return Ok(());
        }

        return Ok(());
    }

    async fn add_agent_by_address(
        &mut self,
        agent_address: &Pubkey,
        agent: Agent,
    ) -> Result<()> {
        self.agents_registry.lock().await.insert(*agent_address, agent);
        Ok(())
    }

    async fn add_agent(
        &mut self,
        agent: Agent,
    ) -> Result<()> {
        let agent_address = self.adapter.derive_agent_pda(&agent.owner, agent.agent_id)?;
        self.agents_registry.lock().await.insert(agent_address, agent);

        Ok(())
    }

    async fn remove_agent(
        &mut self,
        agent: &Agent,
    ) -> Result<()> {
        let agent_address = self.adapter.derive_agent_pda(&agent.owner, agent.agent_id)?;
        self.agents_registry.lock().await.remove(&agent_address);

        Ok(())
    }

    pub async fn get_agent(
        &self,
        agent_address: &Pubkey,
    ) -> Option<Agent> {
        self.agents_registry.lock().await.get(agent_address).cloned() // dont like this,
    }

    pub fn start_watching_agent_accounts(
        &self, node_pubkey: &Pubkey, agents_registry: Arc<Mutex<HashMap<Pubkey, Agent>>>
    ) -> tokio::task::JoinHandle<Result<()>> {
        let adapter = Arc::clone(&self.adapter);
        let node_pubkey = *node_pubkey;

        tokio::spawn(async move {
            let (tx, mut rx) = mpsc::channel::<Agent>(16);
            
            let watch_handle = adapter.watch_agent_accounts(&node_pubkey, tx);

            println!("Watching agent accounts...");
            loop {
                tokio::select! {
                    result = rx.recv() => {
                        match result {
                            Some(agent) => {
                                println!("Agent account detected: {:?}", agent);
                                let agent_address = adapter.derive_agent_pda(&agent.owner, agent.agent_id).unwrap(); //TODO: pass the agent and the address
                                if agent.status == AgentStatus::Pending {
                                    println!("Activating pending agent: {:?}", agent_address);
                                    if let Err(e) = adapter.activate_agent(&agent).await {
                                        eprintln!("Failed to activate agent {:?}: {}", agent_address, e);
                                    } else {
                                        agents_registry.lock().await.insert(agent_address, agent);
                                        println!("Successfully activated agent: {:?}", agent_address);
                                    }
                                }
                            }
                            None => {
                                println!("Watch channel closed");
                                break;
                            }
                        }
                    }
                    _ = tokio::signal::ctrl_c() => {
                        println!("Closing watch agent accounts...");
                        watch_handle.abort();
                        break;
                    }
                }
            }

            Ok(())
        })
    }
}
