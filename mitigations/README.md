# Mitigations

The easiest way to defend against the technique used by tardis is to audit or
outright block calls to the `memfd_create` and `execveat` syscalls. This
directory and this README contain various tools and references you can use to
implement these mitigations.

## auditd rules

You can find auditd rules for detecting tardis-packed executables in
[audit.rules](audit.rules).

## bpftrace script

I've added a short [bpftrace](https://github.com/iovisor/bpftrace) script,
[tardis.bt](tardis.bt), that can be used to detect the relevant syscalls for
the method used by this code.

## seccomp

If your goal is to block the use of `memfd_create` and `execveat` outright, then
you'll want to use [seccomp](https://en.wikipedia.org/wiki/Seccomp).

For instance, if you're running in a Docker container, you can use a [seccomp
profile](https://docs.docker.com/engine/security/seccomp/) with the
`memfd_create` and `execveat` syscalls removed.

## Additional resources

- [tmp.0ut 1.9 by Netspooky](https://tmpout.sh/1/9.html) describes a variant on
  this technique for loading a kernel module in-memory.
  - Netspooky also has [this code example](https://github.com/netspooky/golfclub/blob/master/linux/dl_memfd_219.asm)
    demonstrating calling `memfd_create` and `execve` from assembly.
