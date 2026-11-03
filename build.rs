// Embeds every lesson file and the friendly error catalog into the binary,
// so `cargo install --path .` produces a self-contained program.
use std::{env, fs, path::Path};

fn main() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let lessons_dir = Path::new(&root).join("lessons");
    println!("cargo:rerun-if-changed=lessons");
    println!("cargo:rerun-if-changed=data/errors.toml");

    let mut files: Vec<_> = fs::read_dir(&lessons_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "toml"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();

    let mut out = String::from("pub static LESSON_FILES: &[(&str, &str)] = &[\n");
    for f in &files {
        println!("cargo:rerun-if-changed={}", f.display());
        let name = f.file_name().unwrap().to_string_lossy();
        out.push_str(&format!(
            "    ({name:?}, include_str!({:?})),\n",
            f.display().to_string()
        ));
    }
    out.push_str("];\n");

    let errors = Path::new(&root).join("data").join("errors.toml");
    if errors.exists() {
        out.push_str(&format!(
            "pub static ERROR_CATALOG: &str = include_str!({:?});\n",
            errors.display().to_string()
        ));
    } else {
        out.push_str("pub static ERROR_CATALOG: &str = \"\";\n");
    }
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("embedded.rs"),
        out,
    )
    .unwrap();
}
