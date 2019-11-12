extern crate chrono;
extern crate dotenv;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
extern crate reqwest;
extern crate serde_json;
extern crate textplots;

use diesel::prelude::*;

use schema::*;

pub mod alphav;
mod schema;

embed_migrations!();

pub struct Assets {
    db_client: diesel::sqlite::SqliteConnection,
    av_client: alphav::AlphaVantageClient,
}

impl Assets {
    pub fn new(db_path: &str, av_api_key: &str) -> Result<Assets, Error> {
        let assets = Assets {
            db_client: diesel::SqliteConnection::establish(db_path)?,
            av_client: alphav::AlphaVantageClient::new(av_api_key.into()),
        };

        assets.run_migrations()?;

        Ok(assets)
    }

    fn run_migrations(&self) -> Result<(), Error> {
        embedded_migrations::run(&self.db_client)?;
        Ok(())
    }

    pub fn list_assets(&self) -> diesel::QueryResult<Vec<(i32, String, f32, f32, String)>> {
        // select
        //  assets.id
        //  assets.name,
        //  updates.amount,
        //  prices.price
        // from
        //  assets join
        //  updates on assets.id = updates.asset_id join
        //  prices on assets.id = prices.asset_id
        // where
        //  updates.timestamp=(select max(timestamp) from updates where updates.asset_id=assets.id) and
        //  prices.timestamp=(select max(timestamp) from prices where prices.asset_id=assets.id) and
        //  updates.amount != 0;
        schema::assets::table
            .inner_join(schema::updates::table)
            .inner_join(schema::prices::table)
            .select((schema::assets::id, schema::assets::name, schema::prices::price, schema::updates::holdings, schema::assets::category))
            .filter(
                schema::updates::timestamp.eq(diesel::dsl::sql::<diesel::sql_types::Timestamp>("(SELECT MAX(timestamp) FROM updates WHERE assets.id = updates.asset_id)")).and(
                    schema::prices::timestamp.eq(diesel::dsl::sql::<diesel::sql_types::Timestamp>("(SELECT MAX(timestamp) FROM prices WHERE assets.id = prices.asset_id)"))
                ).and(schema::updates::holdings.ne(0.0))
            )
            .order_by(schema::assets::name)
            .load(&self.db_client)
    }

    pub fn add_asset(&self, name: &str, description: Option<&str>, query: &str, category: Option<&str>) -> Result<Asset, Error> {
        self.db_client.transaction(|| {
            let price = self.av_client.query(query)?;

            diesel::insert_into(schema::assets::table)
                .values(NewAsset {
                    name,
                    description,
                    query,
                    category,
                }).execute(&self.db_client)?;

            let asset_id: i32 = schema::assets::table
                .order(schema::assets::id.desc())
                .select(schema::assets::id)
                .first(&self.db_client)?;

            diesel::insert_into(schema::prices::table)
                .values(NewPriceEntry {
                    asset_id,
                    price: price as f32,
                })
                .execute(&self.db_client)?;

            diesel::insert_into(schema::updates::table)
                .values(NewHoldingsEntry {
                    asset_id,
                    holdings: 0.0,
                })
                .execute(&self.db_client)?;

            Ok(Asset {
                asset_id,
                client: &self,
            })
        })
    }

    pub fn asset(&self, name: &str) -> Result<Asset, Error> {
        let asset_id: i32 = schema::assets::table
            .filter(schema::assets::name.eq(name))
            .select(schema::assets::id)
            .first(&self.db_client)
            .optional()?
            .ok_or(Error::AssetNotFound)?;

        Ok(Asset {
            asset_id,
            client: self,
        })
    }

    fn update_holdings(&self, asset_id: i32, new_holdings: f32) -> Result<(), Error> {
        diesel::insert_into(schema::updates::table)
            .values(NewHoldingsEntry {
                asset_id,
                holdings: new_holdings,
            })
            .execute(&self.db_client)?;
        Ok(())
    }

    pub fn update_prices(&self) -> Result<(), Error> {
        self.db_client.transaction(|| {
            let assets: Vec<UpdateAsset> = schema::assets::table
                .select((
                    schema::assets::id,
                    schema::assets::query,
                    schema::assets::name,
                ))
                .load(&self.db_client)?;

            let asset_count = assets.len();
            for (idx, asset) in assets.into_iter().enumerate() {
                println!("Fetching price for {:25} ({}/{})", asset.name, idx, asset_count);
                match self.av_client.query(&asset.query) {
                    Ok(price) => {
                        diesel::insert_into(schema::prices::table)
                            .values(NewPriceEntry {
                                asset_id: asset.id,
                                price: price as f32,
                            })
                            .execute(&self.db_client)?;
                    },
                    Err(e) => {
                        eprintln!("Skipping {} because of error {:?}", asset.query, e);
                    },
                }
            }

            Ok(())
        })
    }

