extern crate chrono;
extern crate dotenv;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate reqwest;
extern crate serde_json;
extern crate textplots;

use diesel::deserialize::Queryable;
use diesel::prelude::*;

use crate::plugins::{PluginError, Plugins};
use chrono::NaiveDateTime;
use diesel::dsl::max;
use diesel::select;
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
    id: i32,
    name: String,
    price: f64,
    holdings: f64,
    category: String,
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
                    .and(schema::holdings::update_id.eq(last_snapshot)),
            )
            .order_by(schema::assets::name)
            .load::<Asset>(&self.db_client)?;

        Ok(AssetsSnapshot {
            time: time.parse().unwrap(),
            assets,
        })
    }
}
/*
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
}*/

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
