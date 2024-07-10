use crate::{
    error::AiScriptError,
    parser::{node as cst, visit::Visitor},
};

#[derive(Debug, PartialEq, Clone)]
struct ChainTransformer;

impl Visitor for ChainTransformer {
    fn callback_expression(
        &self,
        expression: cst::Expression,
    ) -> Result<cst::Expression, AiScriptError> {
        // chain
        match &expression {
            cst::Expression::Not(cst::Not {
                chain: Some(chain), ..
            })
            | cst::Expression::And(cst::And {
                chain: Some(chain), ..
            })
            | cst::Expression::Or(cst::Or {
                chain: Some(chain), ..
            })
            | cst::Expression::If(cst::If {
                chain: Some(chain), ..
            })
            | cst::Expression::Fn(cst::Fn_ {
                chain: Some(chain), ..
            })
            | cst::Expression::Match(cst::Match {
                chain: Some(chain), ..
            })
            | cst::Expression::Block(cst::Block {
                chain: Some(chain), ..
            })
            | cst::Expression::Exists(cst::Exists {
                chain: Some(chain), ..
            })
            | cst::Expression::Tmpl(cst::Tmpl {
                chain: Some(chain), ..
            })
            | cst::Expression::Str(cst::Str {
                chain: Some(chain), ..
            })
            | cst::Expression::Num(cst::Num {
                chain: Some(chain), ..
            })
            | cst::Expression::Bool(cst::Bool {
                chain: Some(chain), ..
            })
            | cst::Expression::Null(cst::Null {
                chain: Some(chain), ..
            })
            | cst::Expression::Obj(cst::Obj {
                chain: Some(chain), ..
            })
            | cst::Expression::Arr(cst::Arr {
                chain: Some(chain), ..
            })
            | cst::Expression::Identifier(cst::Identifier {
                chain: Some(chain), ..
            }) => Ok(chain.iter().fold(
                match &expression {
                    cst::Expression::Not(not) => cst::Expression::Not(cst::Not {
                        chain: None,
                        ..not.clone()
                    }),
                    cst::Expression::And(and) => cst::Expression::And(cst::And {
                        chain: None,
                        ..and.clone()
                    }),
                    cst::Expression::Or(or) => cst::Expression::Or(cst::Or {
                        chain: None,
                        ..or.clone()
                    }),
                    cst::Expression::If(if_) => cst::Expression::If(cst::If {
                        chain: None,
                        ..if_.clone()
                    }),
                    cst::Expression::Fn(fn_) => cst::Expression::Fn(cst::Fn_ {
                        chain: None,
                        ..fn_.clone()
                    }),
                    cst::Expression::Match(match_) => cst::Expression::Match(cst::Match {
                        chain: None,
                        ..match_.clone()
                    }),
                    cst::Expression::Block(block) => cst::Expression::Block(cst::Block {
                        chain: None,
                        ..block.clone()
                    }),
                    cst::Expression::Exists(exists) => cst::Expression::Exists(cst::Exists {
                        chain: None,
                        ..exists.clone()
                    }),
                    cst::Expression::Tmpl(tmpl) => cst::Expression::Tmpl(cst::Tmpl {
                        chain: None,
                        ..tmpl.clone()
                    }),
                    cst::Expression::Str(str) => cst::Expression::Str(cst::Str {
                        chain: None,
                        ..str.clone()
                    }),
                    cst::Expression::Num(num) => cst::Expression::Num(cst::Num {
                        chain: None,
                        ..num.clone()
                    }),
                    cst::Expression::Bool(bool) => cst::Expression::Bool(cst::Bool {
                        chain: None,
                        ..bool.clone()
                    }),
                    cst::Expression::Null(null) => cst::Expression::Null(cst::Null {
                        chain: None,
                        ..null.clone()
                    }),
                    cst::Expression::Obj(obj) => cst::Expression::Obj(cst::Obj {
                        chain: None,
                        ..obj.clone()
                    }),
                    cst::Expression::Arr(arr) => cst::Expression::Arr(cst::Arr {
                        chain: None,
                        ..arr.clone()
                    }),
                    cst::Expression::Identifier(identifier) => {
                        cst::Expression::Identifier(cst::Identifier {
                            chain: None,
                            ..identifier.clone()
                        })
                    }
                    cst::Expression::Call(call) => cst::Expression::Call(call.clone()),
                    cst::Expression::Index(index) => cst::Expression::Index(index.clone()),
                    cst::Expression::Prop(prop) => cst::Expression::Prop(prop.clone()),
                },
                |parent, chain_member| match chain_member {
                    cst::ChainMember::CallChain(call_chain) => cst::Expression::Call(cst::Call {
                        target: parent.into(),
                        args: call_chain.args.clone(),
                        loc: call_chain.loc.clone(),
                    }),
                    cst::ChainMember::IndexChain(index_chain) => {
                        cst::Expression::Index(cst::Index {
                            target: parent.into(),
                            index: index_chain.index.clone().into(),
                            loc: index_chain.loc.clone(),
                        })
                    }
                    cst::ChainMember::PropChain(prop_chain) => cst::Expression::Prop(cst::Prop {
                        target: parent.into(),
                        name: prop_chain.name.clone(),
                        loc: prop_chain.loc.clone(),
                    }),
                },
            )),
            _ => Ok(expression),
        }
    }
}

pub fn transform_chain(
    nodes: impl IntoIterator<Item = cst::Node>,
) -> Result<Vec<cst::Node>, AiScriptError> {
    nodes
        .into_iter()
        .map(|node| ChainTransformer.visit_node(node))
        .collect()
}
