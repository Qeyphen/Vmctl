use thiserror::Error;

#[derive(Debug, Error)]
pub enum VmError {
    #[error("failed to create KVM instance")]
    CreateKvm(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("failed to create VM instance")]
    CreateVm(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("mmap failed for size {size} bytes (errno: {errno})")]
    MmapFailed { size: usize, errno: i32 },

    #[error("failed to set user memory region")]
    SetUserMemoryRegionFailed { errno: i32 },

    #[error("I/O error: {0}")]
    Io(std::io::Error),
}
