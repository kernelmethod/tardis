# Tardis executable packer

`tardis` is a simple [executable packer](https://en.wikipedia.org/wiki/Executable_compression)
I hacked together for [ISTS 2022](https://ists.io/). Its main goals are to be
simple and easy to extend.

Tardis can compress an arbitrary Linux ELF file and convert it into a new
program, which then gets unpacked and executed from memory.

## Compiling and running

You can build the release version of the packer by `cd`'ing into `tardis/` and
running

```
$ cargo build --release
```

To compress an executable, you can run

```
$ cargo run -- pack $input_file $output_file
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

**TODO**

## Detections

**TODO**:

- auditd rules
- YARA rules (?)
- seccomp profiles (?)


