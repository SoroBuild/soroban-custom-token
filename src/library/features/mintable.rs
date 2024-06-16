use soroban_sdk::{Address, Env};
use soroban_token_sdk::TokenUtils;

use crate::balance::receive_balance;
use crate::storage_types::{INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD};
use crate::library::access::admin::read_administrator;


pub fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

pub fn mint(e: Env, to: Address, amount: i128) {
    check_nonnegative_amount(amount);
    let admin = read_administrator(&e);
    admin.require_auth();

    e.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);

    receive_balance(&e, to.clone(), amount);
    TokenUtils::new(&e).events().mint(admin, to, amount);
}

