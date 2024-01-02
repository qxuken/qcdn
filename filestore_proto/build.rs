fn main() -> Result<(), Box<dyn std::error::Error>> {
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
