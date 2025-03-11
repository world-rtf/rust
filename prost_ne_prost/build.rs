use std::io::Result;
extern crate prost_build; // компиляция файлов protobuf

// https://docs.rs/prost-build/latest/prost_build/
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/proto/addressbook.proto"], &["src/proto/"])?;
    Ok(())
}