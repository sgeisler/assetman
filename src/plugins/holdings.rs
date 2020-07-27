use assetman_api::holdings::*;
use serde_json::de::Deserializer;
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::error::Error;
use std::process::{Child, Command, Stdio};
use serde_json::to_writer;
use regex::Regex;
use std::fmt::{Display, Debug};
use serde::export::Formatter;

#[derive(Debug)]
pub enum HoldingsPluginError {
    QueryParseError,
    UnknownPlugin,
    BadAnswer,
}

impl Error for HoldingsPluginError {}

impl Display for HoldingsPluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct HoldingsPlugins {
    plugins: HashMap<String, Child>
}

impl HoldingsPlugins {
    pub fn from_paths<P: AsRef<Path>>(paths: impl Iterator<Item=P>) -> Result<Self, Box<dyn Error>>{
        let plugins = paths.map(|ref path| {
            let mut plugin = Command::new(path.as_ref())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            let plugin_info = Deserializer::from_reader(
                plugin.stdout.as_mut().expect("Plugin child process has no stdout")
            )
                .into_iter::<PluginInfo>()
                .next()
                .expect("Plugin sent no info.")?;

            Ok((plugin_info.name, plugin))
        }).collect::<Result<HashMap<String, Child>, Box<dyn Error>>>()?;

        Ok(HoldingsPlugins {
            plugins
        })
    }

    pub fn query_plugin(&mut self, plugin: &str, arguments: &str) -> Result<f64, Box<dyn Error>> {
        let plugin = self.plugins.get_mut(plugin).ok_or(HoldingsPluginError::UnknownPlugin)?;

        let stdin= plugin.stdin
            .as_mut()
            .expect("Plugin child process has no stdin");
        let stdout = plugin.stdout
            .as_mut()
            .expect("Plugin child process has no stdout");

        let req = HoldingsRequest {
            arguments: arguments.to_string(),
        };
        to_writer(stdin, &req);

        let answer = Deserializer::from_reader(stdout)
            .into_iter::<Result<HoldingsAnswer, assetman_api::Error>>()
            .next()
            .ok_or(HoldingsPluginError::BadAnswer)???;
        Ok(answer.holdings)
    }

    pub fn query(&mut self, query: &str) -> Result<f64, Box<dyn Error>> {
        let query_re = Regex::new(r"(.*)\((.*)\)").unwrap();
        let captures = query_re.captures(query)
            .ok_or(HoldingsPluginError::QueryParseError)?;
        let plugin = captures.get(1)
            .ok_or(HoldingsPluginError::QueryParseError)?
            .as_str();
        let arguments = captures.get(2)
            .ok_or(HoldingsPluginError::QueryParseError)?
            .as_str();

        self.query_plugin(plugin, arguments)
    }
}

#[cfg(test)]
mod tests {
    use crate::plugins::holdings::HoldingsPlugins;

    #[test]
    fn test_plugin_registration() {
        let plugins = &["plugins/assetman-static-holdings/target/debug/assetman-static-holdings"];
        let mut registry = HoldingsPlugins::from_paths(plugins.into_iter()).unwrap();
        assert_eq!(registry.query("static(1.234)").unwrap(), 1.234);
    }
}