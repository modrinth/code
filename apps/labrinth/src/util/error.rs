use std::fmt::{Debug, Display};

use crate::routes::ApiError;

pub trait WrapErr<T, E>: Sized {
    fn wrap_request_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Display + Send + Sync + 'static;

    fn wrap_request_err<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Display + Send + Sync + 'static,
    {
        self.wrap_request_err_with(|| msg)
    }

    fn wrap_internal_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Display + Send + Sync + 'static;

    fn wrap_internal_err<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Display + Send + Sync + 'static,
    {
        self.wrap_internal_err_with(|| msg)
    }
}

impl<T, E> WrapErr<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + Sized + 'static,
{
    fn wrap_request_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            let report = eyre::Report::new(err).wrap_err(f());
            ApiError::Request(report)
        })
    }

    fn wrap_internal_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|err| {
            let report = eyre::Report::new(err).wrap_err(f());
            ApiError::Internal(report)
        })
    }
}

pub trait OptionExt<T>: Sized {
    fn ok_or_request_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static;

    fn ok_or_request_err<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static,
    {
        self.ok_or_request_err_with(|| msg)
    }

    fn ok_or_internal_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static;

    fn ok_or_internal_err<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static,
    {
        self.ok_or_internal_err_with(|| msg)
    }
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_request_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| {
            let report = eyre::Report::msg(f());
            ApiError::Request(report)
        })
    }

    fn ok_or_internal_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| {
            let report = eyre::Report::msg(f());
            ApiError::Internal(report)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sqlx_result() -> Result<(), sqlx::Error> {
        Err(sqlx::Error::RowNotFound)
    }

    // these just test that code written with the above API compiles
    #[test]
    fn wrap() -> Result<(), ApiError> {
        sqlx_result()
            .wrap_internal_err("failed to perform database operation")?;
        sqlx_result().wrap_request_err("invalid request parameter")?;
        Ok(())
    }
}
