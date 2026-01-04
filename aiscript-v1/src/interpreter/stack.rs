use crate::error::AiScriptError;

use super::{frame::Frame, value::Value};

pub trait StackExt {
    fn eval(&mut self, node: impl Into<Frame>);

    fn run(&mut self, program: Vec<impl Into<Frame>>);
}

impl StackExt for Vec<Frame> {
    fn eval(&mut self, node: impl Into<Frame>) {
        self.push(node.into());
        self.push(Frame::Eval);
    }

    fn run(&mut self, program: Vec<impl Into<Frame>>) {
        self.push(Frame::Run);
        for node in program.into_iter().rev() {
            self.push(Frame::Unwind);
            self.eval(node);
        }
    }
}

pub trait ValueStackExt {
    fn pop_value(&mut self) -> Result<Value, AiScriptError>;
}

impl ValueStackExt for Vec<Value> {
    fn pop_value(&mut self) -> Result<Value, AiScriptError> {
        self.pop()
            .ok_or_else(|| AiScriptError::internal("stack is empty"))
    }
}
