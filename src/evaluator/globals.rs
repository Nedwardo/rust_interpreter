use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    evaluator::environment::Environment,
    expressions::{Function, FunctionKind, Value},
};

pub fn define_globals(scope: &mut Environment<'_>) {
    scope.add_global(
        "clock",
        Value::Function(Function {
            body: FunctionKind::Rust(clock),
            params: vec![],
            name: "clock",
        }),
    );
}

#[allow(clippy::cast_precision_loss, clippy::unnecessary_wraps)]
fn clock(_: Vec<Value<'_>>) -> Value<'_> {
    let unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time should go forwards");
    let milliseconds = (unix_time.as_secs() * 1000) as f64
        + f64::from(unix_time.subsec_millis()) / 1_000_000.0;
    Value::Number(milliseconds)
}
