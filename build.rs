fn main() -> Result<(), Box<dyn std::error::Error>> {
    // trigger recompilation when a migration is changed
    println!("cargo:rerun-if-changed=migrations");

    tonic_build::configure().compile(
        &[
            "proto/qcdn/general.proto",
            "proto/qcdn/files.proto",
            "proto/qcdn/nodes.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
