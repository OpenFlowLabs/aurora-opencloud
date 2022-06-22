use std::fs::create_dir;
use std::io;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_path = Path::new("../proto/vpc.proto");
    let out_dir = Path::new("./src");

    if !Path::exists(out_dir) {
        create_dir(out_dir)?;
    }

    compile_protos(proto_path, out_dir)?;

    Ok(())
}

pub fn compile_protos(proto: impl AsRef<Path>, out_dir: impl AsRef<Path>) -> io::Result<()> {
    let proto_path: &Path = proto.as_ref();

    // directory the main .proto file resides in
    let proto_dir = proto_path
        .parent()
        .expect("proto file should reside in a directory");

    tonic_build::configure()
        .out_dir(out_dir)
        .compile(&[proto_path], &[proto_dir])?;

    Ok(())
}
