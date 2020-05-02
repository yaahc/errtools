//! Types that support deserialization that mirrors `ErrTools::serialize`
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use std::fmt;

const FIELDS: &'static [&'static str] = &["type_name", "msg", "source"];

#[derive(Debug)]
///
pub struct Error {
    type_name: Option<String>,
    msg: String,
    source: Option<Box<SourceError>>,
}

struct ErrorVisitor;

#[derive(Debug)]
struct SourceError {
    msg: String,
    source: Option<Box<SourceError>>,
}

struct SourceErrorVisitor;

enum Field {
    TypeName,
    Msg,
    Source,
}

impl Field {
    fn as_str(&self) -> &'static str {
        match self {
            Self::TypeName => FIELDS[0],
            Self::Msg => FIELDS[1],
            Self::Source => FIELDS[2],
        }
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Error", FIELDS, ErrorVisitor)
    }
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("`secs` or `nanos`")
            }

            fn visit_str<E>(self, value: &str) -> Result<Field, E>
            where
                E: de::Error,
            {
                match value {
                    "type_name" => Ok(Field::TypeName),
                    "msg" => Ok(Field::Msg),
                    "source" => Ok(Field::Source),
                    _ => Err(de::Error::unknown_field(value, FIELDS)),
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)
    }
}

impl<'de> Deserialize<'de> for SourceError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Error", &FIELDS[1..], SourceErrorVisitor)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.msg.as_str())
    }
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.msg.as_str())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref().map(|s| s as _)
    }
}

impl std::error::Error for SourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_deref().map(|s| s as _)
    }
}

impl<'de> Visitor<'de> for ErrorVisitor {
    type Value = Error;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct errtools::deserialize::Error")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Error, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let type_name = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let msg = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let source = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;

        Ok(Error {
            type_name,
            msg,
            source,
        })
    }

    fn visit_map<V>(self, mut map: V) -> Result<Error, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut type_name = None;
        let mut msg = None;
        let mut source = None;
        while let Some(key) = map.next_key()? {
            match key {
                Field::TypeName => {
                    if type_name.is_some() {
                        return Err(de::Error::duplicate_field("type_name"));
                    }
                    type_name = Some(map.next_value()?);
                }
                Field::Msg => {
                    if msg.is_some() {
                        return Err(de::Error::duplicate_field("msg"));
                    }
                    msg = Some(map.next_value()?);
                }
                Field::Source => {
                    if source.is_some() {
                        return Err(de::Error::duplicate_field("source"));
                    }
                    source = Some(map.next_value()?);
                }
            }
        }

        let msg = msg.ok_or_else(|| de::Error::missing_field("msg"))?;
        let source = source.ok_or_else(|| de::Error::missing_field("source"))?;

        Ok(Error {
            type_name,
            msg,
            source,
        })
    }
}

impl<'de> Visitor<'de> for SourceErrorVisitor {
    type Value = SourceError;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct errtools::deserialize::Error")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<SourceError, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let msg = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let source = seq
            .next_element()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;

