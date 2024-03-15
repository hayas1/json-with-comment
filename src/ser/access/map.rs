use serde::ser;

use crate::ser::formatter::JsoncFormatter;

use super::jsonc::JsoncSerializer;

pub struct MapSerializer<'a, W, F>
where
    F: JsoncFormatter,
{
    serializer: &'a mut JsoncSerializer<W, F>,
    index: usize,
    len: Option<usize>,
}

impl<'a, W, F> MapSerializer<'a, W, F>
where
    W: std::io::Write,
    F: JsoncFormatter,
{
    pub fn start(serializer: &'a mut JsoncSerializer<W, F>, len: Option<usize>) -> crate::Result<Self> {
        serializer.formatter.write_object_start(&mut serializer.write)?;
        Ok(MapSerializer { serializer, index: 0, len })
    }
}

impl<'a, W, F> ser::SerializeMap for MapSerializer<'a, W, F>
where
    W: std::io::Write,
    F: JsoncFormatter,
{
    type Ok = ();
    type Error = crate::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        self.serializer.formatter.write_object_key_start(&mut self.serializer.write, self.index, self.len)?;
        key.serialize(&mut *self.serializer)?;
        self.serializer.formatter.write_object_key_end(&mut self.serializer.write, self.index, self.len)?;
        Ok(self.index += 1)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ser::Serialize,
    {
        self.serializer.formatter.write_object_value_start(&mut self.serializer.write, self.index, self.len)?;
        value.serialize(&mut *self.serializer)?;
        self.serializer.formatter.write_object_value_end(&mut self.serializer.write, self.index, self.len)?;
        Ok(self.index += 1)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.formatter.write_object_end(&mut self.serializer.write)
    }
}
