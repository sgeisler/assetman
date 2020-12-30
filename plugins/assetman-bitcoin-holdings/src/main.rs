use assetman_api::{Answer, PluginInfo, PluginType, Request};
use electrum_client::{Descriptor, ElectrumApi};
use log::debug;
use miniscript::bitcoin::util::bip32::ChildNumber;
use miniscript::descriptor::{DescriptorPublicKey, DescriptorXKey};
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
                    .split(";")
                    .map(|descriptor| {
                        let descriptor_base = descriptor
                            .parse::<Descriptor>()
                            .expect("Invalid descriptor");

                        let derive_ext =
                            |pk: &DescriptorPublicKey| -> Result<DescriptorPublicKey, ()> {
                                Ok(derive_normal_chain(pk, 0))
                            };
                        let descriptor_ext = descriptor_base
                            .translate_pk(derive_ext, derive_ext)
                            .expect("Transformation can't fail");

                        let derive_int =
                            |pk: &DescriptorPublicKey| -> Result<DescriptorPublicKey, ()> {
                                Ok(derive_normal_chain(pk, 1))
                            };
                        let descriptor_int = descriptor_base
                            .translate_pk(derive_int, derive_int)
                            .expect("Transformation can't fail");

                        debug!("Querying BTC account {} (external)", descriptor);
                        let external = electrum
                            .descriptor_balance(&descriptor_ext, gap_limit, 10, false)
                            .unwrap();

                        debug!("Querying BTC account {} (internal)", descriptor);
                        let internal = electrum
                            .descriptor_balance(&descriptor_int, gap_limit, 10, false)
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

fn derive_normal_chain(pk: &DescriptorPublicKey, idx: u32) -> DescriptorPublicKey {
    match pk.clone() {
        DescriptorPublicKey::XPub(xpk) if xpk.is_wildcard => {
            DescriptorPublicKey::XPub(DescriptorXKey {
                derivation_path: xpk
                    .derivation_path
                    .child(ChildNumber::Normal { index: idx }),
                ..xpk
            })
        }
        pk => pk,
    }
}
