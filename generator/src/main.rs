use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use heck::ToSnakeCase;
use kdl::KdlDocument;

#[rustfmt::skip]
fn main() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    let data_file = File::open(manifest_dir.join("keyboard_codes.kdl"))?;
    let out_file = File::create(manifest_dir.parent().unwrap().join("src/keyboard.rs"))?;
    let mut out = BufWriter::new(out_file);
    let mut document_string = String::new();
    BufReader::new(data_file).read_to_string(&mut document_string)?;

    let document: KdlDocument = document_string.parse()?;
    
    let keys: Vec<(i64, &str)> = document
        .nodes()
        .iter()
        .map(|node| {
            (
                node.entries()[0].value().as_i64().unwrap(),
                node.entries()[1].value().as_string().unwrap(),
            )
        })
        .collect();

    writeln!(out, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]")?;
    writeln!(out, "pub enum HIDScanCode {{")?;
    for (code, name) in &keys {
        writeln!(out, "    {} = {:#04x?},", name, code)?;
    }
    writeln!(out, "}}\n")?;

    writeln!(out, "impl HIDScanCode {{")?;
    writeln!(out, "    pub fn from_i32(int: i32) -> Option<Self> {{")?;
    writeln!(out, "        match int {{")?;
    for (code, name) in &keys {
        writeln!(out, "            {:#04x?} => Some(Self::{}),", code, name)?;
    }
    writeln!(out, "            _ => None")?;
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}\n")?;

    writeln!(out, "use crate::{{SuPath, driver_interface::DriverRuntimeInterface}};")?;
    writeln!(out, "pub struct KeyboardPaths {{")?;
    for (_, name) in &keys {
        let mut name_sc = name.to_snake_case();
        if name_sc == "return" {
            name_sc = "r#return".to_string();
        }

        writeln!(out, "    pub {}: SuPath,", name_sc)?;
    }
    writeln!(out, "}}\n")?;

    writeln!(out, "impl KeyboardPaths {{")?;
    writeln!(out,"    pub fn new(driver_manager: &dyn DriverRuntimeInterface) -> Self {{")?;
    writeln!(out, "        Self {{")?;
    for (_, name) in &keys {
        let name_sc = name.to_snake_case();
        let name_sc_sanitized = if name_sc == "return" {
            "r#return"
        } else {
            &name_sc
        };

        writeln!(out,"            {name_sc_sanitized}: driver_manager.get_path(\"/input/button_{name_sc}/click\").unwrap(),")?;
    }
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "\n")?;
    writeln!(out,"    pub fn get(&self, hid_scan_code: HIDScanCode) -> SuPath {{")?;
    writeln!(out, "        match hid_scan_code {{")?;
    for (_, name) in &keys {
        let name_sc = name.to_snake_case();
        let name_sc_sanitized = if name_sc == "return" {
            "r#return"
        } else {
            &name_sc
        };

        writeln!(out, "            HIDScanCode::{name} => self.{name_sc_sanitized},")?;
    }
    writeln!(out, "        }}")?;
    writeln!(out, "    }}")?;
    writeln!(out, "}}\n")?;

    Ok(())
}