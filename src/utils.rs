pub mod rug_integer {
    use rug::Integer;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Serialisation function for big ints
    pub fn serialize<S>(x: &Integer, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value: String = x.to_string_radix(16);
        value.serialize(s)
    }

    /// Deserialisation function for big ints
    pub fn deserialize<'de, D>(d: D) -> Result<Integer, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(d)?;
        Integer::from_str_radix(&value, 16).map_err(serde::de::Error::custom)
    }
}