extern crate chrono;
extern crate dotenv;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
extern crate quandl_v3;

use diesel::prelude::*;
use diesel::query_dsl::*;
use quandl_v3::prelude::*;

use schema::*;

pub mod quandl;
mod schema;

embed_migrations!();

pub struct Assets {
    db_client: diesel::sqlite::SqliteConnection,
    qunadl_client: quandl::QuandlClient,
}

impl Assets {
    pub fn new(db_path: &str, quandl_api_key: &str) -> Result<Assets, Error> {
        let assets = Assets {
            db_client: diesel::SqliteConnection::establish(db_path)?,
            qunadl_client: quandl::QuandlClient::new(quandl_api_key.into()),
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
        //  Wprices.timestamp=(select max(timestamp) from prices where prices.asset_id=assets.id);
        schema::assets::table
            .inner_join(schema::updates::table)
            .inner_join(schema::prices::table)
            .select((schema::assets::id, schema::assets::name, schema::prices::price, schema::updates::holdings, schema::assets::category))
            .filter(
                schema::updates::timestamp.eq(diesel::dsl::sql::<diesel::sql_types::Timestamp>("(SELECT MAX(timestamp) FROM updates WHERE assets.id = updates.asset_id)")).and(
                    schema::prices::timestamp.eq(diesel::dsl::sql::<diesel::sql_types::Timestamp>("(SELECT MAX(timestamp) FROM prices WHERE assets.id = prices.asset_id)"))
                )
            )
            .order_by(schema::assets::name)
            .load(&self.db_client)
    }

    pub fn add_asset(&self, name: &str, description: Option<&str>, quandl_database: &str, quandl_dataset: &str, quandl_idx: u16, category: Option<&str>) -> Result<Asset, Error> {
        self.db_client.transaction(|| {
            let price = self.qunadl_client.query_last(
                quandl_database,
                quandl_dataset,
                quandl_idx as usize
            )?;

            diesel::insert_into(schema::assets::table)
                .values(NewAsset {
                    name,
                    description,
                    quandl_database,
                    quandl_dataset,
                    quandl_price_idx: quandl_idx as i32,
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
                .filter(schema::assets::quandl_database.is_not_null())
                .select((
                    schema::assets::id,
                    schema::assets::quandl_database,
                    schema::assets::quandl_dataset,
                    schema::assets::quandl_price_idx
                ))
                .load(&self.db_client)?;

            for asset in assets {
                let price = self.qunadl_client.query_last(
                    &asset.quandl_database.expect("not possible due to query and constraints"),
                    &asset.quandl_dataset.expect("not possible due to query and constraints"),
                    asset.quandl_price_idx.expect("not possible due to query and constraints") as usize
                )?;

                diesel::insert_into(schema::prices::table)
                    .values(NewPriceEntry {
                        asset_id: asset.id,
                        price: price as f32,
                    })
                    .execute(&self.db_client)?;
            }

            Ok(())
        })
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
    quandl_database: &'a str,
    quandl_dataset: &'a str,
    quandl_price_idx: i32,
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
    quandl_database: Option<String>,
    quandl_dataset: Option<String>,
    quandl_price_idx: Option<i32>,
}

#[derive(Debug)]
pub enum Error {
    AssetNotFound,
    DatabaseConnectionError(diesel::ConnectionError),
    DatabaseError(diesel::result::Error),
    DatabaseMigrationError(diesel_migrations::RunMigrationsError),
    QuandlError(quandl::Error),
}

impl From<quandl::Error> for Error {
    fn from(e: quandl::Error) -> Self {
        Error::QuandlError(e)
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

        let api_token = std::env::var("QUANDL_TOKEN")
            .expect("QUANDL_TOKEN must be set");

        let assets = super::Assets::new(":memory:", &api_token).unwrap();
        crate::embedded_migrations::run(&assets.db_client).unwrap();

        let siemens_ref = assets.add_asset(
            "Siemens",
            None,
            "FSE",
            "SIE_X",
            4
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