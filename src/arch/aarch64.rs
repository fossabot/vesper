/// The entry to Rust, all things must be initialized
/// This is called by assembly trampoline, does arch-specific init
/// and passes control to the kernel boot function kmain().
#[no_mangle]
pub unsafe extern fn karch_start() -> ! {
    // Todo: arch-specific init
    ::kmain()
}
