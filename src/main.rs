use std::fs;
use std::fs::{ read_to_string};
use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand};
use clipboard::{ClipboardContext, ClipboardProvider};
use rust_embed::Embed;
use serde::Deserialize;
use tera::{Context, Tera};
use toml::from_str;
use error::CodingGameError;
use game_session::{create_game_session, get_session_data};
use language::{Language, LanguageAsset};

mod error;
mod game_session;
mod language;

#[derive(Parser)]
#[command(name = "app", about = "Application description")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        path: String,
    },
    Pack{
        path: Option<String>,
    }
}

async fn cmd_new(lang: Language, name : String) -> Result<(), CodingGameError>{
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

    let mut context = Context::from_serialize(&game_data)?;

    context.insert("project_language", &lang);

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

fn cmd_clear(path : String) -> Result<(), CodingGameError> {
    let path = Path::new(&path);

    let _ = codingame_configuration(path)?;

    Ok(fs::remove_dir_all(path)?)
}




fn cmd_pack(path : Option<String>) -> Result<(), CodingGameError> {
    let path = path.unwrap_or(".".to_string());
    let path = Path::new(&path);

    let config = codingame_configuration(path)?;


    let data = config.language.pack(path, &config)?;


    let ctx_result : Result<ClipboardContext, _> = ClipboardProvider::new();


    match ctx_result {
        Ok(mut ctx) => {
            ctx.set_contents(data)?;
            println!("Source code merged into a single file and copied to clipboard !");
        },
        _ => println!("{data}"),
    };

    Ok(())
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CodinGameConfig {
    name : String,
    language : Language
}
fn codingame_configuration(path: &Path) -> Result<CodinGameConfig, CodingGameError> {
    let mut path_buf = PathBuf::from(path);
    path_buf.push("CodingGame.toml");
    if ! path_buf.exists() {
        Err(CodingGameError::NotACodingGamePuzzle(path.to_string_lossy().to_string()))
    } else {
        let toml_content = read_to_string(path_buf)?;
        Ok(from_str(&toml_content)?)
    }
}


#[derive(Embed)]
#[folder = "tpl/rust/"]
struct RustAsset;


#[tokio::main]
async fn main() -> Result<(), CodingGameError>{
    let cli = Cli::parse();

    match cli.command {
        Commands::New { language, name } => {
            cmd_new(language, name).await?;
        }
        Commands::Clear { path } => {
            cmd_clear(path)?;
        }
        Commands::Pack { path } => {
            cmd_pack(path)?;
        }
    }
    Ok(())
}

