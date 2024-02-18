fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../proto/qcdn");
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_client(false)
        .compile(
            &[
                "../../proto/qcdn/general.proto",
                "../../proto/qcdn/files.proto",
            ],
            &["../../proto"],
        )?;
    Ok(())
}
