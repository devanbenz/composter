pub mod buffer_pool_manager;
pub(crate) mod clock_replacer;
pub(crate) mod disk_manager;
pub(crate) mod disk_scheduler;

const DEFAULT_PAGE_SIZE: usize = 4096;
