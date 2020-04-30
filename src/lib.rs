//! Extra error handling helpers
#![feature(backtrace)]
#![warn(missing_docs)]

use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::error::Error;
use std::fmt::Display;

pub trait ErrTools<'a>: Error {
    type Serialize;

    fn serialize(&'a self) -> Self::Serialize;

    fn downcast_refchain<T: Error + Sized + 'static>(&self) -> Option<&T>;
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

    fn downcast_refchain<T: Error + Sized + 'static>(&self) -> Option<&T> {
        let mut cur_error = Some(self as &dyn Error);

        while let Some(error) = cur_error {
            if let Some(error) = error.downcast_ref() {
                return Some(error);
            }

            cur_error = error.source();
        }

        None
    }
}

impl<'a> ErrTools<'a> for dyn Error + 'static {
    type Serialize = SerializeableError<'a>;

    fn serialize(&'a self) -> Self::Serialize {
        SerializeableError(self)
    }

    fn downcast_refchain<T: Error + Sized + 'static>(&self) -> Option<&T> {
        let mut cur_error = Some(self as &dyn Error);

        while let Some(error) = cur_error {
            if let Some(error) = error.downcast_ref() {
                return Some(error);
            }

            cur_error = error.source();
        }

        None
    }
}

impl<'a> ErrTools<'a> for dyn Error + Send + Sync + 'static {
    type Serialize = SerializeableError<'a>;

    fn serialize(&'a self) -> Self::Serialize {
        SerializeableError(self)
    }

    fn downcast_refchain<T: Error + Sized + 'static>(&self) -> Option<&T> {
        let mut cur_error = Some(self as &dyn Error);

        while let Some(error) = cur_error {
            if let Some(error) = error.downcast_ref() {
                return Some(error);
            }

            cur_error = error.source();
        }

        None
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

#[cfg(test)]
mod tests {
    use super::*;
    use displaydoc::Display;
    use std::error::Error;
    use std::rc::Rc;

    /// Fake Error
    #[derive(Debug, Display)]
    struct E1;

    impl std::error::Error for E1 {}

    /// Fake Error 2
    #[derive(Debug, Display)]
    struct E2(E1);

    impl std::error::Error for E2 {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.0)
        }
    }

    /// Non Send Error
    #[derive(Debug, Display)]
    struct E3(Rc<()>);

    impl std::error::Error for E3 {}

    #[test]
    fn downcast_refchain_test() {
        let e: &dyn Error = &E2(E1);

        assert!(matches!(e.downcast_refchain::<E1>(), Some(&E1)));
        assert!(matches!(e.downcast_refchain::<E2>(), Some(&E2(_))));
        assert!(matches!(e.downcast_refchain::<std::io::Error>(), None));
    }

    #[test]
    fn downcast_concrete_refchain_test() {
        let e = E2(E1);

        assert!(matches!(e.downcast_refchain::<E1>(), Some(&E1)));
        assert!(matches!(e.downcast_refchain::<E2>(), Some(&E2(_))));
        assert!(matches!(e.downcast_refchain::<std::io::Error>(), None));
    }

    #[test]
    fn downcast_nonsend_refchain_test() {
        let e = E3(Rc::new(()));

        assert!(matches!(e.downcast_refchain::<E3>(), Some(&E3(_))));
        assert!(matches!(e.downcast_refchain::<std::io::Error>(), None));
    }
}
