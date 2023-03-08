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

pub mod unicorn_selection {
    use crate::unicorn::UnicornInfo;
    use crate::fortuna::Fortuna;

    pub fn get_unicorn_prn(unicorn: &UnicornInfo, usage_number: u128) -> u64 {
        let prn_seed: [u8; 32] = unicorn.g_value.as_bytes()[..32]
            .try_into()
            .unwrap();

        let mut csprng = Fortuna::new(&prn_seed, usage_number).unwrap();

        let val = csprng.get_bytes(8).unwrap();
        u64::from_be_bytes(val[0..8].try_into().unwrap())
    }
}