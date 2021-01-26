extern crate assetman;
extern crate dotenv;
#[macro_use]
extern crate prettytable;
extern crate structopt;

use assetman::AssetsCfg;
use std::collections::btree_set::BTreeSet;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "assets", about = "manage assets and track their price")]
struct Options {
    #[structopt(subcommand)]
    command: Commands,
}

#[derive(StructOpt)]
enum Commands {
    #[structopt(name = "add", about = "add a new asset that should be tracked")]
    Add {
        name: String,
        price_query: String,
        holdings_query: String,
        category: String,
    },
    #[structopt(name = "fetch", about = "fetch new prices and holdings for all assets")]
    Fetch,
    #[structopt(name = "list", about = "list all assets and their price")]
    List {
        #[structopt(
            short = "v",
            long = "sort-by-value",
            help = "sorts the table by the value of the assets"
        )]
        order_by_value: bool,
        #[structopt(
            short = "c",
            long = "group-by-category",
            help = "show assets grouped by category"
        )]
        group_by_category: bool,
    },
}

fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let options = Options::from_args();

    let mut assets = assetman::Assets::new(AssetsCfg::from_env().unwrap()).unwrap();

    match options.command {
        Commands::Add {
            name,
            price_query,
            holdings_query,
            category,
        } => {
            assets
                .add_asset(&name, &category, &price_query, &holdings_query)
                .expect("Error: Couldn't add asset.");
        }
        Commands::Fetch => {
            assets
                .fetch_data()
                .expect("Error: could not update prices and holdings.");
        }
        Commands::List {
            order_by_value,
            group_by_category,
        } => {
            let mut asset_list = assets
                .list_assets()
                .unwrap_or_else(|_| {
                    println!("No assets in database yet or no data was fetched yet, add asssets or fetch prices.");
                    exit(0);
                })
                .assets;

            if order_by_value {
                asset_list.sort_by(|a, b| {
                    (b.price * b.holdings)
                        .partial_cmp(&(a.price * a.holdings))
                        .expect("values shouldn't be NaN or inf")
                });
            }

            let sum: f64 = asset_list
                .iter()
                .map(|asset| asset.price * asset.holdings)
                .sum();

            let mut table = prettytable::Table::new();

            if group_by_category {
                table.set_titles(row!["Asset", "Holdings", "Price", "Value", "Rel"]);
                let asset_categories = asset_list
                    .iter()
                    .map(|asset| asset.category.as_str())
                    .collect::<BTreeSet<_>>();

                for asset_category in asset_categories {
                    table.add_empty_row();
                    table.add_row(row!(bFy -> asset_category, "", "", "", ""));

                    let assets = asset_list
                        .iter()
                        .filter(|&asset| asset.category == asset_category)
                        .collect::<Vec<_>>();

                    let cat_sum: f64 = assets
                        .iter()
                        .map(|asset| asset.price * asset.holdings)
                        .sum();

                    for asset in assets {
                        table.add_row(row![
                            asset.name,
                            r -> format_money(asset.holdings),
                            r -> format_money(asset.price),
                            r -> format_money(asset.holdings * asset.price),
                            r -> ""
                        ]);
                    }

                    table.add_row(row!(b -> "Category Sum", "", "", br -> format_money(cat_sum), format!("{:.1}%", cat_sum / sum * 100.0)));
                }
            } else {
                table.set_titles(row!["Asset", "Holdings", "Price", "Value"]);
                for asset in asset_list {
                    table.add_row(row![
                        asset.name,
                        r -> format_money(asset.holdings),
                        r -> format_money(asset.price),
                        r -> format_money(asset.holdings * asset.price),
                    ]);
                }
            }

            table.add_empty_row();
            table.add_row(row!(b -> "Sum", "", "", br -> format_money(sum)));

            table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
            table.printstd();
        }
    }
}

fn format_money(amount: f64) -> String {
    let mut base = format!("{:.2}", amount);
    if base.len() > 6 {
        base.insert(base.len() - 6, '\'');
    }

    base
}
