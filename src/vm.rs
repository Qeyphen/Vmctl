use kvm_bindings::KVM_MEM_LOG_DIRTY_PAGES;
use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::{Kvm, VcpuFd, VmFd};
use std::io::Write;
use std::ptr::null_mut;
use std::slice;

use crate::error::VmError;

#[derive(Debug)]
pub struct Vm {
    kvm: Kvm,
    vm: VmFd,
    vcpu_fd: VcpuFd,
}

impl Vm {
    pub fn build(
        instructions: &[u8],
        mem_size: usize,
        guest_addr: u64,
        vcpu_id: u64,
    ) -> Result<Self, VmError> {
        let kvm = Kvm::new().map_err(|e| VmError::CreateKvm(Box::new(e)))?;

        let vm = kvm
            .create_vm()
            .map_err(|e| VmError::CreateVm(Box::new(e)))?;

        let load_addr = unsafe {
            libc::mmap(
                null_mut(),
                mem_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_ANONYMOUS | libc::MAP_SHARED | libc::MAP_NORESERVE,
                -1,
                0,
            )
        };

        if load_addr == libc::MAP_FAILED {
            return Err(VmError::MmapFailed {
                size: mem_size,
                errno: unsafe { *libc::__errno_location() },
            });
        }

        let load_addr = load_addr as *mut u8;

        let mem_region = kvm_userspace_memory_region {
            slot: 0,
            guest_phys_addr: guest_addr,
            memory_size: mem_size as u64,
            userspace_addr: load_addr as u64,
            flags: KVM_MEM_LOG_DIRTY_PAGES,
        };

        unsafe {
            vm.set_user_memory_region(mem_region).map_err(|_| {
                VmError::SetUserMemoryRegionFailed {
                    errno: *libc::__errno_location(),
                }
            })?;
        }

        unsafe {
            let mut slice = slice::from_raw_parts_mut(load_addr, mem_size);
            slice.write_all(instructions).map_err(VmError::Io)?;
        }

        let mut vcpu_fd = vm
            .create_vcpu(vcpu_id)
            .map_err(|e| VmError::CreateVm(Box::new(e)))?;

        #[cfg(target_arch = "x86_64")]
        {
            let mut sregs = vcpu_fd.get_sregs().unwrap();
            sregs.cs.base = 0;
            sregs.cs.selector = 0;
            vcpu_fd.set_sregs(&sregs).unwrap();

            let mut regs = vcpu_fd.get_regs().unwrap();
            regs.rip = guest_addr;
            regs.rflags = 2;
            vcpu_fd.set_regs(&regs).unwrap();
        }

        Ok(Self { kvm, vm, vcpu_fd })
    }
}
