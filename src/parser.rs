use crate::{
    error::{AiScriptError, AiScriptSyntaxError},
    node as ast,
};

use self::{
    node as cst,
    parser::parser::{main, preprocess},
    plugins::{
        set_attribute::set_attribute, transform_chain::transform_chain,
        validate_keyword::validate_keyword, validate_type::validate_type,
    },
};

pub mod node;
#[allow(clippy::module_inception)]
mod parser;
mod plugins;
mod visit;

pub type ParserPlugin = fn(Vec<cst::Node>) -> Result<Vec<cst::Node>, AiScriptError>;

pub enum PluginType {
    Validate(ParserPlugin),
    Transform(ParserPlugin),
}

struct Plugins {
    pub validate: Vec<ParserPlugin>,
    pub transform: Vec<ParserPlugin>,
}

impl Default for Plugins {
    fn default() -> Self {
        Self {
            validate: vec![validate_keyword, validate_type],
            transform: vec![set_attribute, transform_chain],
        }
    }
}

#[derive(Default)]
pub struct Parser {
    plugins: Plugins,
}

impl Parser {
    pub fn new(validate: Vec<ParserPlugin>, transform: Vec<ParserPlugin>) -> Self {
        Parser {
            plugins: Plugins {
                validate,
                transform,
            },
        }
    }

    pub fn parse(&self, input: &str) -> Result<Vec<ast::Node>, AiScriptError> {
        let code = preprocess(input).map_err(AiScriptSyntaxError::Parse)?;
        let nodes: Vec<node::Node> = main(&code).map_err(AiScriptSyntaxError::Parse)?;
        let nodes = self
            .plugins
            .validate
            .iter()
            .try_fold(nodes, |nodes, plugin| plugin(nodes))?;
        let nodes = self
            .plugins
            .transform
            .iter()
            .try_fold(nodes, |nodes, plugin| plugin(nodes))?;
        Ok(nodes.into_iter().map(Into::into).collect())
    }

    pub fn add_plugin(&mut self, plugin: PluginType) {
        match plugin {
            PluginType::Validate(plugin) => self.plugins.validate.push(plugin),
            PluginType::Transform(plugin) => self.plugins.transform.push(plugin),
        }
    }
}
