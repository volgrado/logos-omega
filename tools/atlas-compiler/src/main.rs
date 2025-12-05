use clap::Parser;
use std::fs;
use std::path::PathBuf;
use logos_protocol::{Dictionary};
use rkyv::ser::{serializers::AllocSerializer, Serializer};

#[derive(Parser)]
#[command(author, version, about = "Compiles JSON dictionary to rkyv binary")]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    #[arg(short, long, value_name = "FILE")]
    output: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    println!("📖 Reading JSON from {:?}...", cli.input);
    let input_data = fs::read_to_string(&cli.input)?;

    // 2. Deserialize JSON to Rust Structs
    // Ensure logos-protocol types derive Deserialize (from serde)
    let dict: Dictionary = serde_json::from_str(&input_data)?;

    println!("⚙️  Compiling Dictionary version {} with {} lemmas...", dict.version, dict.lemmas.len());

    // 3. Serialize to RKYV
    let mut serializer = AllocSerializer::<256>::default();
    serializer.serialize_value(&dict).expect("Failed to rkyv serialize");
    let bytes = serializer.into_serializer().into_inner();

    // 4. Write Binary
    fs::write(&cli.output, bytes)?;

    println!("✅ Success! Binary written to {:?}", cli.output);
    Ok(())
}
