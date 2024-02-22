# Tardis: multi-executable packer

`tardis` is a simple [executable packer](https://en.wikipedia.org/wiki/Executable_compression)
I hacked together for [ISTS 2022](https://ists.io/). Its main goals are to be
simple and easy to extend.

Tardis can compress an arbitrary Linux ELF file and convert it into a new
program, which then gets unpacked and executed from memory. It can also compress
multiple executables _into the same binary_. In this case, Tardis will fork off
a new process for each binary that it unpacks.

## Compiling and running

You can build the release version of the packer by `cd`'ing into `tardis/` and
running

```
$ cargo build --release
```

To compress an executable, you can run

```
$ cargo run -- -i $input_file -o $output_file
```

You can compress multiple executables so that they run concurrently:

```
$ cargo run -- -i $exe1 -i $exe2 -i $exe3 -o $output_file
```

## Important usage notes

**Binary sizes:** this is a very simple packer implementation. The `loader`
binary is statically linked with musl, and is still ~400Kb after stripping. As a
result, it'll only really compress binaries that are at least 1-2Mb. This makes
`tardis` good for obfuscating code and relatively large payloads, but not so
good as a general-purpose packer. A general-purpose packer would probably want
the loader to be written as a `no_std` binary so that it doesn't need to be
linked with a libc.

**Linux versions:** this code relies on the `memfd_create` and `execveat` Linux
syscalls in order to run. As such, it won't run for Linux versions before 3.19.

## Methodology

`tardis` compresses and encrypts a binary using LZ4 and ChaCha20-Poly1305,
creating a new ELF file. At runtime, it decompresses and runs itself using the
[`memfd_create`](https://man7.org/linux/man-pages/man2/memfd_create.2.html) and
[`execveat`](https://man7.org/linux/man-pages/man2/execveat.2.html) syscalls.
This method runs *entirely* from memory, without leaving any artefacts on disk.

### Additional resources

The `memfd_create` + `execveat` methodology is a relatively simple and fairly
well-known technique in a family of methods known as *reflective code loading*.
Here are some additional resources you can look at to learn more:

- [MITRE ATT&CK T1620: Reflective Code
  Loading](https://attack.mitre.org/techniques/T1620/)

- [tmp.0ut 1.9 by Netspooky](https://tmpout.sh/1/9.html) describes a variant on
  this technique for loading a kernel module in-memory.
  - Netspooky also has [this code example](https://github.com/netspooky/golfclub/blob/master/linux/dl_memfd_219.asm)
    demonstrating calling `memfd_create` and `execve` from assembly.

- [Forum post on 0x00sec](https://0x00sec.org/t/super-stealthy-droppers/3715)
  describing this technique.

