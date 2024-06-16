#![no_std]
mod library{
    pub mod features {
        pub mod  mintable;
    }
    pub mod access {
        pub mod admin;
    }
}
mod allowance;
mod balance;
mod contract;
mod metadata;
mod storage_types;
mod test;

pub use crate::contract::TokenClient;
