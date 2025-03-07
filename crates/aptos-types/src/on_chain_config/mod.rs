// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::event::{EventHandle, EventKey};

use move_core_types::{
    ident_str,
    identifier::{IdentStr, Identifier},
    language_storage::{StructTag, TypeTag, CORE_CODE_ADDRESS},
    move_resource::{MoveResource, MoveStructType},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, fmt::Debug, str::FromStr};

// mod approved_execution_hashes;
// mod aptos_features;
// mod aptos_version;
// mod chain_id;
// mod commit_history;
// mod consensus_config;
// mod execution_config;
// mod gas_schedule;
// mod jwk_consensus_config;
// pub mod randomness_api_v0_config;
// mod randomness_config;
// mod timed_features;
// mod timestamp;
// mod transaction_fee;
// mod validator_set;

/// To register an on-chain config in Rust:
/// 1. Implement the `OnChainConfig` trait for the Rust representation of the config
/// 2. Add the config's `ConfigID` to `ON_CHAIN_CONFIG_REGISTRY`

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ConfigID(&'static str, &'static str, &'static str);

impl ConfigID {
    pub fn name(&self) -> String {
        self.2.to_string()
    }
}

impl fmt::Display for ConfigID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OnChain config ID [address: {}, identifier: {}]",
            self.0, self.1
        )
    }
}

// pub trait OnChainConfigProvider: Debug + Clone + Send + Sync + 'static {
//     fn get<T: OnChainConfig>(&self) -> Result<T>;
// }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InMemoryOnChainConfig {
    configs: HashMap<ConfigID, Vec<u8>>,
}

impl InMemoryOnChainConfig {
    pub fn new(configs: HashMap<ConfigID, Vec<u8>>) -> Self {
        Self { configs }
    }
}

pub fn new_epoch_event_key() -> EventKey {
    EventKey::new(2, CORE_CODE_ADDRESS)
}

pub fn new_epoch_event_type_tag() -> TypeTag {
    TypeTag::from_str("0x1::reconfiguration::NewEpoch").expect("cannot fail")
}

pub fn struct_tag_for_config(config_id: ConfigID) -> StructTag {
    StructTag {
        address: CORE_CODE_ADDRESS,
        module: Identifier::new(config_id.1).expect("fail to make identifier"),
        name: Identifier::new(config_id.2).expect("fail to make identifier"),
        type_args: vec![],
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigurationResource {
    epoch: u64,
    /// Unix epoch timestamp (in microseconds) of the last reconfiguration time.
    last_reconfiguration_time: u64,
    events: EventHandle,
}

impl ConfigurationResource {
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Return the last Unix epoch timestamp (in microseconds) of the last reconfiguration time.
    pub fn last_reconfiguration_time_micros(&self) -> u64 {
        self.last_reconfiguration_time
    }

    pub fn events(&self) -> &EventHandle {
        &self.events
    }
}

impl MoveStructType for ConfigurationResource {
    const MODULE_NAME: &'static IdentStr = ident_str!("reconfiguration");
    const STRUCT_NAME: &'static IdentStr = ident_str!("Configuration");
}

impl MoveResource for ConfigurationResource {}

// impl OnChainConfig for ConfigurationResource {
//     const MODULE_IDENTIFIER: &'static str = "reconfiguration";
//     const TYPE_IDENTIFIER: &'static str = "Configuration";
// }
