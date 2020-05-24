extern crate inflector;
extern crate protobuf_parser;
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

pub trait ProtoGen {
    fn emit_proto(&self, buf: &mut String, indent_depth: usize);
}

impl ProtoGen for FileDescriptor {
    fn emit_proto(&self, buf: &mut String, indent_depth: usize) {
        let indent = " ".repeat(indent_depth);
        self.syntax.emit_proto(buf, indent_depth);
        buf.push_str(&format!("{}package {};\n", indent, self.package));
        for imp in &self.import_paths {
            buf.push_str(&format!("{}import \"{}\";\n", indent, imp));
        }
        for msg in &self.messages {
            msg.emit_proto(buf, indent_depth);
        }
    }
}

impl ProtoGen for Message {
    fn emit_proto(&self, buf: &mut String, indent_depth: usize) {
        let indent = " ".repeat(indent_depth);
        buf.push_str(&format!("{}message {} {{\n", indent, self.name));
        let has_reserved = self.reserved_nums.len() > 0;
        if has_reserved {
            buf.push_str(&format!("{}  reserved ", indent));
        }
        for (i, range) in self.reserved_nums.iter().enumerate() {
            if i != 0 {
                buf.push_str(", ");
            }
            if range.start == range.end - 1 {
                buf.push_str(&format!("{}", range.start));
            } else {
                buf.push_str(&format!("{} to {}", range.start, range.end - 1));
            }
        }
        if has_reserved {
            buf.push_str(";\n");
        }
        for msg in &self.fields {
            msg.emit_proto(buf, indent_depth + 2);
        }
        for msg in &self.oneofs {
            msg.emit_proto(buf, indent_depth + 2);
        }
        for msg in &self.messages {
            msg.emit_proto(buf, indent_depth + 2);
        }
        buf.push_str(&format!("{}}}\n", indent));
    }
}

impl ProtoGen for OneOf {
    fn emit_proto(&self, buf: &mut String, indent_depth: usize) {
        let indent = " ".repeat(indent_depth);
        buf.push_str(&format!("{}oneof {} {{\n", indent, self.name));
        for msg in &self.fields {
            msg.emit_proto(buf, indent_depth + 2);
        }
        buf.push_str(&format!("{}}}\n", indent));
    }
}

impl ProtoGen for Field {
    fn emit_proto(&self, buf: &mut String, indent_depth: usize) {
        buf.push_str(&" ".repeat(indent_depth));
        self.typ.emit_proto(buf, 0);
        self.rule.emit_proto(buf, 0);
        buf.push_str(" ");
        buf.push_str(&self.name);
        buf.push_str(" = ");
        buf.push_str(&self.number.to_string());
        if self.deprecated {
            buf.push_str(" [deprecated = true]");
        }
        buf.push_str(";\n");
    }
}

impl ProtoGen for FieldType {
    fn emit_proto(&self, buf: &mut String, _indent_depth: usize) {
        match self {
            FieldType::Int32 => buf.push_str("int32"),
            FieldType::Int64 => buf.push_str("int64"),
            FieldType::Uint32 => buf.push_str("uint32"),
            FieldType::Uint64 => buf.push_str("uint64"),
            FieldType::Sint32 => buf.push_str("sint32"),
            FieldType::Sint64 => buf.push_str("sint64"),
            FieldType::Fixed32 => buf.push_str("fixed32"),
            FieldType::Fixed64 => buf.push_str("fixed64"),
            FieldType::Sfixed32 => buf.push_str("sfixed32"),
            FieldType::Sfixed64 => buf.push_str("sfixed64"),
            FieldType::Bool => buf.push_str("bool"),
            FieldType::String => buf.push_str("string"),
            FieldType::Bytes => buf.push_str("bytes"),
            FieldType::Float => buf.push_str("float"),
            FieldType::Double => buf.push_str("double"),
            FieldType::Group(_) => (),
            FieldType::MessageOrEnum(name) => buf.push_str(name),
            FieldType::Map(types_box) => {
                let (key_type, value_type) = &**types_box;
                buf.push_str("<");
                key_type.emit_proto(buf, 0);
                buf.push_str(" ");
                value_type.emit_proto(buf, 0);
                buf.push_str(">");
            }
        }
    }
}

impl ProtoGen for Rule {
    fn emit_proto(&self, buf: &mut String, _indent_depth: usize) {
        match self {
            Rule::Repeated => buf.push_str("repeated "),
            Rule::Required => (),
            Rule::Optional => (),
        }
    }
}
impl ProtoGen for Syntax {
    fn emit_proto(&self, buf: &mut String, indent_depth: usize) {
        let indent = " ".repeat(indent_depth);
        let syntax = match self {
            Syntax::Proto2 => "proto2",
            Syntax::Proto3 => "proto3",
        };
        buf.push_str(&format!("{}syntax = \"{}\";\n", indent, syntax));
    }
}
