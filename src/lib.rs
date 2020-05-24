mod extender;
mod gen;
mod parser;

use std::path::PathBuf;
use std::fs::{self, File};
use std::io::prelude::*;
use clap::Clap;
use extender::*;
use gen::ProtoGen;

#[derive(Clap, Debug)]
#[clap(name = "Lab protobuf generator and extender")]
pub struct Config {
    #[clap(short, parse(from_os_str))]
    pub file: PathBuf,
    #[clap(short, arg_enum, default_value = "deprecate")]
    pub absent_field_action: extender::AbsentFieldAction,
    #[clap(name = "DIR", parse(from_os_str), required = true, min_values = 2)]
    pub subs_dir: Vec<PathBuf>
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    // parse proto based on the proto roots dir structure
    let new_proto = parser::proto_from_fs(&config.subs_dir)?;

    // read existing proto from file and extend it with new proto while staying backward compatible
    let proto = if let Ok(old_proto_str) = fs::read_to_string(&config.file) {
        let mut old_proto = protobuf_parser::FileDescriptor::parse(old_proto_str)
            .expect("failed to parse existing protobuf");
        old_proto.extend(&new_proto, config.absent_field_action, None);
        old_proto
    } else {
        new_proto
    };

    // generate proto syntax and save to file
    let mut out = String::new();
    proto.emit_proto(&mut out, 0);
    let mut file = File::create(&config.file)?;
    file.write_all(out.as_bytes())?;

    Ok(())
}
