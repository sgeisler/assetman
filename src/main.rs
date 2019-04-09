extern crate assetman;
extern crate dotenv;
#[macro_use] extern crate prettytable;
extern crate structopt;

use assetman::Error;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "assets", about = "manage assets and track their price")]
struct Options {
    #[structopt(subcommand)]
    command: Commands,
    #[structopt(short = "d", long = "database", help = "path to database (env: ASSET_DATABASE)")]
    db_path: Option<String>,
    #[structopt(short = "q", long = "quandl-token", help = "quandl token to query price data (env: ASSET_QUANDL_TOKEN)")]
    quandl_token: Option<String>,
}

#[derive(StructOpt)]
enum Commands {
    #[structopt(name = "add", about = "add a new asset that should be tracked")]
    Add {
        name: String,
        quandl_database: String,
        quandl_dataset: String,
        quandl_price_idx: u16,
        description: Option<String>,
    },
    #[structopt(name = "update", about = "update the amount of an asset")]
    Update {
        asset_name: String,
        new_holdings: f32,
    },
    #[structopt(name = "fetch", about = "fetch new prices for all assets")]
    Fetch,
    #[structopt(name = "list", about = "list all assets and their price")]
    List {
        #[structopt(short = "v", long = "sort-by-value", help = "sorts the table by the value of the assets")]
        order_by_value: bool
    },
}

fn main() {
    dotenv::dotenv().ok();
    let options = Options::from_args();

    let db_path = std::env::var("ASSET_DATABASE")
        .ok()
        .or(options.db_path)
        .expect("No database is set!");
    let quandl_token = std::env::var("ASSET_QUANDL_TOKEN")
        .ok()
        .or(options.quandl_token)
        .expect("No quandl API token is set!");

    let assets = assetman::Assets::new(&db_path, &quandl_token)
        .expect("Could not open database.");


    match options.command {
        Commands::Add {
            name,
            quandl_database,
            quandl_dataset,
            quandl_price_idx,
            description,
        } => {
            let description_ref = &description;
            assets.add_asset(
                &name,
                description_ref.as_ref().map(|s| s.as_str()),
                &quandl_database,
                &quandl_dataset,
                quandl_price_idx
            ).expect("Error: Couldn't add asset.");
        },
        Commands::Update {
            asset_name,
            new_holdings,
        } => {
            let asset = match assets.asset(&asset_name) {
                Ok(asset) => asset,
                Err(Error::AssetNotFound) => {
                    eprintln!("Asset not found: {}", asset_name);
                    std::process::exit(1);
                },
                Err(e) => {
                    panic!("Error: {:?}", e);
                }
            };

            asset.update_holdings(new_holdings).expect("Error: could not update asset.");
        },
        Commands::Fetch => {
            assets.update_prices().expect("Error: could not update prices.");
        },
        Commands::List {
            order_by_value,
        } => {
            let mut asset_list = assets.list_assets()
                .expect("Error: could not list assets.");

            if order_by_value {
                asset_list.sort_by(|a, b|
                    (b.2 * b.3).partial_cmp(&(a.2 * a.3)).expect("values shouldn't be NaN or inf")
                );
            }

            let sum: f32 = asset_list.iter()
                .map(|asset| asset.2 * asset.3)
                .sum();

            let mut table = prettytable::Table::new();
            table.set_titles(row!["Asset", "Holdings", "Price", "Value"]);
            for (_, name, price, amount) in asset_list {
                table.add_row(row![
                    name,
                    r -> format_money(amount),
                    r -> format_money(price),
                    r -> format_money(amount * price),
                ]);
            }

            table.add_empty_row();
            table.add_row(row!("Sum", "", "", r -> format_money(sum)));

            table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            table.printstd();
        },
    }
}

fn format_money(amount: f32) -> String {
    let mut base = format!("{:.2}", amount);
    if base.len() > 6 {
        base.insert(base.len() - 6, '\'');
    }

    base
}
