use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path(out_dir.join("policy_service_descriptor.bin"))
        .compile(
            &["proto/policy_service.proto"],
            &["proto"],
        )?;

    Ok(())
}
