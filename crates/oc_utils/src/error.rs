pub trait OkOrLogError {
    fn ok_or_log(&self);
}

#[macro_export]
macro_rules! unwrap_or_log {
    ($expr:expr, $prefix:expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => {
                tracing::error!("{}: {}", $prefix, error);
                return;
            }
        }
    };
    ($expr:expr, $prefix:expr, $default:expr) => {
        match $expr {
            Ok(value) => value,
            Err(error) => {
                tracing::error!("{}: {}", $prefix, error);
                $default
            }
        }
    };
}

impl<T, E: std::fmt::Display> OkOrLogError for std::result::Result<T, E> {
    fn ok_or_log(&self) {
        if let Err(error) = self {
            tracing::error!("Error: {}", error.to_string())
        }
    }
}
