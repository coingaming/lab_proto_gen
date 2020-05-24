extern crate protobuf_parser;
use crate::gen::ProtoGen;
use protobuf_parser::*;
use clap::Clap;
use std::collections::HashMap;
use std::collections::HashSet;

pub type Ret = Option<(usize, Vec<std::ops::Range<i32>>)>;

#[derive(Clone, Copy, Debug, Clap)]
pub enum AbsentFieldAction {
    Deprecate,
    Reserve,
    Remove
}

pub trait ProtoExtender {
    fn extend(&mut self, other: &Self, rem_beh: AbsentFieldAction, last_number: Option<usize>) -> Ret;
}

impl ProtoExtender for FileDescriptor {
    fn extend(&mut self, other: &Self, rem_beh: AbsentFieldAction, _last_number: Option<usize>) -> Ret {
        self.import_paths.extend(other.import_paths.clone());
        self.import_paths.sort();
        self.import_paths.dedup();
        self.messages.extend(other.messages.clone());
        self.messages.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        self.messages.dedup_by(|a, b| {
            if a.name == b.name {
                ProtoExtender::extend(b, a, rem_beh, None);
                true
            } else {
                false
            }
        });
        None
    }
}

impl ProtoExtender for Message {
    fn extend(&mut self, other: &Self, rem_beh: AbsentFieldAction, _last_number: Option<usize>) -> Ret {
        // extend sub-messages
        self.messages.extend(other.messages.clone());
        self.messages.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        self.messages.dedup_by(|a, b| {
            if a.name == b.name {
                ProtoExtender::extend(b, a, rem_beh, None);
                true
            } else {
                false
            }
        });

        // find max field number
        let mut number = self
            .fields
            .iter()
            .fold(1, |acc, field| std::cmp::max(acc, field.number as usize));

        // find max reserved number
        number = std::cmp::max(
            number,
            self.reserved_nums
                .iter()
                .fold(number, |acc, range| std::cmp::max(acc, range.end as usize - 1)),
        );

        // find max oneof field number
        number = std::cmp::max(
            number,
            self.oneofs.iter().fold(number, |acc, oneof| {
                std::cmp::max(
                    acc,
                    oneof
                        .fields
                        .iter()
                        .fold(acc, |acc, field| std::cmp::max(acc, field.number as usize)),
                )
            }),
        );

        // extend oneoffs
        let mut reserved = self.reserved_nums.clone();
        self.oneofs.extend(other.oneofs.clone());
        self.oneofs.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        self.oneofs.dedup_by(|a, b| {
            if a.name == b.name {
                let (_, oneof_reserved) = ProtoExtender::extend(b, a, rem_beh, Some(number)).unwrap();
                reserved.extend(oneof_reserved);
                true
            } else {
                false
            }
        });

        // prepare data for diffing and extend
        let mut self_map = HashMap::new();
        for field in &self.fields {
            self_map.insert(&field.name, field);
        }
        let self_names: HashSet<_> = self_map.keys().collect();

        let mut other_map = HashMap::new();
        for field in &other.fields {
            other_map.insert(&field.name, field);
        }
        let other_names: HashSet<_> = other_map.keys().collect();

        let mut new_fields = self.fields.clone();

        // remove and reserve absent fields from self
        for field_name in self_names.difference(&other_names) {
            let mut field = (*self_map.get(*field_name).unwrap()).clone();
            let mut proto = String::new();
            field.emit_proto(&mut proto, 0);
            new_fields.retain(|f| f.number != field.number);
            match rem_beh {
                AbsentFieldAction::Deprecate => {
                    if field.deprecated {
                        print!("Keeping deprecated: {}", proto);
                    }
                    else {
                        print!("Deprecating: {}", proto);
                        field.deprecated = true;
                    }
                    new_fields.push(field);
                },
                AbsentFieldAction::Reserve => {
                    print!("Removing and reserving: {}", proto);
                    reserved.push(field.number..field.number + 1);
                },
                AbsentFieldAction::Remove => {
                    print!("Removing: {}", proto);
                }
            }
        }
        
        if let AbsentFieldAction::Deprecate = rem_beh {
            // merge ranges
            reserved.sort_unstable_by_key(|range| range.start);
            reserved.dedup_by(|a, b| {
                if b.contains(&(a.start + 1)) || a.contains(&b.end) {
                    b.start = std::cmp::min(a.start, b.start);
                    b.end = std::cmp::max(a.end, b.end);
                    true
                }
                else {
                    false
                }
            });
            self.reserved_nums = reserved;
        }

        // extend self with new fields
        for field_name in other_names.difference(&self_names) {
            let mut field = (*other_map.get(*field_name).unwrap()).clone();
            let mut proto = String::new();
            field.emit_proto(&mut proto, 0);
            print!("Added: {}", proto);
            number += 1;
            field.number = number as i32;
            new_fields.push(field);
        }
        self.fields = new_fields;

        // update fields
        self.fields.extend(other.fields.clone());
        self.fields.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        self.fields.dedup_by(|a, b| {
            if a.name == b.name {
                ProtoExtender::extend(b, a, rem_beh, Some(number));
                true
            } else {
                false
            }
        });
        self.fields.sort_unstable_by(|a, b| a.number.cmp(&b.number));

        None
    }
}

