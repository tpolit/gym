// The JSON is read at compile time so that the domain does not depend on serde.
// An unknown enum value produces a compiler error in the generated code.

use std::collections::BTreeSet;
use std::fmt::{self, Write as _};
use std::path::PathBuf;
use std::{env, fs, io};

use serde::Deserialize;

const CATALOG_PATH: &str = "src/catalog.json";
const GENERATED_FILE: &str = "catalog_data.rs";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    category: String,
    muscle_group: String,
    equipment: String,
    measure: String,
    #[serde(default)]
    retired: bool,
}

enum BuildError {
    Io(io::Error),
    Json(serde_json::Error),
    Format(fmt::Error),
    MissingOutDir,
    InvalidId(String),
    DuplicateId(String),
    EmptyName(String),
    InvalidEnumValue { id: String, value: String },
}

// Cargo prints the error returned by `main` with `Debug`: make it a readable message.
impl fmt::Debug for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{CATALOG_PATH}: I/O error: {error}"),
            Self::Json(error) => write!(f, "{CATALOG_PATH}: invalid JSON: {error}"),
            Self::Format(error) => write!(f, "code generation: {error}"),
            Self::MissingOutDir => write!(f, "missing OUT_DIR variable"),
            Self::InvalidId(id) => write!(f, "id is not snake_case: {id:?}"),
            Self::DuplicateId(id) => write!(f, "duplicate id: {id:?}"),
            Self::EmptyName(id) => write!(f, "empty name for {id:?}"),
            Self::InvalidEnumValue { id, value } => {
                write!(f, "invalid enum value {value:?} for {id:?}")
            }
        }
    }
}

fn main() -> Result<(), BuildError> {
    println!("cargo::rerun-if-changed={CATALOG_PATH}");

    let json = fs::read_to_string(CATALOG_PATH).map_err(BuildError::Io)?;
    let entries: Vec<Entry> = serde_json::from_str(&json).map_err(BuildError::Json)?;

    let mut ids = BTreeSet::new();
    let mut code = String::from(
        "// Généré par build.rs à partir de src/catalog.json : ne pas modifier.\n\
         const RAW_CATALOG: &[RawBuiltin] = &[\n",
    );
    for entry in &entries {
        if !is_snake_case(&entry.id) {
            return Err(BuildError::InvalidId(entry.id.clone()));
        }
        if !ids.insert(entry.id.as_str()) {
            return Err(BuildError::DuplicateId(entry.id.clone()));
        }
        if entry.name.trim().is_empty() {
            return Err(BuildError::EmptyName(entry.id.clone()));
        }
        writeln!(
            code,
            "    RawBuiltin {{ id: {:?}, name: {:?}, aliases: &{:?}, category: Category::{}, \
             muscle_group: MuscleGroup::{}, equipment: Equipment::{}, measure: MeasureKind::{}, \
             retired: {} }},",
            entry.id,
            entry.name.trim(),
            entry.aliases,
            rust_variant_name(&entry.id, &entry.category)?,
            rust_variant_name(&entry.id, &entry.muscle_group)?,
            rust_variant_name(&entry.id, &entry.equipment)?,
            rust_variant_name(&entry.id, &entry.measure)?,
            entry.retired,
        )
        .map_err(BuildError::Format)?;
    }
    code.push_str("];\n");

    let out_dir = env::var_os("OUT_DIR").ok_or(BuildError::MissingOutDir)?;
    fs::write(PathBuf::from(out_dir).join(GENERATED_FILE), code).map_err(BuildError::Io)
}

fn is_snake_case(value: &str) -> bool {
    value.starts_with(|c: char| c.is_ascii_lowercase())
        && !value.ends_with('_')
        && !value.contains("__")
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn rust_variant_name(id: &str, value: &str) -> Result<String, BuildError> {
    if !is_snake_case(value) {
        return Err(BuildError::InvalidEnumValue {
            id: id.to_owned(),
            value: value.to_owned(),
        });
    }
    Ok(value
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            chars
                .next()
                .map(|first| first.to_ascii_uppercase().to_string() + chars.as_str())
                .unwrap_or_default()
        })
        .collect())
}
