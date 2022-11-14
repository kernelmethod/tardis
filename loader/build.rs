fn main() {
    // We apply some additional optimizations to reduce binary size
    // when we're building with the --embed profile
    let profile = std::env::var("PROFILE").unwrap();

    if profile.as_str() == "release" {
        // Tell Cargo to build the loader without any debug symbols. This
        // helps make the loader much smaller.
        //
        // See: https://github.com/rust-lang/cargo/issues/3483
        println!("cargo:rustc-link-arg=-s");
    }
}
