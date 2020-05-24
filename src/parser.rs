use std::path::PathBuf;
use inflector::cases::pascalcase::to_pascal_case;
use protobuf_parser::*;

#[derive(Debug, Clone)]
struct LabSpec(Vec<String>, Vec<Substance>);

#[derive(Debug, Clone)]
struct Substance {
    name: String,
    reactions: Vec<String>,
    elements: Vec<String>,
    observations: Vec<String>,
    effects: Vec<String>,
}

impl std::ops::Add for LabSpec {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let LabSpec(a1, a2) = self;
        let LabSpec(b1, b2) = other;
        let mut c1 = a1.clone();
        let mut c2 = a2.clone();
        c1.extend(b1.iter().cloned());
        c2.extend(b2.iter().cloned());
        LabSpec(c1, c2)
    }
}

pub fn proto_from_fs(
    root_paths: &Vec<PathBuf>,
) -> Result<FileDescriptor, Box<dyn std::error::Error>> {
    let LabSpec(mut paths, subs) = root_paths
        .iter()
        .map(|path| parse_lab_spec(path))
        .collect::<Result<Vec<LabSpec>, _>>()?
        .into_iter()
        .fold(LabSpec(vec![], vec![]), |acc, spec| acc + spec);

    paths.push(String::from("lab_protobuf/global/context.proto"));
    paths.push(String::from("lab_protobuf/global/shared_observation.proto"));
    paths.push(String::from("lab_protobuf/global/shared_effect.proto"));

    let messages = vec![
        build_payload_message(&subs, "Request"),
        build_payload_message(&subs, "Response"),
    ];

    Ok(FileDescriptor {
        syntax: Syntax::Proto3,
        import_paths: paths,
        package: String::from("Lab.Payload"),
        messages,
        enums: vec![],
        extensions: vec![],
    })
}

fn parse_lab_spec(path: &PathBuf) -> Result<LabSpec, Box<dyn std::error::Error>> {
    let mut paths: Vec<String> = vec![];
    let mut substances = Vec::new();

    for entry in path
        .read_dir()
        .or_else(|e| Err(format!("failed to read dir '{}': {}", path.to_str().unwrap(), e)))?
    {
        let entry = entry?;

        if entry.metadata()?.is_dir() {
            let mut substance = Substance {
                name: String::from(entry.path().file_stem().unwrap().to_str().unwrap()),
                reactions: Vec::new(),
                elements: Vec::new(),
                observations: Vec::new(),
                effects: Vec::new(),
            };

            for entry in entry.path().read_dir()? {
                let entry = entry?;

                if entry.metadata()?.is_dir() {
                    let acts = match entry.path().file_stem().unwrap().to_str().unwrap() {
                        "reaction" => &mut substance.reactions,
                        "element" => &mut substance.elements,
                        "observation" => &mut substance.observations,
                        "effect" => &mut substance.effects,
                        unknown => panic!("what is this act kind? {}", unknown),
                    };
                    for entry in entry.path().read_dir()? {
                        let entry = entry?;
                        let path = entry.path();
                        paths.push(String::from(path.to_str().unwrap()));
                        acts.push(String::from(path.file_stem().unwrap().to_str().unwrap()));
                    }
                }
            }
            substances.push(substance)
        }
    }
    Ok(LabSpec(paths, substances))
}

fn get_substance_acts<'a>(sub: &'a Substance, act_type: &str) -> &'a Vec<String> {
    match act_type {
        "reaction" => &sub.reactions,
        "element" => &sub.elements,
        "observation" => &sub.observations,
        "effect" => &sub.effects,
        _ => panic!(),
    }
}

