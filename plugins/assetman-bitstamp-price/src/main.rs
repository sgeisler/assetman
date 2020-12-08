use assetman_api::{Answer, PluginInfo, PluginType, Request};
use reqwest::blocking::get;
use serde_json::{de::Deserializer, to_writer, Value};
use std::collections::HashMap;
use std::io::{stdin, stdout, Write};

fn main() {
    let mut stdout = stdout();
    let mut stdin = stdin();

    let mut cache = HashMap::<String, f64>::new();

    let info = PluginInfo {
        name: "bitstamp".to_string(),
        plugin_type: PluginType::Price,
        description: "Returns the BTC price in the currency pair (e.g. BTCUSD) given as argument"
            .to_string(),
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

                if let Some(price) = cache.get(&req.arguments) {
                    return Ok(Answer { answer: *price });
                }

                let biststamp_resp = get(&format!(
                    "https://www.bitstamp.net/api/v2/ticker/{}",
                    req.arguments
                ))
                .map_err(|e| assetman_api::Error {
                    code: 3,
                    description: format!("HTTP error: {:?}", e),
                })?;

                let price = biststamp_resp
                    .json()
                    .map_err(|e| assetman_api::Error {
                        code: 4,
                        description: format!("Invalid API response, parse error: {:?}", e),
                    })
                    .and_then(|json: Value| -> Result<f64, assetman_api::Error> {
                        json.get("bid")
                            .ok_or(assetman_api::Error {
                                code: 5,
                                description: format!("Invalid API response, no 'bid' field."),
                            })?
                            .as_str()
                            .ok_or(assetman_api::Error {
                                code: 6,
                                description: format!(
                                    "Invalid API response: 'bid' is not a string!"
                                ),
                            })?
                            .parse()
                            .map_err(|e| assetman_api::Error {
                                code: 7,
                                description: format!(
                                    "Invalid API response: 'bid' is not a valid float string: {:?}",
                                    e
                                ),
                            })
                    })?;

                cache.insert(req.arguments, price);

                Ok(Answer { answer: price })
            },
        )
        .for_each(|resp| {
            to_writer(&mut stdout, &resp).unwrap();
            stdout.flush().unwrap();
        });
}
