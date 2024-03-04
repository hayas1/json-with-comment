pub mod de;
pub mod number;
pub mod string;

pub type MapImpl<K, V> = std::collections::HashMap<K, V>;

#[derive(Debug, Clone, PartialEq)]
pub enum JsoncValue<'a, I, F> {
    Number(number::NumberValue<I, F>),
    String(string::StringValue<'a>),
    Object(MapImpl<string::StringValue<'a>, JsoncValue<'a, I, F>>),
    Array(Vec<JsoncValue<'a, I, F>>),
    Null,
    Bool(bool),
}
