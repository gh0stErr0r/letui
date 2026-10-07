use std::{env, fs, path::PathBuf};

fn rust_string(value: &str) -> String {
    format!("{:?}", value)
}

fn main() {
    println!("cargo:rerun-if-changed=src/abbreviations.json");

    let source =
        fs::read_to_string("src/abbreviations.json").expect("failed to read abbreviations.json");
    let abbreviations: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&source).expect("invalid abbreviations.json");

    let mut generated = String::from(
        "#[derive(Clone, Copy)]\npub struct Abbreviation {\n    pub key: &'static str,\n    pub value: &'static str,\n}\n\npub static ABBREVIATIONS: &[Abbreviation] = &[\n",
    );

    for (key, value) in abbreviations {
        let value = value.as_str().expect("abbreviation values must be strings");
        generated.push_str(&format!(
            "    Abbreviation {{ key: {}, value: {} }},\n",
            rust_string(&key),
            rust_string(value)
        ));
    }
    generated.push_str("];\n");

    let output =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is not set")).join("abbreviations.rs");
    fs::write(output, generated).expect("failed to write generated abbreviations");
}