    pub fn plot(&self) {
        use textplots::Plot;
        use terminal_size::{Width, Height, terminal_size};

        let time_series = diesel::dsl::sql::<(diesel::sql_types::Timestamp, diesel::sql_types::Float)>(
                "select ts,
                       (
                         select SUM(updates.holdings * prices.price)
                         from assets
                                join updates on assets.id = updates.asset_id
                                join prices on assets.id = prices.asset_id
                         where updates.timestamp = (select max(timestamp) from updates where updates.asset_id = assets.id and updates.timestamp <= ts)
                           and prices.timestamp = (select max(timestamp) from prices where prices.asset_id = assets.id and prices.timestamp <= ts)
                           and updates.holdings != 0
                       ) as nw
                from (select prices.timestamp as ts from prices union select updates.timestamp as ts from updates)
                where nw not null;
                "
            )
            .load(&self.db_client)
            .unwrap()
            .into_iter()
            .skip(5)
            .map(|(t, v): (chrono::NaiveDateTime, f32)| (t.timestamp() as f32, v))
            .collect::<Vec<_>>();

        let x_min = time_series
            .iter()
            .min_by(|(t1, _), (t2, _)| t1.partial_cmp(t2).unwrap())
            .unwrap()
            .0;

        let x_max = time_series
            .iter()
            .max_by(|(t1, _), (t2, _)| t1.partial_cmp(t2).unwrap())
            .unwrap()
            .0;

        println!("{:?}", terminal_size());

        let (width, height) = if let Some((Width(width), Height(height))) = terminal_size() {
            (std::cmp::max(width * 3 / 2, 32), std::cmp::max(height * 3, 32))
        } else {
            (200, 100)
        };

        println!("{}, {}", width, height);

        textplots::Chart::new(width as u32, height as u32, x_min, x_max)
            .lineplot(textplots::Shape::Lines(&time_series))
            .nice();
    }
}

pub struct Asset<'a> {
    asset_id: i32,
    client: &'a Assets,
}

impl<'a> Asset<'a> {
    pub fn update_holdings(&self, new_holdings: f32) -> Result<(), Error> {
        self.client.update_holdings(self.asset_id, new_holdings)
    }
}

#[derive(Insertable)]
#[table_name = "assets"]
struct NewAsset<'a> {
    name: &'a str,
    description: Option<&'a str>,
    query: &'a str,
    category: Option<&'a str>,
}

#[derive(Insertable)]
#[table_name = "prices"]
struct NewPriceEntry {
    asset_id: i32,
    price: f32,
}

#[derive(Insertable)]
#[table_name = "updates"]
struct NewHoldingsEntry {
    asset_id: i32,
    holdings: f32,
}

#[derive(Queryable)]
struct UpdateAsset {
    id: i32,
    query: String,
    name: String,
}

#[derive(Debug)]
pub enum Error {
    AssetNotFound,
    DatabaseConnectionError(diesel::ConnectionError),
    DatabaseError(diesel::result::Error),
    DatabaseMigrationError(diesel_migrations::RunMigrationsError),
    AlphaVantageError(alphav::AlphaVantageError),
}

impl From<alphav::AlphaVantageError> for Error {
    fn from(e: alphav::AlphaVantageError) -> Self {
        Error::AlphaVantageError(e)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Error::DatabaseError(e)
    }
}

impl From<diesel::ConnectionError> for Error {
    fn from(e: diesel::ConnectionError) -> Self {
        Error::DatabaseConnectionError(e)
    }
}

impl From<diesel_migrations::RunMigrationsError> for Error {
    fn from(e: diesel_migrations::RunMigrationsError) -> Self {
        Error::DatabaseMigrationError(e)
    }
}

#[cfg(test)]
mod tests {
    use diesel::{QueryDsl, RunQueryDsl};
    use std::thread::sleep;
    use std::time::Duration;
    use crate::schema::*;

    #[test]
    fn add_assets() {
        dotenv::dotenv().ok();

        let api_token = std::env::var("ALPHA_VANTAGE_KEY")
            .expect("ALPHA_VANTAGE_KEY must be set");

        let assets = super::Assets::new(":memory:", &api_token).unwrap();
        crate::embedded_migrations::run(&assets.db_client).unwrap();

        let siemens_ref = assets.add_asset(
            "Siemens",
            None,
            "stock/SIE/EUR",
            Some("stock")
        ).unwrap();

        let asset_list = assets.list_assets().unwrap();
        assert_eq!(asset_list.len(), 1);
        let siemens = asset_list.first().unwrap();
        assert_eq!(siemens.0, 1);
        assert_eq!(siemens.1, "Siemens");
        assert_eq!(siemens.3, 0.0);

        sleep(Duration::from_secs(1));

        siemens_ref.update_holdings(2.0).unwrap();
        let asset_list = assets.list_assets().unwrap();
        assert_eq!(asset_list.len(), 1);
        let siemens = asset_list.first().unwrap();
        assert_eq!(siemens.3, 2.0);

        let price_updates: i64 = prices::table
            .select(diesel::dsl::count_star())
            .get_result(&assets.db_client)
            .unwrap();
        assert_eq!(price_updates, 1);
        assets.update_prices();
        let price_updates: i64 = prices::table
            .select(diesel::dsl::count_star())
            .get_result(&assets.db_client)
            .unwrap();
        assert_eq!(price_updates, 2);
    }
}