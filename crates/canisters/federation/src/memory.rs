//! Memory management for the federation canister.

mod principal;
mod user_data;

use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as IcMemoryManager};

pub use self::principal::StorablePrincipal;
pub use self::user_data::UserData;

// Settings memory ids
pub const DIRECTORY_CANISTER_MEMORY_ID: MemoryId = MemoryId::new(0);
pub const PUBLIC_URL_MEMORY_ID: MemoryId = MemoryId::new(1);

// users
pub const USER_DATA_BY_ID_MEMORY_ID: MemoryId = MemoryId::new(10);
pub const USER_DATA_BY_HANDLE_MEMORY_ID: MemoryId = MemoryId::new(11);
pub const USER_DATA_MEMORY_ID: MemoryId = MemoryId::new(12);
pub const USER_DATA_BY_CANISTER_ID_MEMORY_ID: MemoryId = MemoryId::new(13);

thread_local! {
    /// Memory manager
    pub static MEMORY_MANAGER: IcMemoryManager<DefaultMemoryImpl> = IcMemoryManager::init(DefaultMemoryImpl::default());
}
