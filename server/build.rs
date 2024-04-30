use std::{env, error::Error, fs, os::unix::ffi::OsStrExt};

fn main() -> Result<(), Box<dyn Error>> {
  let proto_dir = env::current_dir()?.parent().unwrap().join("proto");
  prost_build::compile_protos(
    &fs::read_dir(&proto_dir)?
      .filter_map(|proto_file| {
        proto_file.map_or(None, |proto_file| {
          if proto_file.file_name().as_bytes().ends_with(b".proto") {
            Some(proto_file)
          } else {
            None
          }
        })
      })
      .map(|proto_file| proto_file.path())
      .collect::<Vec<_>>()[..],
    &[&proto_dir],
  )?;
  Ok(())
}
