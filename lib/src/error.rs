use std::borrow::Cow;

use exn::{Exn, OptionExt as _, ResultExt as _};

macro_rules! error_type {
    ($name:ident) => {
        #[derive(Debug, derive_more::Display, derive_more::From)]
        pub struct $name(std::borrow::Cow<'static, str>);
        impl std::error::Error for $name {}
    };
}
pub(crate) use error_type;

pub trait ExnExt<TData> {
    #[track_caller]
    fn or_raise_str<TNewErr, TStr, Cb>(self, cb: Cb) -> exn::Result<TData, TNewErr>
    where
        TNewErr: std::error::Error + Send + Sync + From<Cow<'static, str>>,
        TStr: Into<Cow<'static, str>>,
        Cb: FnOnce() -> TStr;
}

pub trait ResultExt<TData> {
    #[track_caller]
    fn or_raise_str<TNewErr, TStr, Cb>(self, cb: Cb) -> exn::Result<TData, TNewErr>
    where
        TNewErr: std::error::Error + Send + Sync + From<Cow<'static, str>>,
        TStr: Into<Cow<'static, str>>,
        Cb: FnOnce() -> TStr;
}

pub trait OptionExt<TData> {
    #[track_caller]
    fn ok_or_raise_str<TErr, TStr, Cb>(self, cb: Cb) -> exn::Result<TData, TErr>
    where
        TErr: std::error::Error + Send + Sync + From<Cow<'static, str>>,
        TStr: Into<Cow<'static, str>>,
        Cb: FnOnce() -> TStr;
}

impl<TData, TErr> ExnExt<TData> for Result<TData, Exn<TErr>>
where
    TErr: std::error::Error + Send + Sync,
{
    #[track_caller]
    fn or_raise_str<TNewErr, TStr, Cb>(self, cb: Cb) -> exn::Result<TData, TNewErr>
    where
        TNewErr: std::error::Error + Send + Sync + From<Cow<'static, str>>,
        TStr: Into<Cow<'static, str>>,
        Cb: FnOnce() -> TStr,
    {
        self.or_raise(|| TNewErr::from(cb().into()))
    }
}

impl<TData, TErr> ResultExt<TData> for Result<TData, TErr>
where
    TErr: std::error::Error + Send + Sync + 'static,
{
    #[track_caller]
    fn or_raise_str<TNewErr, TStr, Cb>(self, cb: Cb) -> exn::Result<TData, TNewErr>
    where
        TNewErr: std::error::Error + Send + Sync + From<Cow<'static, str>>,
        TStr: Into<Cow<'static, str>>,
        Cb: FnOnce() -> TStr,
    {
        self.or_raise(|| TNewErr::from(cb().into()))
    }
}

impl<TData> OptionExt<TData> for Option<TData> {
    #[track_caller]
    fn ok_or_raise_str<TErr, TStr, Cb>(self, cb: Cb) -> exn::Result<TData, TErr>
    where
        TErr: std::error::Error + Send + Sync + From<Cow<'static, str>>,
        TStr: Into<Cow<'static, str>>,
        Cb: FnOnce() -> TStr,
    {
        self.ok_or_raise(|| TErr::from(cb().into()))
    }
}

macro_rules! bail_str {
    ($msg:literal) => {
        return Err(exn::Exn::new(<std::borrow::Cow<'static, str>>::from($msg).into()))
    };
    ($($arg:tt)*) => {
        return Err(exn::Exn::new(<std::borrow::Cow<'static, str>>::from(format!($($arg)*)).into()))
    }
}
pub(crate) use bail_str;

macro_rules! ensure_str {
    ($cond:expr, $msg:literal) => {
        if !bool::from($cond) {
            return Err(exn::Exn::new(<std::borrow::Cow<'static, str>>::from($msg).into()))
        }
    };
    ($cond:expr, $($arg:tt)*) => {
        if !bool::from($cond) {
            return Err(exn::Exn::new(<std::borrow::Cow<'static, str>>::from(format!($($arg)*)).into()))
        }
    }
}
pub(crate) use ensure_str;
