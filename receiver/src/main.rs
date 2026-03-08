use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "receiver", about = "Fetch offers from the market and download data from a sender")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List available offers from the market
    List {
        #[arg(long)]
        market: String,
    },

    /// Download an item by name (first matching offer)
    Download {
        #[arg(long)]
        market: String,

        #[arg(long)]
        item: String,

        #[arg(long, default_value = "downloads")]
        out: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = run(cli).await {
        eprintln!("receiver failed: {}", err);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), receiver::ReceiverError> {
    match cli.command {
        Command::List { market } => {
            let offers = receiver::list_offers(&market).await?;
            for offer in offers {
                println!("{} {} {} bytes", offer.item, offer.address, offer.item_size);
            }
            Ok(())
        }
        Command::Download { market, item, out } => {
            let offers = receiver::list_offers(&market).await?;
            let offer = offers
                .into_iter()
                .find(|offer| offer.item == item)
                .ok_or_else(|| receiver::ReceiverError::new("offer not found"))?;

            let (_metadata, files) = receiver::download_offer(&offer, &out).await?;
            println!("downloaded {} files into {}", files.len(), out.display());
            Ok(())
        }
    }
}
