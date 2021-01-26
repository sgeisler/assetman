use assetman_api::{Answer, PluginInfo, PluginType, Request};
use serde_json::{de::Deserializer, to_writer};
use std::io::{stdin, stdout, BufRead, BufReader, Write};

fn main() {
    let mut stdout = stdout();
    let mut stdin = stdin();

    let info = PluginInfo {
        name: "csv_scan".to_string(),
        plugin_type: PluginType::Any,
        description: "Searches a csv file for a certain term in one row column and returns a cell's from the found row. Arguments: path,search_col,search_term,return_col".to_string(),
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

                let args: Vec<&str> = req.arguments.split(',').collect();
                if args.len() != 4 {
                    return Err(assetman_api::Error {
                        code: 2,
                        description: format!("Wrong number of arguments."),
                    });
                }

                let file = args[0];
                let search_col: u32 = args[1].parse().map_err(|e| assetman_api::Error {
                    code: 3,
                    description: format!("Input parsing error: search col not a number. {:?}", e),
                })?;
                let search_term = args[2];
                let return_col: u32 = args[3].parse().map_err(|e| assetman_api::Error {
                    code: 4,
                    description: format!("Input parsing error: return col not a number. {:?}", e),
                })?;

                let file = std::fs::File::open(file).map_err(|e| assetman_api::Error {
                    code: 5,
                    description: format!("Input parsing error: can't open file: {:?}", e),
                })?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    let line = line.map_err(|e| assetman_api::Error {
                        code: 6,
                        description: format!("Input parsing error: read error: {:?}", e),
                    })?;
                    let cells: Vec<&str> = line.split(';').collect();

                    if cells.get(search_col as usize) == Some(&search_term) {
                        let value = cells
                            .get(return_col as usize)
                            .ok_or(assetman_api::Error {
                                code: 7,
                                description: format!("Return col not found."),
                            })?
                            .replace(',', ".")
                            .parse()
                            .map_err(|e| assetman_api::Error {
                                code: 8,
                                description: format!(
                                    "CSV parsing error: return col isn't a float. {:?}",
                                    e
                                ),
                            })?;

                        return Ok(Answer { answer: value });
                    }
                }
                Err(assetman_api::Error {
                    code: 9,
                    description: format!("No row found."),
                })
            },
        )
        .for_each(|resp| {
            to_writer(&mut stdout, &resp).unwrap();
            stdout.flush().unwrap();
        });
}
