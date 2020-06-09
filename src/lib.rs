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
        let mut messages = Vec::new();

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
                let request_type = format!(
                    ".Lab.Substance.{}.{}.{}.Request",
                    substance_pascal, act_type_pascal, act_pascal
                )
                .to_string();

                let input_type = format!(
                    ".Lab.Substance.Service.{}.{}.{}.Request",
                    substance_pascal, act_type_pascal, act_pascal
                );

                let message = prost_types::DescriptorProto {
                    name: Some(input_type.clone()),
                    field: vec![
                        prost_types::FieldDescriptorProto {
                            type_name: Some(request_type),
                            name: Some("request".to_string()),
                            number: Some(1),
                            ..Default::default()
                        },
                        prost_types::FieldDescriptorProto {
                            type_name: Some(".Lab.Global.Context".to_string()),
                            name: Some("context".to_string()),
                            number: Some(2),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                };
                messages.push(message);

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
            message_type: messages,
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
