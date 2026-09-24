use std::fmt;

/// A value which can be sent as a PostgreSQL parameter.
///
/// This is a semantic value representation, not PostgreSQL wire encoding.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => formatter.write_str("NULL"),
            Self::Bool(value) => value.fmt(formatter),
            Self::Int(value) => value.fmt(formatter),
            Self::Float(value) => value.fmt(formatter),
            Self::Text(value) => value.fmt(formatter),
        }
    }
}

/// Converts a Rust value into the value accepted by a typed column.
mod sealed {
    pub trait Sealed {}
}

pub trait ColumnValue<T>: sealed::Sealed {
    fn into_value(self) -> Value;
}

impl sealed::Sealed for bool {}

impl ColumnValue<bool> for bool {
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}

macro_rules! signed_value {
    ($($type:ty),+ $(,)?) => {
        $(
            impl sealed::Sealed for $type {}

            impl ColumnValue<$type> for $type {
                fn into_value(self) -> Value {
                    Value::Int(self as i64)
                }
            }
        )+
    };
}

signed_value!(i8, i16, i32, i64, isize);

impl sealed::Sealed for f32 {}

impl ColumnValue<f32> for f32 {
    fn into_value(self) -> Value {
        Value::Float(self as f64)
    }
}

impl sealed::Sealed for f64 {}

impl ColumnValue<f64> for f64 {
    fn into_value(self) -> Value {
        Value::Float(self)
    }
}

impl sealed::Sealed for String {}

impl ColumnValue<String> for String {
    fn into_value(self) -> Value {
        Value::Text(self)
    }
}

impl sealed::Sealed for &str {}

impl ColumnValue<String> for &str {
    fn into_value(self) -> Value {
        Value::Text(self.to_owned())
    }
}
