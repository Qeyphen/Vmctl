use std::io::Write;
use std::ptr::null_mut;
use std::slice;

use kvm_bindings::KVM_MEM_LOG_DIRTY_PAGES;
use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::VcpuExit;
use kvm_ioctls::{Kvm, VcpuFd, VmFd};

pub struct Vm {
    kvm: Kvm,
    vm: VmFd,
    vcpu_fd: VcpuFd,
}

impl Vm {
    pub fn build(instructions: &[u8], mem_size: usize, guest_addr: u64, vcpu_id: u64) -> Self {
        let kvm = Kvm::new().unwrap();
        let vm = kvm.create_vm().unwrap();

        let load_addr: *mut u8 = unsafe {
            libc::mmap(
                null_mut(),
                mem_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_ANONYMOUS | libc::MAP_SHARED | libc::MAP_NORESERVE,
                -1,
                0,
            ) as *mut u8
        };

        if load_addr == libc::MAP_FAILED as *mut u8 {
            panic!("mmap failed")
        }

        let slot = 0;
        let mem_region = kvm_userspace_memory_region {
            slot,
            guest_phys_addr: guest_addr,
            memory_size: mem_size as u64,
            userspace_addr: load_addr as u64,
            flags: KVM_MEM_LOG_DIRTY_PAGES,
        };
        unsafe { vm.set_user_memory_region(mem_region).unwrap() };

        unsafe {
            let mut slice = slice::from_raw_parts_mut(load_addr, mem_size);
            slice.write(&instructions).unwrap();
        }

        let mut vcpu_fd = vm.create_vcpu(vcpu_id).unwrap();

        #[cfg(target_arch = "x86_64")]
        {
            let mut vcpu_sregs = vcpu_fd.get_sregs().unwrap();
            vcpu_sregs.cs.base = 0;
            vcpu_sregs.cs.selector = 0;
            vcpu_fd.set_sregs(&vcpu_sregs).unwrap();

            let mut vcpu_regs = vcpu_fd.get_regs().unwrap();
            vcpu_regs.rip = guest_addr;
            vcpu_regs.rflags = 2;
            vcpu_fd.set_regs(&vcpu_regs).unwrap();
        }

        Self { kvm, vm, vcpu_fd }
    }
}
