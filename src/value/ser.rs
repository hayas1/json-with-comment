use serde::Serialize;

use super::{number::Number, JsoncValue};

impl<I: Serialize, F: Serialize> Serialize for JsoncValue<I, F> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            JsoncValue::Object(map) => map.serialize(serializer),
            JsoncValue::Array(vec) => vec.serialize(serializer),
            JsoncValue::Bool(b) => b.serialize(serializer),
            JsoncValue::Null => ().serialize(serializer),
            JsoncValue::String(s) => s.serialize(serializer),
            JsoncValue::Number(n) => match n {
                Number::Integer(i) => i.serialize(serializer),
                Number::Float(f) => f.serialize(serializer),
            },
        }
    }
}
