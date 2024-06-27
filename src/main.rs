use std::fs;
use std::marker::PhantomData;
use std::path::Path;
use clap::{Parser, Subcommand, ValueEnum};
use rust_embed::{Embed, Filenames};
use tera::{Context, Tera};
use error::CodingGameError;
use game_session::{create_game_session, get_session_data};

mod error;
mod game_session;

#[derive(Parser)]
#[command(name = "app", about = "Application description")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}


#[derive(Default, Debug, Clone, ValueEnum)]
enum Language {
    #[default]
    Rust
}

trait LanguageAsset {
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
    fn asset(&self) -> impl LanguageAsset {
        match self {
            Language::Rust => AssetWrapper::<RustAsset>::new(),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    New {
        #[arg(required = true)]
        name: String,
        #[arg(required = true)]
        language: Language,
    },
    Clear {
        #[arg(required = true)]
        name: String,
    }
}

async fn new(lang: Language, name : String) -> Result<(), CodingGameError>{
    let session_id = create_game_session(name.clone()).await?;

    let mut game_data = get_session_data(session_id).await?;

    game_data.set_safe_name(&name);

    let puzzle_folder = Path::new(&game_data.safe_name);

    if puzzle_folder.exists() {
        return Err(CodingGameError::PuzzleAlreadyExists(game_data.safe_name));
    }

    fs::create_dir(puzzle_folder)?;

    let mut tera = Tera::default();

    let asset_folder = lang.asset();

    for file in asset_folder.iter() {
        let asset = RustAsset::get(file.as_ref()).ok_or(CodingGameError::AssetError(file.as_ref().to_string()))?;
        tera.add_raw_template(file.as_ref(), std::str::from_utf8(asset.data.as_ref())?)?;
    }

    let context = Context::from_serialize(&game_data)?;

    for template_name in tera.templates.keys() {
        let rendered_template = tera.render(template_name, &context)?;

        let rendered_path = format!("{}/{}", puzzle_folder.to_str().unwrap(), template_name);

        let rendered_path = Path::new(&rendered_path);

        fs::create_dir_all(rendered_path.parent().unwrap_or(Path::new(".")))?;


        fs::write(rendered_path, rendered_template)?;

        println!("Template {} rendered and saved.", template_name);
    }

    for test in game_data.tests {
        let test_path_str = format!("{}/tests/data/{}", puzzle_folder.to_str().unwrap(), test.safe_label);
        let test_folder_path = Path::new(&test_path_str);

        fs::create_dir_all(test_folder_path)?;

        let input_path = format!("{test_path_str}/input.txt");
        let input_path = Path::new(&input_path);
        fs::write(input_path, test.input_text)?;

        let output_path = format!("{test_path_str}/output.txt");
        let output_path = Path::new(&output_path);
        fs::write(output_path, test.output_text)?;

        println!("Test {} downloaded.", test.label);
    }

    Ok(())
}

fn clear(name: String) -> Result<(), CodingGameError> {
    let path = Path::new(&name);

    if path.is_dir() {
        fs::remove_dir_all(path)?;
    }

    Ok(())
}



#[derive(Embed)]
#[folder = "tpl/rust/"]
struct RustAsset;


#[tokio::main]
async fn main() -> Result<(), CodingGameError>{
    let cli = Cli::parse();



    match cli.command {
        Commands::New { language, name } => {
            new(language, name).await?;
        }
        Commands::Clear { name } => {
            clear(name)?;
        }
    }
    Ok(())
}

