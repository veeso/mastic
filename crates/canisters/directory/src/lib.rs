mod memory;

use std::cell::RefCell;

use ic_cdk_macros::{post_upgrade, pre_upgrade, query, update};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::{DefaultMemoryImpl, StableCell};
use memory::{MEMORY_MANAGER, STATE_MEMORY_ID};

#[pre_upgrade]
fn pre_upgrade() {}

#[post_upgrade]
fn post_upgrade() {}

ic_cdk::export_candid!();