fn build_payload_message(subs: &Vec<Substance>, payload_type: &str) -> Message {
    let fields = vec![
        Field {
            name: String::from("id"),
            number: 1,
            typ: FieldType::Uint32,
            rule: Rule::Optional,
            default: None,
            packed: None,
            deprecated: false,
        },
        Field {
            name: String::from("context"),
            number: 2,
            typ: FieldType::MessageOrEnum(String::from(".Lab.Global.Context")),
            rule: Rule::Optional,
            default: None,
            packed: None,
            deprecated: false,
        },
    ];

    let oneof_fields = vec![
        Field {
            name: String::from("reaction"),
            number: 3,
            typ: FieldType::MessageOrEnum(String::from("Reaction")),
            rule: Rule::Optional,
            default: None,
            packed: None,
            deprecated: false,
        },
        Field {
            name: String::from("element"),
            number: 4,
            typ: FieldType::MessageOrEnum(String::from("Element")),
            rule: Rule::Optional,
            default: None,
            packed: None,
            deprecated: false,
        },
        Field {
            name: String::from("observation"),
            number: 5,
            typ: FieldType::MessageOrEnum(String::from(if payload_type == "Response" {
                ".Lab.Global.SharedObservation.Response"
            } else {
                "Observation"
            })),
            rule: Rule::Optional,
            default: None,
            packed: None,
            deprecated: false,
        },
        Field {
            name: String::from("effect"),
            number: 6,
            typ: FieldType::MessageOrEnum(String::from(if payload_type == "Response" {
                ".Lab.Global.SharedEffect.Response"
            } else {
                "Effect"
            })),
            rule: Rule::Optional,
            default: None,
            packed: None,
            deprecated: false,
        },
    ];

    let mut messages = vec![
        build_act_message(subs, payload_type, "reaction"),
        build_act_message(subs, payload_type, "element"),
    ];

    if payload_type == "Request" {
        messages.push(build_act_message(subs, payload_type, "observation"));
        messages.push(build_act_message(subs, payload_type, "effect"));
    }

    Message {
        name: String::from(payload_type),
        fields,
        oneofs: vec![OneOf {
            name: String::from("act_type"),
            fields: oneof_fields,
        }],
        messages,
        reserved_nums: vec![],
        reserved_names: vec![],
        enums: vec![],
    }
}

fn build_act_message(subs: &Vec<Substance>, payload_type: &str, act_type: &str) -> Message {
    let oneof_fields: Vec<_> = subs
        .iter()
        .filter(|sub| !get_substance_acts(sub, act_type).is_empty())
        .enumerate()
        .map(|(i, sub)| Field {
            name: String::from(&sub.name),
            number: i as i32 + 1,
            typ: FieldType::MessageOrEnum(to_pascal_case(&sub.name)),
            rule: Rule::Optional,
            default: None,
            packed: None,
            deprecated: false,
        })
        .collect();

    let messages: Vec<_> = subs
        .iter()
        .filter(|sub| !get_substance_acts(sub, act_type).is_empty())
        .map(|sub| build_substance_message(sub, payload_type, act_type))
        .collect();

    Message {
        name: to_pascal_case(act_type),
        fields: vec![],
        oneofs: vec![OneOf {
            name: String::from("substance"),
            fields: oneof_fields,
        }],
        messages,
        reserved_nums: vec![],
        reserved_names: vec![],
        enums: vec![],
    }
}

fn build_substance_message(sub: &Substance, payload_type: &str, act_type: &str) -> Message {
    let acts = get_substance_acts(sub, act_type);
    let oneof_fields: Vec<_> = acts
        .iter()
        .enumerate()
        .map(|(i, act)| Field {
            name: String::from(act),
            number: i as i32 + 1,
            typ: FieldType::MessageOrEnum(format!(
                ".Lab.Substance.{}.{}.{}.{}",
                to_pascal_case(&sub.name),
                to_pascal_case(act_type),
                to_pascal_case(&act),
                payload_type
            )),
            rule: Rule::Optional,
            default: None,
            packed: None,
            deprecated: false,
        })
        .collect();

    Message {
        name: to_pascal_case(&sub.name),
        fields: vec![],
        oneofs: vec![OneOf {
            name: String::from("act"),
            fields: oneof_fields,
        }],
        messages: vec![],
        reserved_nums: vec![],
        reserved_names: vec![],
        enums: vec![],
    }
}
