use crate::{
    error::AiScriptSyntaxError,
    node as ast,
    parser::{
        plugins::{
            validate_jump_statement::validate_jump_statement, validate_keyword::validate_keyword,
            validate_type::validate_type,
        },
        syntaxes::toplevel::parse_top_level,
    },
};

use self::scanner::read_tokens;

mod plugins;
mod scanner;
mod syntaxes;
mod token;
mod visit;

pub type ParserPlugin = fn(Vec<ast::Node>) -> Result<Vec<ast::Node>, AiScriptSyntaxError>;

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
            validate: vec![validate_keyword, validate_type, validate_jump_statement],
            transform: Vec::new(),
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

    pub fn parse(&self, input: &str) -> Result<Vec<ast::Node>, AiScriptSyntaxError> {
        let mut tokens = read_tokens(input)?;
        tokens.reverse();
        let nodes = parse_top_level(&mut tokens)?;
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
        Ok(nodes)
    }

    pub fn add_plugin(&mut self, plugin: PluginType) {
        match plugin {
            PluginType::Validate(plugin) => self.plugins.validate.push(plugin),
            PluginType::Transform(plugin) => self.plugins.transform.push(plugin),
        }
    }
}
