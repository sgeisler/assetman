use assetman_api::{Answer, PluginInfo, PluginType, Request};
use regex::Regex;
use serde::export::Formatter;
use serde_json::de::Deserializer;
use serde_json::to_writer;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::path::Path;
use std::process::{Child, Command, Stdio};

#[derive(Debug)]
pub struct Plugins {
    plugins: HashMap<String, Plugin>,
}

#[derive(Debug)]
struct Plugin {
    process: Child,
    meta: PluginInfo,
}

impl Plugins {
    pub fn from_paths<P: AsRef<Path>>(paths: impl Iterator<Item = P>) -> Result<Self, PluginError> {
        let plugins = paths
            .map(|ref path| {
                let mut plugin = Command::new(path.as_ref())
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()?;

                let plugin_info = Deserializer::from_reader(
                    plugin
                        .stdout
                        .as_mut()
                        .expect("Plugin child process has no stdout"),
                )
                .into_iter::<PluginInfo>()
                .next()
                .expect("Plugin sent no info.")
                .map_err(|e| PluginError::BadAnswer)?;

                Ok((
                    plugin_info.name.clone(),
                    Plugin {
                        process: plugin,
                        meta: plugin_info,
                    },
                ))
            })
            .collect::<Result<HashMap<String, Plugin>, PluginError>>()?;

        Ok(Plugins { plugins })
    }

    pub fn query_plugin(
        &mut self,
        plugin: &str,
        arguments: &str,
        expected_type: PluginType,
    ) -> Result<f64, PluginError> {
        let plugin = self
            .plugins
            .get_mut(plugin)
            .ok_or(PluginError::UnknownPlugin)?;

        if plugin.meta.plugin_type != expected_type {
            return Err(PluginError::WrongType);
        }

        let stdin = plugin
            .process
            .stdin
            .as_mut()
            .expect("Plugin child process has no stdin");
        let stdout = plugin
            .process
            .stdout
            .as_mut()
            .expect("Plugin child process has no stdout");

        let req = Request {
            arguments: arguments.to_string(),
        };
        to_writer(stdin, &req);

        let answer = Deserializer::from_reader(stdout)
            .into_iter::<Result<Answer, assetman_api::Error>>()
            .next()
            .ok_or(PluginError::BadAnswer)?
            .map_err(|_| PluginError::BadAnswer)??;
        Ok(answer.answer)
    }

    pub fn query(&mut self, query: &str, expected_type: PluginType) -> Result<f64, PluginError> {
        let query_re = Regex::new(r"([^(]*)\((.*)\)").unwrap();
        let captures = query_re
            .captures(query)
            .ok_or(PluginError::QueryParseError)?;
        let plugin = captures
            .get(1)
            .ok_or(PluginError::QueryParseError)?
            .as_str();
        let arguments = captures
            .get(2)
            .ok_or(PluginError::QueryParseError)?
            .as_str();

        self.query_plugin(plugin, arguments, expected_type)
    }
}

#[derive(Debug)]
pub enum PluginError {
    PluginStartupFailed(std::io::Error),
    QueryParseError,
    UnknownPlugin,
    WrongType,
    BadAnswer,
    PluginError(assetman_api::Error),
}

impl std::error::Error for PluginError {}

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl From<std::io::Error> for PluginError {
    fn from(e: std::io::Error) -> Self {
        PluginError::PluginStartupFailed(e)
    }
}

impl From<assetman_api::Error> for PluginError {
    fn from(e: assetman_api::Error) -> Self {
        PluginError::PluginError(e)
    }
}

#[cfg(test)]
mod tests {
    use crate::plugins::Plugins;
    use assetman_api::PluginType::Holdings;

    #[test]
    fn test_plugins() {
        let plugins = &[
            "../target/debug/assetman-static-holdings",
            "../target/debug/assetman-static-price",
        ];
        let mut registry = Plugins::from_paths(plugins.into_iter()).unwrap();
        assert_eq!(registry.query("static_h(1.234)", Holdings).unwrap(), 1.234);
    }
}
