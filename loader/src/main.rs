//! Loader executable for Tardis
//!
//! This program is in charge of reading the compressed binary from
//! the manifest, decompressing it in memory, and then running it.

use deku::DekuContainerRead;
use libtardis::serialization::{EndMarker, TardisResource};
use libtardis::syscall as sysc;
use nix::unistd::{fork, ForkResult};
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
    let flags = 0x1; // MFD_CLOEXEC
    let fd = unsafe { sysc::memfd_create(name, flags) };

    // Write the guest binary to the in-memory file
    let mut f = unsafe { File::from_raw_fd(fd) };
    f.write_all(&guest)?;

    // Use execveat to run the binary
    let argv: Vec<CString> = env::args().filter_map(|x| CString::new(x).ok()).collect();

    let envp: Vec<CString> = env::vars()
        .map(|(k, v)| format!("{k}={v}\0"))
        .filter_map(|x| CString::new(x).ok())
        .collect();

    let path = CStr::from_bytes_with_nul(b"\0")?;
    let flags = 0x1000; // AT_EMPTY_PATH

    unsafe {
        sysc::execveat(fd, path, &argv, &envp, flags);
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    let host = fs::read("/proc/self/exe").unwrap();
    let marker_start = host.len() - EndMarker::nbytes();

    let (_, marker) = EndMarker::from_bytes((&host[marker_start..], 0))?;
    let mut offset = marker.manifest_start;

    // Read the next resource and fork a new process off
    // of it
    for _ in 0..marker.n_resources {
        let (_, resource) = TardisResource::from_bytes((&host[offset..], 0))?;
        offset += resource.len();

        // Only fork off processes if there is more than one executable that needs
        // to be launched.
        if marker.n_resources == 1 {
            spawn_guest(resource)?;

            // Should not reach this point
            return Ok(());
        }

        match unsafe { fork() } {
            Ok(ForkResult::Child) => spawn_guest(resource)?,
            Ok(_) => continue,
            Err(e) => return Err(Box::new(e)),
        }
    }

    Ok(())
}
