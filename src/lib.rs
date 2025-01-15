mod buffer_pool_manager;
mod clock_replacer;
pub mod disk_manager;
pub mod disk_scheduler;
mod page;

const DEFAULT_PAGE_SIZE: usize = 4096;