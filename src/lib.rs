use glob::glob;
use inflector::Inflector;
use itertools::Itertools;
use protobuf_gen::ProtobufString;
use std::fs::File;
use std::io::prelude::*;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let files: Vec<_> = glob("*_protobuf/substance/*/*/*.proto")
        .unwrap()
        .map(Result::unwrap)
        .collect();

    for (dir, files) in files
        .into_iter()
        .group_by(|f| f.parent().unwrap().parent().unwrap().to_owned())
        .into_iter()
    {
        let substance = dir.file_name().unwrap().to_str().unwrap().to_string();
        let substance_pascal = substance.to_pascal_case();
        let mut imports = Vec::new();
        let mut services = Vec::new();

        for (act_type, files) in files
            .into_iter()
            .group_by(|f| {
                f.parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .into_iter()
        {
            let act_type_pascal = act_type.to_pascal_case();
            let mut methods = Vec::new();

            for file in files {
                let act = file.file_stem().unwrap().to_str().unwrap().to_string();
                let file = file.to_str().unwrap().to_string();
                let act_pascal = act.to_pascal_case();
                let input_type = format!(
                    ".Lab.Substance.{}.{}.{}.Request",
                    substance_pascal, act_type_pascal, act_pascal
                )
                .to_string();

                let output_type = if act_type == "observation" {
                    imports.push("lab_protobuf/global/shared_observation.proto".to_string());
                    ".Lab.Global.SharedObservation.Response".to_string()
                } else if act_type == "effect" {
                    imports.push("lab_protobuf/global/shared_effect.proto".to_string());
                    ".Lab.Global.SharedEffect.Response".to_string()
                } else {
                    format!(
                        ".Lab.Substance.{}.{}.{}.Response",
                        substance_pascal, act_type_pascal, act_pascal
                    )
                    .to_string()
                };

                imports.push(file);
                methods.push(prost_types::MethodDescriptorProto {
                    name: Some(act),
                    input_type: Some(input_type),
                    output_type: Some(output_type),
                    ..Default::default()
                });
            }

            services.push(prost_types::ServiceDescriptorProto {
                name: Some(act_type.to_pascal_case()),
                method: methods,
                ..Default::default()
            });
        }

        let fd = prost_types::FileDescriptorProto {
            name: Some(substance.clone()),
            syntax: Some("proto3".to_string()),
            package: Some(format!("Lab.Substance.Service.{}", substance_pascal)),
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
    fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        super::run()
    }
}
