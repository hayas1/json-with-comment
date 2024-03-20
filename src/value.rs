pub mod de;
pub mod from;
pub mod index;
pub mod into;
pub mod macros;
pub mod number;
pub mod ser;

pub type MapImpl<K, V> = std::collections::HashMap<K, V>;

/// TODO doc
#[derive(Debug, Clone, PartialEq)]
// if JsoncValue<'a, I, F>, cannot implement FromStr
pub enum JsoncValue<I, F> {
    Object(MapImpl<String, JsoncValue<I, F>>),
    Array(Vec<JsoncValue<I, F>>),
    Bool(bool),
    Null,
    String(String),
    Number(number::Number<I, F>),
}

impl<I, F> Default for JsoncValue<I, F> {
    fn default() -> Self {
        Self::Null
    }
}
impl<I: num::FromPrimitive, F: num::FromPrimitive> std::str::FromStr for JsoncValue<I, F> {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::from_str(s)
    }
}
impl<I, F> JsoncValue<I, F> {
    /// TODO doc
    pub fn query(&self, query: &str) -> Option<&JsoncValue<I, F>> {
        // TODO better implement, tests
        query.split('.').try_fold(self, |value, key| match value {
            JsoncValue::Object(map) => map.get(key),
            JsoncValue::Array(vec) => vec.get(key.parse::<usize>().ok()?),
            _ => None,
        })
    }

    /// TODO doc
    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }

    /// TODO doc
    /// get the value type representation of [`JsoncValue`]
    pub fn value_type(&self) -> String {
        match self {
            JsoncValue::Object(_) => "Object",
            JsoncValue::Array(_) => "Array",
            JsoncValue::Bool(_) => "Boolean",
            JsoncValue::Null => "Null",
            JsoncValue::String(_) => "String",
            JsoncValue::Number(_) => "Number",
        }
        .to_string()
    }
}

/// Deserialize [`JsoncValue`] to `T`
///
/// # Example
/// TODO example
pub fn from_value<'de, T, I, F>(value: JsoncValue<I, F>) -> crate::Result<T>
where
    T: serde::de::Deserialize<'de>,
    I: serde::de::Deserialize<'de>,
    F: serde::de::Deserialize<'de>,
{
    T::deserialize(value)
}

/// Serialize `T` to [`JsoncValue`]
///
/// # Example
/// TODO example
pub fn to_value<T, I, F>(value: T) -> crate::Result<JsoncValue<I, F>>
where
    T: serde::Serialize,
    I: serde::Serialize,
    F: serde::Serialize,
{
    value.serialize(ser::serializer::ValueSerializer::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonc;

    #[test]
    fn test_from_value() {
        let v = jsonc!(true);
        let t: bool = from_value(v).unwrap();
        assert_eq!(t, true);
    }

    #[test]
    fn test_to_value() {
        let v = jsonc!(true);
        let t: JsoncValue<i64, f64> = to_value(v).unwrap();
        assert_eq!(t, JsoncValue::Bool(true));
    }
}
