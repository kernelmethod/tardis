//! Loader executable for Tardis
//!
//! This program is in charge of reading the compressed binary from
//! the manifest, decompressing it in memory, and then running it.

// extern crate panic_abort;

use deku::DekuContainerRead;
use libtardis::serialization::{EndMarker, TardisResource};
use nix::{
    fcntl::AtFlags,
    sys::memfd::{memfd_create, MemFdCreateFlag},
    unistd::execveat,
};
use std::{
    env,
    error::Error,
    ffi::{CStr, CString},
    fs::{self, File},
    io::Write,
    os::unix::io::FromRawFd,
};

fn spawn_guest(res: TardisResource) -> Result<(), Box<dyn Error>> {
    // Decompress the guest
    let guest = res.decompress().expect("invalid lz4 payload");

    // Exec into the guest binary by creating an  in-memory file
    // with memfd_create
    let name = match CStr::from_bytes_with_nul(b"a\0") {
        Ok(fd) => fd,
        Err(_) => panic!("aborted"),
    };
    let flags = MemFdCreateFlag::MFD_CLOEXEC;
    let fd = match memfd_create(name, flags) {
        Ok(fd) => fd,
        Err(_) => panic!("aborted"),
    };

    // Write the guest binary to the in-memory file
    let mut f = unsafe { File::from_raw_fd(fd) };
    f.write_all(&guest)?;

    // Use execveat to run the binary
    let argv: Vec<CString> = env::args().filter_map(|x| CString::new(x).ok()).collect();

    let envp: Vec<CString> = env::vars()
        .map(|(k, v)| format!("{}={}\0", k, v))
        .filter_map(|x| CString::new(x).ok())
        .collect();

    let path = CStr::from_bytes_with_nul(b"\0")?;
    let flags = AtFlags::AT_EMPTY_PATH;

    execveat(fd, path, &argv, &envp, flags)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let host = fs::read("/proc/self/exe").unwrap();
    let marker_start = host.len() - EndMarker::nbytes();

    let (_, marker) = EndMarker::from_bytes((&host[marker_start..], 0))?;

    let mut offset = marker.manifest_start;

    for _ in 0..marker.n_resources {
        // Read the next resource and fork a new process off
        // of it
        let (_, resource) = TardisResource::from_bytes((&host[offset..], 0))?;
        let resource_len = resource.len();
        spawn_guest(resource)?;

        // Update the offset into the manifest
        offset += resource_len;
    }

    Ok(())
}
