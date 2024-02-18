fn main() -> Result<(), Box<dyn std::error::Error>> {
    // trigger recompilation when a migration is changed
    println!("cargo:rerun-if-changed=../migrations");

    println!("cargo:rerun-if-changed=../proto_client/qcdn");
    tonic_build::configure().compile(
        &[
            "../proto_client/qcdn/general.proto_client",
            "../proto_client/qcdn/files.proto_client",
            "../proto_client/qcdn/nodes.proto_client",
        ],
        &["proto_client"],
    )?;
    Ok(())
}
