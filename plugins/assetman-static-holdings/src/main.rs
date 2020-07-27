use assetman_api::holdings::*;
use serde_json::{to_writer, de::Deserializer};
use std::io::{stdin, stdout, Write};

fn main() {
    let mut stdout = stdout();
    let mut stdin = stdin();

    let info = PluginInfo {
        description: "Returns the static holdings amount given as argument".to_string()
    };
    to_writer(&mut stdout, &info).unwrap();
    stdout.flush().unwrap();

    Deserializer::from_reader(&mut stdin)
        .into_iter()
        .map(|req: Result<HoldingsRequest, _>| -> Result<HoldingsAnswer, assetman_api::Error> {
            let req = req.map_err(|e| {
                assetman_api::Error {
                    code: 1,
                    description: format!("Input parsing error: {:?}", e),
                }
            })?;

            let amt = req.arguments.parse()
                .map_err(|e| {
                    assetman_api::Error {
                        code: 2,
                        description: format!("Amount parsing error: {:?}", e),
                    }
                })?;

            Ok(HoldingsAnswer {
                req,
                holdings: amt
            })
        })
        .for_each(|resp| {
            to_writer(&mut stdout, &resp).unwrap();
            stdout.flush().unwrap();
        });
}
