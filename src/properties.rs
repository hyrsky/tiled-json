use std::collections::HashMap;
use std::fmt;

use serde::{de, Deserialize, Deserializer};

use crate::Color;

#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type", content = "value")]
pub enum Property {
	Bool(bool),
	Float(f32),
	Int(i32),
	Color(Color),
	String(String),
	File(String),
}

pub type Properties = HashMap<String, Property>;

/// Helper struct
#[derive(Debug, PartialEq, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
struct PropertyValue {
	name: String,
	#[serde(flatten)]
	value: Property,
}

struct PropertiesVisitor;

impl<'de> de::Visitor<'de> for PropertiesVisitor {
	type Value = Properties;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("properties array")
	}

	fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
	where
		S: de::SeqAccess<'de>,
	{
		let mut map = Properties::with_capacity(seq.size_hint().unwrap_or(0));

		// First deserialize array items to PropertyValue.
		while let Some(value) = (seq.next_element() as Result<Option<PropertyValue>, _>)? {
			// Then add Property to hashmap.
			map.insert(value.name, value.value);
		}

		Ok(map)
	}
}

pub fn deserialize_properties<'de, D>(deserializer: D) -> Result<Option<Properties>, D::Error>
where
	D: Deserializer<'de>,
{
	if let Ok(properties) = deserializer.deserialize_seq(PropertiesVisitor) {
		Ok(Some(properties))
	} else {
		Ok(None)
	}
}
