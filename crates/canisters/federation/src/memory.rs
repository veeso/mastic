use ic_stable_structures::DefaultMemoryImpl;
use ic_stable_structures::memory_manager::MemoryManager as IcMemoryManager;

thread_local! {
    /// Memory manager
    pub static MEMORY_MANAGER: IcMemoryManager<DefaultMemoryImpl> = IcMemoryManager::init(DefaultMemoryImpl::default());
}
