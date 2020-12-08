use assetman_api::{Answer, PluginInfo, PluginType, Request};
use electrum_client::ElectrumApi;
use log::debug;
use serde_json::{de::Deserializer, to_writer};
use std::io::{stdin, stdout, Write};

fn main() {
    pretty_env_logger::init();

    let mut stdout = stdout();
    let mut stdin = stdin();

    let electrum_addr = dotenv::var("AM_ELECTRUM_SERVER").expect("AM_ELECTRUM_SERVER not set!");
    let electrum = electrum_client::Client::new(&electrum_addr).unwrap();

    let gap_limit: usize = dotenv::var("AM_GAP_LIMIT")
        .map(|s| s.parse().expect("malformed gap limit"))
        .unwrap_or(10);

    let info = PluginInfo {
        name: "bitcoin_h".to_string(),
        plugin_type: PluginType::Holdings,
        description: "Returns the sum of the fund held by a list of comma separated list of wallet descriptors".to_string(),
    };
    to_writer(&mut stdout, &info).unwrap();
    stdout.flush().unwrap();

    Deserializer::from_reader(&mut stdin)
        .into_iter()
        .map(
            |req: Result<Request, _>| -> Result<Answer, assetman_api::Error> {
                let req = req.map_err(|e| assetman_api::Error {
                    code: 1,
                    description: format!("Input parsing error: {:?}", e),
                })?;

                let amount_sat = req
                    .arguments
                    .split(",")
                    .map(|descriptor| {
                        let descriptor_ext = descriptor
                            .replace("*", "0/*")
                            .parse()
                            .expect("Invalid descriptor");
                        let descriptor_int = descriptor
                            .replace("*", "1/*")
                            .parse()
                            .expect("Invalid descriptor");

                        debug!("Querying BTC account {} (external)", descriptor);
                        let external = electrum
                            .descriptor_balance(&descriptor_ext, gap_limit, false)
                            .unwrap();

                        debug!("Querying BTC account {} (internal)", descriptor);
                        let internal = electrum
                            .descriptor_balance(&descriptor_int, gap_limit, false)
                            .unwrap();

                        internal + external
                    })
                    .sum::<u64>() as f64;

                Ok(Answer {
                    answer: amount_sat / 100_000_000.0,
                })
            },
        )
        .for_each(|resp| {
            to_writer(&mut stdout, &resp).unwrap();
            stdout.flush().unwrap();
        });
}
