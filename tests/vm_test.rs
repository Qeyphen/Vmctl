use vmctl::vm::Vm;

#[test]
fn test_vm_build() {
    if std::fs::metadata("/dev/kvm").is_err() {
        eprintln!("Skipping test: KVM not available");
        return;
    }

    let instructions = vec![0xB8, 0x04, 0x00, 0x00, 0x00, 0xF4];

    let vm = Vm::build(&instructions, 0x1000, 0x1000, 0);

    assert!(vm.is_ok(), "VM should build successfully");
}
