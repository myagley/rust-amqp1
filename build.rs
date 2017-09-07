extern crate handlebars;
#[macro_use]
extern crate lazy_static;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod codegen;

use std::env;
use handlebars::{Handlebars, RenderError, RenderContext, Helper};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};


fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is not defined");
    let template = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/codegen/definitions.rs"));
    let spec = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/codegen/specification.json"
    ));

    let definitions = codegen::parse(spec);

    let mut codegen = Handlebars::new();
    codegen.register_helper("snake", Box::new(snake_helper));
    codegen
        .register_template_string("definitions", template.to_string())
        .expect("Failed to register template.");
    let mut data = std::collections::BTreeMap::new();

    data.insert("defs", definitions);
    let def_path = std::path::Path::new(&out_dir).join("definitions.rs");
    {
    let mut f = File::create(def_path.clone()).expect("Failed to create target file.");
    let rendered = codegen.render("definitions", &data).expect("Failed to render template.");
    writeln!(f, "{}", rendered).expect("Failed to write to file.");
    }

    reformat_file(&def_path);
}

pub fn snake_helper (h: &Helper, _: &Handlebars, rc: &mut RenderContext) -> Result<(), RenderError> {
    let value = h.param(0).ok_or_else(|| RenderError::new("Param not found for helper \"snake\""))?;
    let param = value.value().as_str().ok_or_else(|| RenderError::new("Non-string param given to helper \"snake\""))?;
    rc.writer.write_all(codegen::snake_case(param).as_bytes())?;
    Ok(())
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
    f.seek(std::io::SeekFrom::Start(0)).unwrap();
    f.set_len(data.len() as u64).unwrap();
    f.write_all(data.as_bytes()).expect("Error writing reformatted file");
}