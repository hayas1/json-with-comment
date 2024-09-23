pub mod de;
pub mod from;
pub mod index;
pub mod into;
pub mod macros;
pub mod number;
pub mod ser;

#[cfg(feature = "raw_value")]
pub mod raw;

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
///
/// // as rust value
/// let v = value["keywords"].as_vec().unwrap();
/// let mut iter = v.iter();
/// assert_eq!(iter.next().unwrap().as_str().unwrap(), "JSON with comments");
/// assert_eq!(iter.next().unwrap().as_str().unwrap(), "jsonc");
/// assert_eq!(iter.next().unwrap().as_str().unwrap(), "trailing comma");
/// assert_eq!(iter.next(), None);
/// ```
#[derive(Debug, Clone, PartialEq)]
// if JsoncValue<'a, I, F>, cannot implement FromStr
pub enum JsoncValue<I, F> {
    /// Represents any valid JSON with comments object.
    /// Default implementation is `HashMap`. If `preserve_order` feature is enabled, that will be `IndexMap`.
    /// ```
    /// let v = json_with_comments::jsonc!({"key": "value"});
    /// ```
    Object(MapImpl<String, JsoncValue<I, F>>),

    /// Represents any valid JSON with comments array.
    /// ```
    /// let v = json_with_comments::jsonc!([1, 2, 3]);
    /// ```
    Array(Vec<JsoncValue<I, F>>),

    /// Represents any valid JSON with comments boolean.
    /// ```
    /// let v = json_with_comments::jsonc!(true);
    /// ```
    Bool(bool),

    /// Represents any valid JSON with comments null.
    /// ```
    /// let v = json_with_comments::jsonc!(null);
    /// ```
    Null,

    /// Represents any valid JSON with comments string.
    /// ```
    /// let v = json_with_comments::jsonc!("string");
    /// ```
    String(String),

    /// Represents any valid JSON with comments number, whether integer or float.
    /// ```
    /// let v = json_with_comments::jsonc!(123.45);
    /// ```
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
    /// - If you want to replace the values of two variables, see [`Self::swap`].
    /// - If you want to replace with a passed value instead of the default value, see [`Self::replace`].
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

    /// Swaps `self` value and `other` value.
    /// - If you want to swap with a default or dummy value, see [`Self::take`].
    /// - If you want to replace with a passed value instead of the default value, see [`Self::replace`].
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
    /// let mut lower = "jsonc".into();
    /// let name = value["keywords"][1].swap(&mut lower);
    /// assert_eq!(lower, "JSONC".into());
    /// assert_eq!(value, jsonc!({
    ///     "name": "json-with-comments",
    ///     "keywords": [
    ///         "JSON with comments",
    ///         "jsonc",
    ///         "trailing comma"
    ///     ]
    /// }));
    /// ```
    pub fn swap(&mut self, other: &mut Self) {
        std::mem::swap(self, other)
    }

    /// Moves `other` value into `self` value, returning the previous `self` value.
    /// - If you want to swap with a default or dummy value, see [`Self::take`].
    /// - If you want to replace the values of two variables, see [`Self::swap`].
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
    /// let upper = "JSON WITH COMMENTS".into();
    /// let original = value["keywords"][0].replace(upper);
    /// assert_eq!(original, "JSON with comments".into());
    /// assert_eq!(value, jsonc!({
    ///     "name": "json-with-comments",
    ///     "keywords": [
    ///         "JSON WITH COMMENTS",
    ///         "JSONC",
    ///         "trailing comma"
    ///     ]
    /// }));
    /// ```
    pub fn replace(&mut self, other: Self) -> Self {
        std::mem::replace(self, other)
    }

    /// Get the value type representation of [`JsoncValue`].
    /// Main use case is for error reporting.
    ///
    /// # Examples
    /// ```
    /// use json_with_comments::jsonc;
    /// assert_eq!(jsonc!({"key": "value"}).value_type(), "Object");
    /// assert_eq!(jsonc!([1, 2, 3]).value_type(), "Array");
    /// assert_eq!(jsonc!(true).value_type(), "Boolean");
    /// assert_eq!(jsonc!(null).value_type(), "Null");
    /// assert_eq!(jsonc!("string").value_type(), "String");
    /// assert_eq!(jsonc!(123).value_type(), "Number");
    /// assert_eq!(jsonc!(123.45).value_type(), "Number");
    /// ```
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
