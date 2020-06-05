use protobuf_gen::ProtobufString;
use glob::glob;
use itertools::Itertools;
use inflector::Inflector;
use std::fs::File;
use std::io::prelude::*;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let files: Vec<_> = glob("*_protobuf/substance/*/*/*.proto").unwrap().map(Result::unwrap).collect();

    for (dir, files) in files.into_iter().group_by(|f| f.parent().unwrap().parent().unwrap().to_owned()).into_iter() {
        let substance = dir.file_name().unwrap().to_str().unwrap().to_string();
        let substance_pascal = substance.to_pascal_case();
        let mut imports = Vec::new();
        let mut services = Vec::new();

        for (act_type, files) in files.into_iter() .group_by(|f| f.parent().unwrap().file_name().unwrap().to_str().unwrap().to_string()).into_iter() {
            let act_type_pascal = act_type.to_pascal_case();
            let mut methods = Vec::new();

            for file in files {
                let act = file.file_stem().unwrap().to_str().unwrap().to_string();
                let file = file.to_str().unwrap().to_string();
                let act_pascal = act.to_pascal_case();
                imports.push(file);
                methods.push(prost_types::MethodDescriptorProto{
                    name: Some(act),
                    input_type: Some(format!(".Lab.Substance.{}.{}.{}.Request", substance_pascal, act_type_pascal, act_pascal).to_string()),
                    output_type: Some(format!(".Lab.Substance.{}.{}.{}.Response", substance_pascal, act_type_pascal, act_pascal).to_string()),
                    ..Default::default()
                });
            }

            services.push(prost_types::ServiceDescriptorProto{
                name: Some(act_type.to_pascal_case()),
                method: methods,
                ..Default::default()
            });
        }

        let fd = prost_types::FileDescriptorProto{
            name: Some(substance.clone()),
            syntax: Some("proto3".to_string()),
            package: Some(format!("Lab.Substance.{}", substance_pascal)),
            dependency: imports,
            message_type: Vec::new(),
            service: services,
            ..Default::default()
        };

        let mut file = File::create(format!("{}.proto", dir.to_str().unwrap()))?;
        file.write_all(fd.to_protobuf(fd.to_owned()).as_bytes())?;
    }

    Ok(())
}

#[cfg(test)]
mod test {

    #[test]
    fn it_works() -> Result <(), Box<dyn std::error::Error>>{
        super::run()
    }

}
