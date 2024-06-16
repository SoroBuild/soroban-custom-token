#![no_std]

extern crate soroban_sdk as sdk;

use sdk::{
    contract, contractimpl, contractmeta, Address, BytesN, ConversionError, Env, IntoVal,
    TryFromVal, Val, Symbol, Vec,
};
use core::ops::Div;

#[derive(Default, Debug)]
pub struct UserInfo {
    pub deposit: i128,
    pub reward_debt: i128,
}

#[derive(Default, Debug)]
pub struct ContractData {
    pub liquidity_token: Option<BytesN<32>>,
    pub reward_token: Option<BytesN<32>>,
    pub reward_rate: i128,
    pub total_deposit: i128,
    pub total_rewards: i128,
    pub end_time: i64,
    pub last_update_time: i64,
    pub user_info: Vec<(Address, UserInfo)>,
}

#[contract]
pub struct LiquidityPoolContract;

#[contractimpl]
impl LiquidityPoolContract {
    pub fn initialize(
        env: Env,
        liquidity_token: Address,
        reward_token: Address,
    ) {
        let admin = env.invoker();
        env.storage().set(
            &Symbol::from("data"),
            &ContractData {
                liquidity_token: Some(liquidity_token),
                reward_token: Some(reward_token),
                reward_rate: 0,
                total_deposit: 0,
                total_rewards: 0,
                end_time: 0,
                last_update_time: env.block().timestamp(),
                user_info: Vec::new(),
            },
        );
    }

    pub fn deposit(env: Env, amount: i128) {
        let invoker = env.invoker();
        let mut data = Self::get_contract_data(&env);

        let liquidity_token = sdk::token::Token::new(env.clone(), data.liquidity_token.clone().unwrap());
        liquidity_token.transfer_from(invoker.clone(), env.current_contract_address(), amount);

        let mut user_info = data.user_info.iter_mut().find(|(addr, _)| addr == &invoker);
        if let Some(user) = user_info {
            Self::update_rewards(&mut data, env.block().timestamp());
            user.1.deposit += amount;
            user.1.reward_debt = user.1.deposit * data.reward_rate / data.total_deposit;
        } else {
            data.user_info.push((invoker.clone(), UserInfo { deposit: amount, reward_debt: 0 }));
        }

        data.total_deposit += amount;

        Self::set_contract_data(&env, data);
    }

    pub fn withdraw(env: Env, amount: i128) {
        let invoker = env.invoker();
        let mut data = Self::get_contract_data(&env);

        let user_info = data.user_info.iter_mut().find(|(addr, _)| addr == &invoker).unwrap();
        if user_info.1.deposit < amount {
            env.panic(sdk::Error::from(ContractError::InsufficientBalance));
        }

        Self::update_rewards(&mut data, env.block().timestamp());

        user_info.1.deposit -= amount;
        user_info.1.reward_debt = user_info.1.deposit * data.reward_rate / data.total_deposit;

        data.total_deposit -= amount;

        let liquidity_token = sdk::token::Token::new(env.clone(), data.liquidity_token.clone().unwrap());
        liquidity_token.transfer(invoker.clone(), amount);

        Self::set_contract_data(&env, data);
    }

    pub fn claim_reward(env: Env) {
        let invoker = env.invoker();
        let mut data = Self::get_contract_data(&env);

        Self::update_rewards(&mut data, env.block().timestamp());

        let user_info = data.user_info.iter_mut().find(|(addr, _)| addr == &invoker).unwrap();
        let pending_reward = user_info.1.deposit * data.reward_rate / data.total_deposit - user_info.1.reward_debt;

        if pending_reward > 0 {
            let reward_token = sdk::token::Token::new(env.clone(), data.reward_token.clone().unwrap());
            reward_token.transfer(invoker.clone(), pending_reward);
        }

        user_info.1.reward_debt = user_info.1.deposit * data.reward_rate / data.total_deposit;

        Self::set_contract_data(&env, data);
    }

    pub fn admin_set_rewards(env: Env, reward_amount: i128, duration: i64) {
        let mut data = Self::get_contract_data(&env);

        Self::update_rewards(&mut data, env.block().timestamp());

        let new_rate = reward_amount / duration;
        data.reward_rate = new_rate;
        data.total_rewards += reward_amount;
        data.end_time = env.block().timestamp() + duration;

        let reward_token = sdk::token::Token::new(env.clone(), data.reward_token.clone().unwrap());
        reward_token.transfer_from(env.invoker(), env.current_contract_address(), reward_amount);

        Self::set_contract_data(&env, data);
    }

    pub fn admin_adjust_rewards(env: Env, additional_reward: i128, additional_duration: i64) {
        let mut data = Self::get_contract_data(&env);

        Self::update_rewards(&mut data, env.block().timestamp());

        let new_total_rewards = data.total_rewards + additional_reward;
        let new_end_time = data.end_time + additional_duration;
        let new_rate = new_total_rewards / (new_end_time - env.block().timestamp());

        data.reward_rate = new_rate;
        data.total_rewards = new_total_rewards;
        data.end_time = new_end_time;

        let reward_token = sdk::token::Token::new(env.clone(), data.reward_token.clone().unwrap());
        reward_token.transfer_from(env.invoker(), env.current_contract_address(), additional_reward);

        Self::set_contract_data(&env, data);
    }

    fn get_contract_data(env: &Env) -> ContractData {
        env.storage().get::<Symbol, ContractData>(&Symbol::from("data")).unwrap_or_default()
    }

    fn set_contract_data(env: &Env, data: ContractData) {
        env.storage().set(&Symbol::from("data"), &data);
    }

    fn update_rewards(data: &mut ContractData, current_time: i64) {
        if current_time > data.end_time {
            data.reward_rate = 0;
        }
        data.last_update_time = current_time;
    }
}

