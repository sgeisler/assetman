extern crate chrono;
extern crate dotenv;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate serde_json;

use diesel::prelude::*;

use crate::plugins::{PluginError, Plugins};
use assetman_api::PluginType::{Holdings, Price};
use chrono::NaiveDateTime;
use schema::*;
use std::path::PathBuf;

mod plugins;
mod schema;

embed_migrations!();

pub struct Assets {
    db_client: diesel::sqlite::SqliteConnection,
    plugins: Plugins,
}

#[derive(Debug)]
pub struct AssetsCfg {
    pub db_path: String,
    pub plugins: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct AssetsSnapshot {
    pub time: chrono::NaiveDateTime,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Queryable)]
pub struct Asset {
    pub id: i32,
    pub name: String,
    pub price: f64,
    pub holdings: f64,
    pub category: String,
}

#[derive(Debug, Insertable)]
#[table_name = "assets"]
struct InsertAsset<'a> {
    name: &'a str,
    price_query: &'a str,
    holdings_query: &'a str,
    category: &'a str,
}

#[derive(Debug, Queryable)]
struct QueryAsset {
    id: i32,
    name: String,
    price_query: String,
    holdings_query: String,
    category: String,
}

#[derive(Debug, Insertable)]
#[table_name = "holdings"]
struct InsertHoldings {
    update_id: i32,
    asset_id: i32,
    amount: f64,
}

#[derive(Debug, Insertable)]
#[table_name = "prices"]
struct InsertPrices {
    update_id: i32,
    asset_id: i32,
    price: f64,
}

impl Assets {
    pub fn new(cfg: AssetsCfg) -> Result<Assets, Error> {
        let assets = Assets {
            db_client: diesel::SqliteConnection::establish(&cfg.db_path)?,
            plugins: Plugins::from_paths(cfg.plugins.iter())?,
        };

        assets.run_migrations()?;

        Ok(assets)
    }

    fn run_migrations(&self) -> Result<(), Error> {
        embedded_migrations::run(&self.db_client)?;
        Ok(())
    }

    pub fn list_assets(&self) -> Result<AssetsSnapshot, Error> {
        let (last_snapshot, time) = schema::updates::table
            .select((schema::updates::id, schema::updates::timestamp))
            .order(schema::updates::timestamp.desc())
            .limit(1)
            .get_result::<(i32, String)>(&self.db_client)?;

        let assets = schema::assets::table
            .inner_join(schema::holdings::table)
            .inner_join(schema::prices::table)
            .select((
                schema::assets::id,
                schema::assets::name,
                schema::prices::price,
                schema::holdings::amount,
                schema::assets::category,
            ))
            .filter(
                schema::prices::update_id
                    .eq(last_snapshot)
                    .and(schema::holdings::update_id.eq(last_snapshot))
                    .and(schema::holdings::amount.ne(0f64)),
            )
            .order_by(schema::assets::name)
            .load::<Asset>(&self.db_client)?;

        Ok(AssetsSnapshot {
            time: NaiveDateTime::parse_from_str(&time, "%Y-%m-%d %H:%M:%S").unwrap(),
            assets,
        })
    }

    pub fn add_asset(
        &mut self,
        name: &str,
        category: &str,
        price_query: &str,
        holdings_query: &str,
    ) -> Result<(), Error> {
        let _ = self.plugins.query(price_query, Price)?;
        let _ = self.plugins.query(holdings_query, Holdings)?;

        diesel::insert_into(schema::assets::table)
            .values(InsertAsset {
                name,
                price_query,
                holdings_query,
                category,
            })
            .execute(&self.db_client)?;

        Ok(())
    }

    pub fn fetch_data(&mut self) -> Result<(), Error> {
        let Assets { db_client, plugins } = self;

        db_client.transaction(|| {
            let assets = schema::assets::table.load::<QueryAsset>(db_client)?;

            // create update entry
            diesel::insert_into(schema::updates::table)
                .default_values()
                .execute(db_client)?;
            let update_id = schema::updates::table
                .select(schema::updates::id)
                .order(schema::updates::id.desc())
                .limit(1)
                .get_result(db_client)?;

            for asset in assets {
                let price = plugins.query(&asset.price_query, Price)?;
                let holdings = plugins.query(&asset.holdings_query, Holdings)?;

                diesel::insert_into(schema::prices::table)
                    .values(InsertPrices {
                        update_id,
                        asset_id: asset.id,
                        price,
                    })
                    .execute(db_client)?;

                diesel::insert_into(schema::holdings::table)
                    .values(InsertHoldings {
                        update_id,
                        asset_id: asset.id,
                        amount: holdings,
                    })
                    .execute(db_client)?;
            }

            Ok(())
        })
    }
}

impl AssetsCfg {
    pub fn from_env() -> Result<Self, &'static str> {
        let database = dotenv::var("AM_DATABASE").map_err(|_| "AM_DATABASE not set!")?;
        let plugins = dotenv::var("AM_PLUGINS")
            .map_err(|_| "AM_PLUGINS not set!")?
            .split(":")
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        Ok(AssetsCfg {
            db_path: database,
            plugins,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    AssetNotFound,
    DatabaseConnectionError(diesel::ConnectionError),
    DatabaseError(diesel::result::Error),
    DatabaseMigrationError(diesel_migrations::RunMigrationsError),
    PluginError(PluginError),
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

impl From<PluginError> for Error {
    fn from(e: PluginError) -> Self {
        Error::PluginError(e)
    }
}