        Ok(SourceError { msg, source })
    }

    fn visit_map<V>(self, mut map: V) -> Result<SourceError, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut msg = None;
        let mut source = None;
        while let Some(key) = map.next_key()? {
            match key {
                Field::Msg => {
                    if msg.is_some() {
                        return Err(de::Error::duplicate_field("msg"));
                    }
                    msg = Some(map.next_value()?);
                }
                Field::Source => {
                    if source.is_some() {
                        return Err(de::Error::duplicate_field("source"));
                    }
                    source = Some(map.next_value()?);
                }
                _ => Err(de::Error::unknown_field(key.as_str(), &FIELDS[1..]))?,
            }
        }

        let msg = msg.ok_or_else(|| de::Error::missing_field("msg"))?;
        let source = source.ok_or_else(|| de::Error::missing_field("source"))?;

        Ok(SourceError { msg, source })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use eyre::{eyre, Report};
    use std::error::Error;
    use std::fmt::Write;

    #[test]
    fn serialize_eyre() {
        let err: Report = eyre!("root cause")
            .wrap_err("second error")
            .wrap_err("outermost error");
        let json = serde_json::to_string_pretty(&err.serialize()).unwrap();
        println!("{}", json);
    }

    #[test]
    fn serialize_anyhow() {
        let err = anyhow::anyhow!("root cause")
            .context("second error")
            .context("outermost error");
        let json = serde_json::to_string_pretty(&err.serialize()).unwrap();
        println!("{}", json);
    }

    #[test]
    fn serialize_concrete() {
        let err = super::Error {
            type_name: Some("FakeError".into()),
            msg: "outer error".into(),
            source: Some(Box::new(SourceError {
                msg: "root cause".into(),
                source: None,
            })),
        };
        let json = serde_json::to_string_pretty(&err.serialize()).unwrap();

        println!("concrete serialization:\n{}\n", json);
    }

    #[test]
    fn serialize_dyn() {
        let err = super::Error {
            type_name: Some("FakeError".into()),
            msg: "outer error".into(),
            source: Some(Box::new(SourceError {
                msg: "root cause".into(),
                source: None,
            })),
        };
        let err: &dyn Error = &err;
        let json = serde_json::to_string_pretty(&err.serialize()).unwrap();

        println!("dyn serialization:\n{}", json);
    }

    fn report(error: &dyn Error) -> String {
        let mut out = String::new();
        let mut cur_error = Some(error);

        while let Some(error) = cur_error {
            writeln!(out, "{}", error).unwrap();
            cur_error = error.source();
        }

        out
    }

    #[test]
    fn deserialize_concrete() {
        let err = super::Error {
            type_name: Some("FakeError".into()),
            msg: "outer error".into(),
            source: Some(Box::new(SourceError {
                msg: "root cause".into(),
                source: None,
            })),
        };
        let json = serde_json::to_string_pretty(&err.serialize()).unwrap();
        let report: Report = err.into();
        let display = report.to_string();

        let err_out: deserialize::Error = serde_json::from_str(&json).unwrap();
        let report: Report = err_out.into();
        let deserialized_display = report.to_string();

        assert_eq!(display, deserialized_display);
    }

    #[test]
    fn deserialize_dyn() {
        let err = super::Error {
            type_name: Some("FakeError".into()),
            msg: "outer error".into(),
            source: Some(Box::new(SourceError {
                msg: "root cause".into(),
                source: None,
            })),
        };

        let err: &(dyn Error + Send + Sync + 'static) = &err;
        let json = serde_json::to_string_pretty(&err.serialize()).unwrap();
        let display = report(err);

        let err_out: deserialize::Error = serde_json::from_str(&json).unwrap();
        let err_out = &err_out as &dyn Error;
        let deserialized_display = report(err_out);

        assert_eq!(display, deserialized_display);
    }

    #[test]
    fn deserialize_concrete_bincode() {
        let err = super::Error {
            type_name: Some("FakeError".into()),
            msg: "outer error".into(),
            source: Some(Box::new(SourceError {
                msg: "root cause".into(),
                source: None,
            })),
        };
        let buf = bincode::serialize(&err.serialize()).unwrap();
        let report: Report = err.into();
        let display = report.to_string();

        let err_out: deserialize::Error = bincode::deserialize(&buf).unwrap();
        let report: Report = err_out.into();
        let deserialized_display = report.to_string();

        assert_eq!(display, deserialized_display);
    }

    #[test]
    fn deserialize_dyn_bincode() {
        let err = super::Error {
            type_name: Some("FakeError".into()),
            msg: "outer error".into(),
            source: Some(Box::new(SourceError {
                msg: "root cause".into(),
                source: None,
            })),
        };

        let err: &(dyn Error + Send + Sync + 'static) = &err;
        let buf = bincode::serialize(&err.serialize()).unwrap();
        let display = report(err);

        let err_out: deserialize::Error = bincode::deserialize(&buf).unwrap();
        let err_out = &err_out as &dyn Error;
        let deserialized_display = report(err_out);

        assert_eq!(display, deserialized_display);
    }
}
