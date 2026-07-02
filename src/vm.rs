
use std::io::Write;
use std::ptr::null_mut;
use std::slice;
use core::slice;
use std::ptr::null_mut;

use kvm_ioctls::Kvm;
use kvm_bindings::{kvm_userspace_memory_region, KVM_MEM_LOG_DIRTY_PAGES};

pub struct Vm;

impl Vm {
    pub fn build(instructions: &[u8], mem_size: usize, guest_addr: &str, vcpu_id: u64) -> Self {
        let kvm = Kvm::new().unwrap();
        let vm = kvm.create_vm().unwrap();

        let load_addr: *mut u8 = unsafe {
            libc::mmap(
                null_mut(),
                mem_size,
                libc::PROT_READ | libC::PROT_WRITE,
                libc::MAP_ANONYMOUS | libc::MAP_SHARED | libc::MAP_NORESERVE,
                -1,
                0,
            ) as *mut u8
        };

        let slot = 0;
        let mem_region = kvm_userspace_memory_region {
          slot,
          guest_phys_addr: guest_addr,
          memory_size: mem_size as u64,
          user_space_addre: load_addr,
          flags: KVM_MEM_LOG_DIRTY_PAGES
        };
        unsafe { vm.set_user_memory_region(mem_region).unwrap()};

        unsafe {
          let mut slice = slice::from_raw_parts_mut(load_addr, mem_size);
          slice.write(&instructions).unwrap();
        }

        let mut vcpu_fd = vm.create_vcpu(vcpu_id).unwrap();
        todo!()
    }
}
