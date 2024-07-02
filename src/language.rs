use std::fs::File;
use std::io::{BufRead, BufReader};
use rust_embed::{Filenames, RustEmbed as Embed};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use crate::error::CodingGameError;
use crate::{CodinGameConfig, RustAsset};

#[derive(Default, Debug, Clone, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Rust
}

impl Language {
    pub fn pack(&self, path: &Path, config : &CodinGameConfig) -> Result<String, CodingGameError> {
        match self {
            Language::Rust => {
                let mut lib_path = PathBuf::from(&path);
                lib_path.push("src");
                lib_path.push("lib.rs");

                let mut data = self.pack_sourcecode(lib_path, Some(config.name.clone()))?;

                let mut main_path = PathBuf::from(&path);
                main_path.push("src");
                main_path.push("main.rs");

                data.push_str(&self.pack_sourcecode(main_path, None)?);

                Ok(data)
            }
        }
    }

    fn pack_sourcecode(&self, path: PathBuf, module_name : Option<String>) -> Result<String, CodingGameError> {
        match self {
            Language::Rust => {
                let mut res = String::new();

                let file_reader = BufReader::new(File::open(path)?);
                let prefix = if let Some(module) = module_name.clone() {
                    res.push_str(&format!("mod {} {{\n", module));
                    "    "
                } else {
                    ""
                };

                for line in file_reader.lines() {
                    let line = line?;
                    res.push_str(prefix);
                    res.push_str(&line);
                    res.push('\n');
                }

                if module_name.is_some() {
                    res.push_str("}\n");
                }

                Ok(res)
            }
        }
    }

}

pub trait LanguageAsset {
    fn iter(&self) -> Filenames;
}

struct AssetWrapper<T>
where T : Embed {
    _unused : PhantomData<T>
}

impl<T> AssetWrapper<T>
where T: Embed{
    fn new() -> Self {
        Self{_unused: PhantomData}
    }
}

impl<T> LanguageAsset for AssetWrapper<T>
where T: Embed {
    fn iter(&self) -> Filenames {
        T::iter()
    }
}

impl Language {
    pub fn asset(&self) -> impl LanguageAsset {
        match self {
            Language::Rust => AssetWrapper::<RustAsset>::new(),
        }
    }
}
