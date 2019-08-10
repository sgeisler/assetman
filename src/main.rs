extern crate assetman;
extern crate dotenv;
#[macro_use] extern crate prettytable;
extern crate structopt;

use assetman::Error;
use structopt::StructOpt;
use std::collections::btree_set::BTreeSet;

#[derive(StructOpt)]
#[structopt(name = "assets", about = "manage assets and track their price")]
struct Options {
    #[structopt(subcommand)]
    command: Commands,
    #[structopt(short = "d", long = "database", help = "path to database (env: ASSET_DATABASE)")]
    db_path: Option<String>,
    #[structopt(short = "a", long = "api-key", help = "Alpha Vantage API token to query price data (env: ALPHA_VANTAGE_KEY)")]
    api_key: Option<String>,
}

#[derive(StructOpt)]
enum Commands {
    #[structopt(name = "add", about = "add a new asset that should be tracked")]
    Add {
        name: String,
        query: String,
        description: Option<String>,
        #[structopt(short = "c", long = "category", help = "sets the asset's category")]
        category: Option<String>,
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
        order_by_value: bool,
        #[structopt(short = "c", long = "group-by-category", help = "show assets grouped by category")]
        group_by_category: bool,
    },
    #[structopt(name = "plot", about = "plots the historical value development")]
    Plot,
}

fn main() {
    dotenv::dotenv().ok();
    let options = Options::from_args();

    let db_path = std::env::var("ASSET_DATABASE")
        .ok()
        .or(options.db_path)
        .expect("No database is set!");
    let av_api_key = std::env::var("ALPHA_VANTAGE_KEY")
        .ok()
        .or(options.api_key)
        .expect("No Alpha Vantage API key is set!");

    let assets = assetman::Assets::new(&db_path, &av_api_key)
        .expect("Could not open database.");


    match options.command {
        Commands::Add {
            name,
            query,
            description,
            category,
        } => {
            let description_ref = &description;
            assets.add_asset(
                &name,
                description_ref.as_ref().map(|s| s.as_str()),
                &query,
                category.as_ref().map(String::as_str)
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
            group_by_category
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

            if group_by_category {
                let asset_types = asset_list.iter()
                    .map(|(_, _, _, _, t)| t.as_str())
                    .collect::<BTreeSet<_>>();

                for asset_type in asset_types {
                    table.add_empty_row();
                    table.add_row(row!(bFy -> asset_type, "", "", ""));

                    let assets = asset_list.iter()
                        .filter(|(_, _, _, _, t)| t == asset_type)
                        .collect::<Vec<_>>();

                    let cat_sum: f32 = assets.iter()
                        .map(|(_, _, price, amount, _)| price * amount)
                        .sum();

                    for (_, name, price, amount, _) in assets {
                        table.add_row(row![
                            name,
                            r -> format_money(*amount),
                            r -> format_money(*price),
                            r -> format_money((*amount) * (*price)),
                        ]);
                    }

                    table.add_row(row!(b -> "Category Sum", "", "", br -> format_money(cat_sum)));
                }
            } else {
                for (_, name, price, amount, _) in asset_list {
                    table.add_row(row![
                        name,
                        r -> format_money(amount),
                        r -> format_money(price),
                        r -> format_money(amount * price),
                    ]);
                }
            }

            table.add_empty_row();
            table.add_row(row!(b -> "Sum", "", "", br -> format_money(sum)));

            table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            table.printstd();
        },
        Commands::Plot => {
            assets.plot();
        }
    }
}

fn format_money(amount: f32) -> String {
    let mut base = format!("{:.2}", amount);
    if base.len() > 6 {
        base.insert(base.len() - 6, '\'');
    }

    base
}
