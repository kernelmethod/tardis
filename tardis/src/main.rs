//! # Tardis executable packer
//!
//! Tardis is a proof-of-concept `memfd_create` and `execveat`-based [executable
//! packer](https://en.wikipedia.org/wiki/Executable_compression) for Linux. It can decrypt and
//! decompress a binary entirely in memory, without leaving any binary artefacts on-disk.
//!
//! ## Usage
//!
//! To compress an executable using the CLI, run `./tardis input_file output_file`. For instance,
//! the following command packs `/usr/bin/ls` and writes it to the file `./packed_ls`.
//!
//! ```
//! $ ./tardis /usr/bin/ls ./packed_ls
//! Wrote ./ls (917.50% of input)
//! ```
//!
//! > **Warning:** `tardis` is not especially effective as an all-around packer for smaller
//! > binaries. The overhead incurred in adding the loader is typically much higher than the
//! > savings from compression at the lower end.

use clap::Parser;
use deku::DekuContainerWrite;
use libtardis::serialization::{EndMarker, TardisResource};
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;

/// Add a new guest binary to the output file. Returns the number
/// of bytes that were written to the file.
fn add_guest(f: &mut File, guest: &[u8]) -> Result<usize, Box<dyn Error>> {
    // Compress the guest binary and write it to the file
    let resource = TardisResource::compress(guest);
    let resource_bytes = resource.to_bytes().unwrap();
    f.write_all(&resource_bytes)?;

    Ok(resource_bytes.len())
}

const LOADER: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/embeds/release/loader"
));

fn pack(input_file: &str, output_file: &str) -> Result<(), Box<dyn Error>> {
    // Write the loader to the output file
    let mut guests_size = 0;
    let mut output = File::create(output_file)?;

    output.write_all(LOADER)?;

    // Set the same permissions on the output file that existed on the
    // input file
    let input_perms = File::open(input_file)?.metadata()?.permissions();
    output.set_permissions(input_perms)?;

    // Read the input executable into memory
    //
    // TODO (kernelmethod): read the file in chunks in case it's too
    // large for us to fit into memory
    let data = fs::read(input_file)?;
    let orig_size = data.len();

    // Compress the executable and write it to the output file in a new
    // data block
    guests_size += add_guest(&mut output, &data)?;

    // Write the EndMarker to the output file
    let marker = EndMarker {
        manifest_start: LOADER.len(),
        n_resources: 1,
    };
    let marker_bytes = marker.to_bytes().unwrap();
    output.write_all(&marker_bytes)?;

    let output_size = LOADER.len() + guests_size;
    println!(
        "Wrote {} ({:.2}% of input)",
        output_file,
        output_size as f64 / orig_size as f64 * 100.
    );

    Ok(())
}

/// Simple executable packer for Linux using the memfd_create and openat
/// syscalls.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the executable to compress.
    #[arg(short, long)]
    input_file: String,

    /// Name of the output file to write to.
    #[arg(short, long)]
    output_file: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    pack(&args.input_file, &args.output_file)?;

    Ok(())
}
