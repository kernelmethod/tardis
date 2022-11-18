//! Utilities for working with syscalls.

mod linux_syscall;
pub use linux_syscall::LinuxSyscall;

use alloc::ffi::CString;
use core::arch::asm;
use core::ffi::CStr;
#[cfg(unix)]
use std::os::unix::io::RawFd;

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
pub unsafe fn memfd_create(name: &CStr, flags: u64) -> RawFd {
    let name = name.as_ptr();
    let mut rax = LinuxSyscall::memfd_create as i64;

    asm!(
        "syscall",
        inout("rax") rax,
        in("rdi") name,
        in("rsi") flags,
        lateout("rcx") _, lateout("r11") _,
        options(nostack),
    );

    let fd: RawFd = rax.try_into().unwrap();
    fd
}

/// Run the `execve_at` Linux syscall.
///
/// # Safety
/// Directly executes assembly code.
#[inline(always)]
pub unsafe fn execveat(
    fd: RawFd,
    pathname: &CStr,
    argv: &[CString],
    envp: &[CString],
    flags: u64,
) -> ! {
    let fd: i64 = fd.into();
    let pathname = pathname.as_ptr();
    let argv = argv
        .iter()
        .map(|s| s.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect::<Vec<_>>();
    let envp = envp
        .iter()
        .map(|s| s.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect::<Vec<_>>();

    let rax = LinuxSyscall::execveat as i64;

    asm!(
        "syscall",
        in("rax") rax,
        in("rdi") fd,
        in("rsi") pathname,
        in("rdx") argv.as_ptr(),
        in("r10") envp.as_ptr(),
        in("r8") flags,
        options(noreturn),
    );
}
