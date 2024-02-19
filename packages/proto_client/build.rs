fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../proto/qcdn");
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .build_server(false)
        .compile(
            &[
                "../../proto/qcdn/general.proto",
                "../../proto/qcdn/file.proto",
            ],
            &["../../proto"],
        )?;
    Ok(())
}