impl ProtoExtender for OneOf {
    fn extend(&mut self, other: &Self, rem_beh: AbsentFieldAction, last_number: Option<usize>) -> Ret {
        // prepare data for diffing and extend
        let mut self_map = HashMap::new();
        for field in &self.fields {
            self_map.insert(&field.name, field);
        }
        let self_names: HashSet<_> = self_map.keys().collect();

        let mut other_map = HashMap::new();
        for field in &other.fields {
            other_map.insert(&field.name, field);
        }
        let other_names: HashSet<_> = other_map.keys().collect();

        // remove and reserve absent fields from self
        let mut new_fields = self.fields.clone();
        let mut reserved: Vec<std::ops::Range<i32>> = Vec::new();
        for field_name in self_names.difference(&other_names) {
            let mut field = (*self_map.get(*field_name).unwrap()).clone();
            let mut proto = String::new();
            field.emit_proto(&mut proto, 0);
            new_fields.retain(|f| f.number != field.number);
            match rem_beh {
                AbsentFieldAction::Deprecate => {
                    if field.deprecated {
                        print!("Keeping deprecated: {}", proto);
                    }
                    else {
                        print!("Deprecating: {}", proto);
                        field.deprecated = true;
                    }
                    new_fields.push(field);
                },
                AbsentFieldAction::Reserve => {
                    print!("Removing and reserving: {}", proto);
                    reserved.push(field.number..field.number + 1);
                },
                AbsentFieldAction::Remove => {
                    print!("Removing: {}", proto);
                }
            }
        }

        // extend self with new fields
        let mut number = last_number?;
        for field_name in other_names.difference(&self_names) {
            let mut field = (*other_map.get(*field_name).unwrap()).clone();
            let mut proto = String::new();
            field.emit_proto(&mut proto, 0);
            print!("Added: {}", proto);
            number += 1;
            field.number = number as i32;
            new_fields.push(field);
        }
        self.fields = new_fields;

        // update fields
        self.fields.extend(other.fields.clone());
        self.fields.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        self.fields.dedup_by(|a, b| {
            if a.name == b.name {
                ProtoExtender::extend(b, a, rem_beh, Some(number));
                true
            } else {
                false
            }
        });
        self.fields.sort_unstable_by(|a, b| a.number.cmp(&b.number));

        Some((number, reserved))
    }
}

impl ProtoExtender for Field {
    fn extend(&mut self, other: &Self, _rem_beh: AbsentFieldAction, _last_number: Option<usize>) -> Ret {
        if self != other {
            if self.number == other.number && self.name == other.name && self.deprecated && !other.deprecated {
                self.deprecated = false;
                let mut self_proto = String::new();
                self.emit_proto(&mut self_proto, 0);
                print!("Un-depracating: {}", self_proto);
            }
            else {
                let mut self_proto = String::new();
                self.emit_proto(&mut self_proto, 0);
                let mut other_proto = String::new();
                other.emit_proto(&mut other_proto, 0);
                println!("Ignoring difference in fields");
                print!("Old: {}New: {}", self_proto, other_proto);
            }
        }
        None
    }
}
