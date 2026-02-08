mod config;
mod ledger;
mod token;
mod compliance;
mod sponsor;
mod utils;

use crate::ledger::account::UserAccount;
use crate::token::rewards::RewardEngine;
use crate::compliance::ethics::EthicalValidator;
use crate::sponsor::grant::GrantDistributor;

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("🔥 Auto_Church node starting...");

    let mut account = UserAccount::new("example_stakeholder");
    account.add_deed("Recycled local waste and helped clean park", 50);

    let ethics = EthicalValidator::new();
    let score = ethics.evaluate(&account);

    if score > 70 {
        let reward_engine = RewardEngine::new();
        let minted = reward_engine.mint_tokens(&account);
        log::info!("Minted {} CHURCH tokens for {}", minted, account.id);
    }

    let grant = GrantDistributor::new();
    grant.allocate_funds(
        "Eco Shelter Program",
        "Providing green housing for the homeless.",
    );

    log::info!("✅ Auto_Church execution completed.");
}
