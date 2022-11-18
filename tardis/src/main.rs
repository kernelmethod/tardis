//! Main executable for running the Tardis packer.

use clap::{app_from_crate, arg, AppSettings};
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
    let resource_bytes = resource.to_bytes()?;
    f.write_all(&resource_bytes)?;

    Ok(resource_bytes.len())
}

const LOADER: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/embeds/x86_64-unknown-linux-gnu/release/loader"
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
    let marker_bytes = marker.to_bytes()?;
    output.write_all(&marker_bytes)?;

    let output_size = LOADER.len() + guests_size;
    println!(
        "Wrote {} ({:.2}% of input)",
        output_file,
        output_size as f64 / orig_size as f64 * 100.
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = app_from_crate!()
        .about("Tardis executable packer for Linux")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(arg!([INPUT_FILE]).required(true))
        .arg(arg!([OUTPUT_FILE]).required(true));

    let m = app.get_matches();
    let input_file = m.value_of("INPUT_FILE").unwrap();
    let output_file = m.value_of("OUTPUT_FILE").unwrap();
    pack(input_file, output_file)?;

    Ok(())
}
