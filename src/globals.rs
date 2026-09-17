use crate::{
    expression::{Literal, Value},
    interpreter::{Interpreter, Interrupt, RuntimeError},
};

pub fn clock_native(
    _interpreter: &mut Interpreter,
    _args: Vec<Value>,
) -> Result<Value, Interrupt<RuntimeError>> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    Ok(Value::Literal(Literal::Float(now as f32)))
}
