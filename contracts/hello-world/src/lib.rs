#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, Address, symbol_short};

// Structure to store campaign details
#[contracttype]
#[derive(Clone)]
pub struct Campaign {
    pub campaign_id: u64,
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub goal_amount: u64,
    pub raised_amount: u64,
    pub deadline: u64,
    pub is_active: bool,
}

// Mapping campaign ID to Campaign struct
#[contracttype]
pub enum CampaignBook {
    Campaign(u64)
}

// Counter for generating unique campaign IDs
const CAMPAIGN_COUNT: Symbol = symbol_short!("C_COUNT");

#[contract]
pub struct CrowdfundingContract;

#[contractimpl]
impl CrowdfundingContract {
    
    /// Creates a new crowdfunding campaign
    /// Returns the unique campaign ID
    pub fn create_campaign(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        goal_amount: u64,
        duration_days: u64
    ) -> u64 {
        // Verify the creator's authorization
        creator.require_auth();
        
        // Get and increment campaign counter
        let mut campaign_count: u64 = env.storage().instance().get(&CAMPAIGN_COUNT).unwrap_or(0);
        campaign_count += 1;
        
        // Calculate deadline timestamp
        let current_time = env.ledger().timestamp();
        let deadline = current_time + (duration_days * 86400); // 86400 seconds in a day
        
        // Create new campaign
        let campaign = Campaign {
            campaign_id: campaign_count,
            creator: creator.clone(),
            title,
            description,
            goal_amount,
            raised_amount: 0,
            deadline,
            is_active: true,
        };
        
        // Store campaign
        env.storage().instance().set(&CampaignBook::Campaign(campaign_count), &campaign);
        env.storage().instance().set(&CAMPAIGN_COUNT, &campaign_count);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Campaign Created with ID: {}", campaign_count);
        campaign_count
    }
    
    /// Allows users to contribute to a campaign
    pub fn contribute(env: Env, contributor: Address, campaign_id: u64, amount: u64) {
        // Verify contributor's authorization
        contributor.require_auth();
        
        // Get campaign details
        let mut campaign = Self::view_campaign(env.clone(), campaign_id);
        
        // Validate campaign exists and is active
        if campaign.campaign_id == 0 {
            log!(&env, "Campaign not found!");
            panic!("Campaign not found!");
        }
        
        if !campaign.is_active {
            log!(&env, "Campaign is not active!");
            panic!("Campaign is not active!");
        }
        
        // Check if deadline has passed
        let current_time = env.ledger().timestamp();
        if current_time > campaign.deadline {
            log!(&env, "Campaign deadline has passed!");
            panic!("Campaign deadline has passed!");
        }
        
        // Update raised amount
        campaign.raised_amount += amount;
        
        // Store updated campaign
        env.storage().instance().set(&CampaignBook::Campaign(campaign_id), &campaign);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Contribution of {} made to Campaign ID: {}", amount, campaign_id);
    }
    
    /// Closes a campaign (can be called by creator or when deadline is reached)
    pub fn close_campaign(env: Env, campaign_id: u64, caller: Address) {
        caller.require_auth();
        
        let mut campaign = Self::view_campaign(env.clone(), campaign_id);
        
        // Validate campaign exists
        if campaign.campaign_id == 0 {
            log!(&env, "Campaign not found!");
            panic!("Campaign not found!");
        }
        
        // Check if caller is the creator or deadline has passed
        let current_time = env.ledger().timestamp();
        let is_creator = campaign.creator == caller;
        let is_expired = current_time > campaign.deadline;
        
        if !is_creator && !is_expired {
            log!(&env, "Only creator can close campaign before deadline!");
            panic!("Unauthorized to close campaign!");
        }
        
        // Mark campaign as inactive
        campaign.is_active = false;
        
        env.storage().instance().set(&CampaignBook::Campaign(campaign_id), &campaign);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Campaign ID: {} has been closed", campaign_id);
    }
    
    /// View campaign details by campaign ID
    pub fn view_campaign(env: Env, campaign_id: u64) -> Campaign {
        let key = CampaignBook::Campaign(campaign_id);
        
        env.storage().instance().get(&key).unwrap_or(Campaign {
            campaign_id: 0,
            creator: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            title: String::from_str(&env, "Not_Found"),
            description: String::from_str(&env, "Not_Found"),
            goal_amount: 0,
            raised_amount: 0,
            deadline: 0,
            is_active: false,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create_campaign() {
        let env = Env::default();
        let contract_id = env.register_contract(None, CrowdfundingContract);
        let client = CrowdfundingContractClient::new(&env, &contract_id);
        
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Build a dApp");
        let description = String::from_str(&env, "Funding for blockchain project");
        
        let campaign_id = client.create_campaign(&creator, &title, &description, &1000, &30);
        
        assert_eq!(campaign_id, 1);
    }
}
