//! Memory management for the federation canister.

mod principal;

use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as IcMemoryManager};

pub use self::principal::StorablePrincipal;

// Settings memory ids
pub const DIRECTORY_CANISTER_MEMORY_ID: MemoryId = MemoryId::new(0);
pub const PUBLIC_URL_MEMORY_ID: MemoryId = MemoryId::new(1);

thread_local! {
    /// Memory manager
    pub static MEMORY_MANAGER: IcMemoryManager<DefaultMemoryImpl> = IcMemoryManager::init(DefaultMemoryImpl::default());
}
