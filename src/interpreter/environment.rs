use crate::expressions::Value;

pub struct Environment {
    enclosing: Option<&Environment>,
    values: Map<&str, Value>
}

pub impl Environment {
    fn new(enclosing: Option<&Environment>) -> Self{
        Environment{
            enclosing, values: HashMap::new();
            }
        }

    fn define(&mut self, name: &str, value: Value) {
        self.values[name] = value;
    }

    fn get(&self, name: &str) -> Option<&Value> {
        return self.values.get(name);
    }
}