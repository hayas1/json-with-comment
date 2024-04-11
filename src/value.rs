pub mod de;
pub mod from;
pub mod index;
pub mod into;
pub mod macros;
pub mod number;
pub mod ser;

#[cfg(not(feature = "preserve_order"))]
pub type MapImpl<K, V> = std::collections::HashMap<K, V>;
#[cfg(feature = "preserve_order")]
pub type MapImpl<K, V> = indexmap::IndexMap<K, V>;

/// Represents any valid JSON with comments value.
///
/// # Examples
/// see [`crate`] document also.
/// ```
/// use json_with_comments::{jsonc_generics, value::JsoncValue};
///
/// let mut value: JsoncValue<u32, f32> = jsonc_generics!({
///     "name": "json-with-comments",
///     "keywords": [
///         "JSON with comments",
///         "JSONC",
///         "trailing comma",
///     ],
/// });
///
/// // index access
/// assert_eq!(value["name"], JsoncValue::String("json-with-comments".to_string()));
/// assert_eq!(
///     value["keywords"].get(..=1),
///     Some(
///         &[JsoncValue::String("JSON with comments".to_string()), JsoncValue::String("JSONC".to_string())][..]
///     )
/// );
///
/// // mutable access
/// value["name"] = "json_with_comments".into();
/// if let Some(JsoncValue::String(jsonc)) = value["keywords"].get_mut(1) {
///     *jsonc = jsonc.to_lowercase();
/// }
/// assert_eq!(value, jsonc_generics!({
///     "name": "json_with_comments",
///     "keywords": [
///         "JSON with comments",
///         "jsonc",
///         "trailing comma",
///     ],
/// }));
/// ```
#[derive(Debug, Clone, PartialEq)]
// if JsoncValue<'a, I, F>, cannot implement FromStr
pub enum JsoncValue<I, F> {
    /// Represents any valid JSON with comments object.
    /// Default implementation is `HashMap`. If `preserve_order` feature is enabled, that will be `IndexMap`.
    Object(MapImpl<String, JsoncValue<I, F>>),

    /// Represents any valid JSON with comments array.
    Array(Vec<JsoncValue<I, F>>),

    /// Represents any valid JSON with comments boolean.
    Bool(bool),

    /// Represents any valid JSON with comments null.
    Null,

    /// Represents any valid JSON with comments string.
    String(String),

    /// Represents any valid JSON with comments number, whether integer or float.
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

    /// Replaces value with the default value `Null`, returning the previous value.
    ///
    /// # Examples
    /// ```
    /// use json_with_comments::jsonc;
    /// let mut value = jsonc!({
    ///     "name": "json-with-comments",
    ///     "keywords": [
    ///         "JSON with comments",
    ///         "JSONC",
    ///         "trailing comma",
    ///     ],
    /// });
    ///
    /// let name = value["name"].take();
    /// assert_eq!(name, "json-with-comments".into());
    /// assert_eq!(value, jsonc!({
    ///     "name": null,
    ///     "keywords": [
    ///         "JSON with comments",
    ///         "JSONC",
    ///         "trailing comma"
    ///     ]
    /// }));
    /// ```
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
