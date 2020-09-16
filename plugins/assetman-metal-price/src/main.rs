use assetman_api::{Answer, PluginInfo, PluginType, Request};
use serde_json::{de::Deserializer, to_writer};
use std::io::{stdin, stdout, Write};

fn main() {
    let mut stdout = stdout();
    let mut stdin = stdin();

    let info = PluginInfo {
        name: "metal_p".to_string(),
        plugin_type: PluginType::Price,
        description: "Returns the price of gold and silver (given as argument)".to_string(),
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

                let api_response: serde_json::Value = reqwest::blocking::get("http://data-asg.goldprice.org/dbXRates/EUR").map_err(|e| assetman_api::Error {
                    code: 2,
                    description: format!("Request error: {:?}", e),
                })?.json().map_err(|e| assetman_api::Error {
                    code: 3,
                    description: format!("Response parsing error: {:?}", e),
                })?;

                let metals = api_response.get("items").unwrap().get(0).unwrap();

                let price = match req.arguments.as_str() {
                    "gold" => {
                        metals.get("xauPrice")
                    },
                    "silver" => {
                        metals.get("xagPrice")
                    },
                    _ => {
                        return Err(assetman_api::Error {
                            code: 4,
                            description: "invalid argument, only silver or gold are valid".into(),
                        });
                    }
                };

                Ok(Answer { answer: price.unwrap().as_f64().unwrap() })
            }
        )
        .for_each(|resp| {
            to_writer(&mut stdout, &resp).unwrap();
            stdout.flush().unwrap();
        });
}
