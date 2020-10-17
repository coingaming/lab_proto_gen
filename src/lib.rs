use glob::glob;
use inflector::Inflector;
use itertools::Itertools;
use protobuf_gen::ProtobufString;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let files: Vec<_> = glob("*_protobuf/substance/*/*/*.proto")
        .unwrap()
        .map(Result::unwrap)
        .collect();

    for (dir, files) in files
        .into_iter()
        .group_by(|f| f.parent().unwrap().to_owned())
        .into_iter()
    {
        let act_type = dir.file_name().unwrap().to_str().unwrap().to_string();
        let act_type_pascal = act_type.to_pascal_case();

        let substance = dir
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let substance_pascal = substance.to_pascal_case();

        let proto_dir = dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let mut imports: Vec<_> = files.collect();
        let mut services = Vec::with_capacity(1);
        let mut messages = Vec::with_capacity(imports.len());
        let mut methods = Vec::with_capacity(imports.len());

        for file in imports.clone() {
            let act = file.file_stem().unwrap().to_str().unwrap().to_string();
            let act_pascal = act.to_pascal_case();
            let request_type = format!(
                ".Lab.Substance.{}.{}.{}.Request",
                substance_pascal, act_type_pascal, act_pascal
            )
            .to_string();

            if act_type == "observation" {
                imports.push(PathBuf::from(format!(
                    "{}/substance/{}/effect/{}.proto",
                    proto_dir, substance, act
                )));
            }

            let input_type = format!("{}Request", act_pascal.clone());

            let message = prost_types::DescriptorProto {
                name: Some(input_type.clone()),
                field: vec![
                    prost_types::FieldDescriptorProto {
                        type_name: Some(".Lab.Global.Context".to_string()),
                        name: Some("context".to_string()),
                        number: Some(1),
                        ..Default::default()
                    },
                    prost_types::FieldDescriptorProto {
                        type_name: Some(request_type),
                        name: Some("request".to_string()),
                        number: Some(2),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            };
            messages.push(message);

            let output_type = if act_type == "observation" {
                format!(
                    ".Lab.Substance.{}.Effect.{}.Request",
                    substance_pascal, act_pascal
                )
                .to_string()
            } else if act_type == "effect" {
                ".Lab.Global.SharedEffect.Response".to_string()
            } else {
                format!(
                    ".Lab.Substance.{}.{}.{}.Response",
                    substance_pascal, act_type_pascal, act_pascal
                )
                .to_string()
            };

            methods.push(prost_types::MethodDescriptorProto {
                name: Some(act_pascal.clone()),
                input_type: Some(input_type),
                output_type: Some(output_type),
                server_streaming: Some(act_type == "observation"),
                ..Default::default()
            });
        }

        imports.push(PathBuf::from("lab_protobuf/global/context.proto"));

        if act_type == "effect" {
            imports.push(PathBuf::from("lab_protobuf/global/shared_effect.proto"));
        };

        services.push(prost_types::ServiceDescriptorProto {
            name: Some("Service".to_string()),
            method: methods,
            ..Default::default()
        });

        let fd = prost_types::FileDescriptorProto {
            // name: Some(substance.clone()),
            syntax: Some("proto3".to_string()),
            package: Some(format!("Lab.Rpc.{}.{}", substance_pascal, act_type_pascal)),
            dependency: imports
                .iter()
                .map(|x| x.to_str().unwrap().to_string())
                .collect(),
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
