//! Extra error handling helpers
#![feature(backtrace)]
#![warn(missing_docs)]

use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::error::Error;
use std::fmt::Display;

pub trait ErrTools<'a>: Error {
    type Serialize;

    fn serialize(&'a self) -> Self::Serialize;
}

pub trait WrapErr<T, E, E2> {
    /// Wrap the error value with a new adhoc error
    fn wrap_err<D>(self, msg: D) -> Result<T, E2>
    where
        D: Display + Send + Sync + 'static,
        E2: From<(E, String)>;

    /// Wrap the error value with a new adhoc error that is evaluated lazily
    /// only once an error does occur.
    fn wrap_err_with<D, F>(self, f: F) -> Result<T, E2>
    where
        D: Display + Send + Sync + 'static,
        E2: From<(E, String)>,
        F: FnOnce() -> D;
}

impl<T, E, E2> WrapErr<T, E, E2> for Result<T, E> {
    fn wrap_err<D>(self, msg: D) -> Result<T, E2>
    where
        D: Display + Send + Sync + 'static,
        E2: From<(E, String)>,
    {
        self.map_err(|source| E2::from((source, format!("{}", msg))))
    }

    fn wrap_err_with<D, F>(self, msg: F) -> Result<T, E2>
    where
        D: Display + Send + Sync + 'static,
        E2: From<(E, String)>,
        F: FnOnce() -> D,
    {
        self.map_err(|source| E2::from((source, format!("{}", msg()))))
    }
}

impl<'a, E> ErrTools<'a> for E
where
    E: Error + Sized + 'static,
{
    type Serialize = SerializeableConcreteError<'a, E>;

    fn serialize(&'a self) -> Self::Serialize {
        SerializeableConcreteError(self)
    }
}

impl<'a> ErrTools<'a> for dyn Error + 'static {
    type Serialize = SerializeableError<'a>;

    fn serialize(&'a self) -> Self::Serialize {
        SerializeableError(self)
    }
}

impl<'a> ErrTools<'a> for dyn Error + Send + Sync + 'static {
    type Serialize = SerializeableError<'a>;

    fn serialize(&'a self) -> Self::Serialize {
        SerializeableError(self)
    }
}

pub struct SerializeableError<'a>(&'a dyn Error);
pub struct SerializeableConcreteError<'a, E>(&'a E)
where
    E: Error + Sized + 'static;

impl Serialize for SerializeableError<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut e = serializer.serialize_struct("error", 3)?;
        let msg = self.0.to_string();
        e.serialize_field("msg", &msg)?;
        e.serialize_field("backtrace", &self.0.backtrace().map(ToString::to_string))?;
        e.serialize_field("source", &self.0.source().map(ErrTools::serialize))?;
        e.end()
    }
}

impl<E> Serialize for SerializeableConcreteError<'_, E>
where
    E: Error + Sized + 'static,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut e = serializer.serialize_struct("error", 4)?;
        let msg = self.0.to_string();
        e.serialize_field("type", &std::any::type_name::<E>())?;
        e.serialize_field("msg", &msg)?;
        e.serialize_field("backtrace", &self.0.backtrace().map(ToString::to_string))?;
        e.serialize_field("source", &self.0.source().map(ErrTools::serialize))?;
        e.end()
    }
}
