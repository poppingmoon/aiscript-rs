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
        fn process_chain(
            parent: cst::Expression,
            chain_member: &cst::ChainMember,
        ) -> cst::Expression {
            match chain_member {
                cst::ChainMember::CallChain(call_chain) => cst::Expression::Call(
                    cst::Call {
                        target: parent.into(),
                        args: call_chain.args.clone(),
                        loc: call_chain.loc.clone(),
                    }
                    .into(),
                ),
                cst::ChainMember::IndexChain(index_chain) => cst::Expression::Index(
                    cst::Index {
                        target: parent.into(),
                        index: index_chain.index.clone().into(),
                        loc: index_chain.loc.clone(),
                    }
                    .into(),
                ),
                cst::ChainMember::PropChain(prop_chain) => cst::Expression::Prop(
                    cst::Prop {
                        target: parent.into(),
                        name: prop_chain.name.clone(),
                        loc: prop_chain.loc.clone(),
                    }
                    .into(),
                ),
            }
        }
        match &expression {
            cst::Expression::Not(not) => not.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Not(
                        cst::Not {
                            chain: None,
                            ..*not.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::And(and) => and.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::And(
                        cst::And {
                            chain: None,
                            ..*and.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Or(or) => or.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Or(
                        cst::Or {
                            chain: None,
                            ..*or.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::If(if_) => if_.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::If(
                        cst::If {
                            chain: None,
                            ..*if_.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Fn(fn_) => fn_.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Fn(
                        cst::Fn_ {
                            chain: None,
                            ..*fn_.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Match(match_) => match_.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Match(
                        cst::Match {
                            chain: None,
                            ..*match_.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Block(block) => block.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Block(
                        cst::Block {
                            chain: None,
                            ..*block.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Exists(exists) => exists.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Exists(
                        cst::Exists {
                            chain: None,
                            ..*exists.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Tmpl(tmpl) => tmpl.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Tmpl(
                        cst::Tmpl {
                            chain: None,
                            ..*tmpl.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Str(str) => str.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Str(
                        cst::Str {
                            chain: None,
                            ..*str.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Num(num) => num.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Num(
                        cst::Num {
                            chain: None,
                            ..*num.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Bool(bool) => bool.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Bool(
                        cst::Bool {
                            chain: None,
                            ..*bool.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Null(null) => null.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Null(
                        cst::Null {
                            chain: None,
                            ..*null.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Obj(obj) => obj.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Obj(
                        cst::Obj {
                            chain: None,
                            ..*obj.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Arr(arr) => arr.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Arr(
                        cst::Arr {
                            chain: None,
                            ..*arr.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Identifier(identifier) => identifier.chain.as_ref().map(|chain| {
                Ok(chain.iter().fold(
                    cst::Expression::Identifier(
                        cst::Identifier {
                            chain: None,
                            ..*identifier.clone()
                        }
                        .into(),
                    ),
                    process_chain,
                ))
            }),
            cst::Expression::Call(_) | cst::Expression::Index(_) | cst::Expression::Prop(_) => None,
        }
        .unwrap_or(Ok(expression))
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
