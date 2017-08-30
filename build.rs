extern crate handlebars;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
use serde_json::from_str;
use handlebars::{to_json, Handlebars, Helper, RenderContext, RenderError, Renderable};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::ascii::AsciiExt;
use std::sync::Mutex;

lazy_static! {
    static ref PRIMITIVE_TYPES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("binary", "Bytes");
        m.insert("string", "ByteStr");
        m.insert("ubyte", "u8");
        m.insert("ushort", "u16");
        m.insert("uint", "u32");
        m.insert("ulong", "u64");
        m.insert("boolean", "bool");
        m
    };

    static ref STRING_TYPES: HashSet<&'static str> = ["string", "symbol"].iter().cloned().collect();
    static ref REF_TYPES: Mutex<HashSet<String>> = Mutex::new([
        "Bytes", "ByteStr", "Symbol", "Fields", "Map",
        "MessageId", "Address", "NodeProperties",
        "Outcome", "DeliveryState", "FilterSet", "DeliveryTag",
        "Symbols", "IetfLanguageTags"]
        .iter().map(|s| s.to_string()).collect());
    static ref ENUM_TYPES: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum _Type {
    Choice(_Enum),
    Described(_Described),
    Alias(Alias),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct _Descriptor {
    name: String,
    code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct _Enum {
    name: String,
    source: String,
    provides: Option<String>,
    choice: Vec<EnumItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct _Described {
    name: String,
    class: String,
    source: String,
    provides: Option<String>,
    descriptor: _Descriptor,
    #[serde(default)] field: Vec<_Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct _Field {
    name: String,
    #[serde(rename = "type")] ty: String,
    #[serde(default)]
    #[serde(deserialize_with = "string_as_bool")]
    mandatory: bool,
    default: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "string_as_bool")]
    multiple: bool,
    requires: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Alias {
    name: String,
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EnumItem {
    name: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Definitions {
    aliases: Vec<Alias>,
    enums: Vec<Enum>,
    lists: Vec<DescribedList>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Enum {
    name: String,
    ty: String,
    provides: Vec<String>,
    items: Vec<EnumItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DescribedList {
    name: String,
    provides: Vec<String>,
    descriptor: Descriptor,
    fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Descriptor {
    name: String,
    domain: u32,
    code: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Field {
    name: String,
    ty: String,
    is_str: bool,
    is_ref: bool,
    optional: bool,
    default: String,
    multiple: bool,
}

impl Definitions {
    fn from(types: Vec<_Type>) -> Definitions {
        let mut aliases = vec![];
        let mut enums = vec![];
        let mut lists = vec![];
        for t in types.into_iter() {
            match t {
                _Type::Alias(ref a) if a.source != "map" => aliases.push(Alias {
                    name: camel_case(&*a.name),
                    source: get_type_name(&*a.source, None),
                }),
                _Type::Choice(ref e) => enums.push(Enum::from(e.clone())),
                _Type::Described(ref l) if l.source == "list" => lists.push(DescribedList::from(l.clone())),
                _ => {}
            }
        }
        Definitions {
            aliases,
            enums,
            lists,
        }
    }
}

impl Enum {
    fn from(e: _Enum) -> Enum {
        Enum {
            name: camel_case(&*e.name),
            ty: get_type_name(&*e.source, None),
            provides: parse_provides(e.provides),
            items: e.choice
                .into_iter()
                .map(|c| {
                    EnumItem {
                        name: camel_case(&*c.name),
                        value: c.value,
                    }
                })
                .collect(),
        }
    }
}

impl DescribedList {
    fn from(d: _Described) -> DescribedList {
        DescribedList {
            name: camel_case(&d.name),
            provides: parse_provides(d.provides),
            descriptor: Descriptor::from(d.descriptor),
            fields: d.field.into_iter().map(|f| Field::from(f)).collect(),
        }
    }
}

impl Descriptor {
    fn from(d: _Descriptor) -> Descriptor {
        let code_parts: Vec<u32> = d.code
            .split(":")
            .map(|p| {
                assert!(p.starts_with("0x"));
                u32::from_str_radix(&p[2..], 16).expect("malformed descriptor code")
            })
            .collect();
        Descriptor {
            name: d.name,
            domain: code_parts[0],
            code: code_parts[1],
        }
    }
}
impl Field {
    fn from(field: _Field) -> Field {
        let mut ty = get_type_name(&*field.ty, field.requires);
        if field.multiple {
            ty.push('s');
        }
        let is_str = STRING_TYPES.contains(&*field.ty) && !field.multiple;
        let is_ref = REF_TYPES.lock().unwrap().contains(&ty);
        let default = Field::format_default(field.default, &ty);
        Field {
            name: snake_case(&*field.name),
            ty: ty,
            is_ref,
            is_str,
            optional: !field.mandatory,
            multiple: field.multiple,
            default,
        }
    }

    fn format_default(default: Option<String>, ty: &str) -> String {
        match default {
            None => String::new(),
            Some(def) => if ENUM_TYPES.lock().unwrap().contains(ty) {
                format!("{}::{}", ty, camel_case(&*def))
            } else {
                def
            },
        }
    }
}

fn get_type_name(ty: &str, req: Option<String>) -> String {
    match PRIMITIVE_TYPES.get(ty) {
        Some(p) => p.to_string(),
        None => if ty == "*" {
            camel_case(&*req.expect("Encountered * type without requires."))
        } else {
            camel_case(&*ty)
        },
    }
}

fn get_option_type(ty: &String, optional: bool) -> String {
    if optional {
        format!("Option<{}>", ty)
    } else {
        ty.clone()
    }
}

fn parse_provides(p: Option<String>) -> Vec<String> {
    p.map(|v| {
        v.split_terminator(",")
            .filter_map(|s| {
                let s = s.trim();
                if s == "" {
                    None
                } else {
                    Some(camel_case(&s))
                }
            })
            .collect()
    }).unwrap_or(vec![])
}

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is not defined");
    let template = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/build/definitions.rs"));
    let spec = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/build/specification.json"
    ));
    let types = from_str::<Vec<_Type>>(spec).expect("Failed to parse AMQP spec.");

    {
        let mut ref_map = REF_TYPES.lock().unwrap();
        let mut enum_map = ENUM_TYPES.lock().unwrap();
        for t in types.iter() {
            match *t {
                _Type::Described(ref l) if l.source == "list" => {
                    ref_map.insert(camel_case(&*l.name));
                }
                _Type::Choice(ref e) => {
                    enum_map.insert(camel_case(&*e.name));
                }
                _ => {}
            }
        }
    }

    let definitions = Definitions::from(types);

    let mut codegen = Handlebars::new();
    codegen
        .register_template_string("definitions", template.to_string())
        .expect("Failed to register template.");
    codegen.register_helper("camel", Box::new(camel_helper));
    codegen.register_helper("snake", Box::new(snake_helper));
    let mut data = std::collections::BTreeMap::new();

    //let ref_types =
    data.insert("defs", definitions);
    let def_path = std::path::Path::new(&out_dir).join("definitions.rs");
    let mut f = File::create(def_path.clone()).expect("Failed to create target file.");
    writeln!(
        f,
        "{}",
        codegen
            .render("definitions", &data)
            .expect("Failed to render template.")
    ).expect("Failed to write to file.");
    drop(f);

    reformat_file(&def_path);
}

fn reformat_file(path: &std::path::Path) {
    std::process::Command::new("rustfmt")
        .arg(path.to_str().unwrap())
        .output()
        .expect("failed to format definitions.rs");

    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .expect("failed to open file");
    let mut data = String::new();
    f.read_to_string(&mut data).expect("failed to read file.");
    data = data.replace("\n\n", "\n")
        .replace("\n\n", "\n")
        .replace("\n\n", "\n");
    f.seek(std::io::SeekFrom::Start(0));
    f.set_len(data.len() as u64);
    f.write_all(data.as_bytes()).expect("abc");
}

fn string_as_bool<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: std::str::FromStr<Err = std::str::ParseBoolError>,
    D: serde::Deserializer<'de>,
{
    Ok(
        String::deserialize(deserializer)?
            .parse::<T>()
            .expect("Error parsing bool from string"),
    )
}

pub fn camel_helper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let value = h.param(0).expect("Param not found for helper \"camel\"");
    let param = value
        .value()
        .as_str()
        .expect("Non-string param given to helper \"camel\"");
    rc.writer.write_all(camel_case(param).as_bytes())?;
    Ok(())
}

pub fn snake_helper(h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let value = h.param(0).expect("Param not found for helper \"snake\"");
    let param = value
        .value()
        .as_str()
        .expect("Non-string param given to helper \"snake\"");
    rc.writer.write_all(snake_case(param).as_bytes())?;
    Ok(())
}

pub fn camel_case(name: &str) -> String {
    let mut new_word = true;
    name.chars().fold("".to_string(), |mut result, ch| {
        if ch == '-' || ch == '_' || ch == ' ' {
            new_word = true;
            result
        } else {
            result.push(if new_word { ch.to_ascii_uppercase() } else { ch });
            new_word = false;
            result
        }
    })
}

pub fn snake_case(name: &str) -> String {
    match name {
        "type" => "type_".to_string(),
        "return" => "return_".to_string(),
        name => {
            let mut new_word = false;
            let mut last_was_upper = false;
            name.chars().fold("".to_string(), |mut result, ch| {
                if ch == '-' || ch == '_' || ch == ' ' {
                    new_word = true;
                    result
                } else {
                    let uppercase = ch.is_uppercase();
                    if new_word || (!last_was_upper && !result.is_empty() && uppercase) {
                        result.push('_');
                        new_word = false;
                    }
                    last_was_upper = uppercase;
                    result.push(if uppercase { ch.to_ascii_lowercase() } else { ch });
                    result
                }
            })
        }
    }
}
