//! Utilities for working with syscalls.

mod linux_syscall;
pub use linux_syscall::LinuxSyscall;

use core::arch::asm;

/// Run the `write` Linux syscall.
///
/// # Safety
/// Directly executes assembly code.
#[inline(always)]
pub unsafe fn write(fd: i64, buf: *const u8, count: u64) -> i64 {
    let mut rax = LinuxSyscall::write as i64;

    asm!(
        "syscall",
        inout("rax") rax,
        in("rdi") fd,
        in("rsi") buf,
        in("rdx") count,
        lateout("rcx") _,
        lateout("r11") _,
        options(nostack),
    );

    rax
}

/// Run the `memfd_create` Linux syscall.
///
/// # Safety
/// Directly executes assembly code.
#[inline(always)]
pub unsafe fn memfd_create(name: *const u8, flags: u64) -> i64 {
    let mut rax = LinuxSyscall::memfd_create as i64;

    asm!(
        "syscall",
        inout("rax") rax,
        in("rdi") name,
        in("rsi") flags,
        lateout("rcx") _, lateout("r11") _,
        options(nostack),
    );
    rax
}

/// Run the `execve_at` Linux syscall.
///
/// # Safety
/// Directly executes assembly code.
#[inline(always)]
pub unsafe fn execveat(
    fd: i64,
    pathname: *const u8,
    argv: *const *const u8,
    envp: *const *const u8,
    flags: u64,
) -> ! {
    let rax = LinuxSyscall::execveat as i64;

    asm!(
        "syscall",
        in("rax") rax,
        in("rdi") fd,
        in("rsi") pathname,
        in("rdx") argv,
        in("r10") envp,
        in("r8") flags,
        options(noreturn),
    );
}
