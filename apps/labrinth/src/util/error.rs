use std::{
    convert::Infallible,
    fmt::{Debug, Display},
};

use crate::routes::ApiError;

pub trait Context<T, E>: Sized {
    fn wrap_request_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static;

    fn wrap_request_err<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static,
    {
        self.wrap_request_err_with(|| msg)
    }

    fn wrap_internal_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static;

    fn wrap_internal_err<D>(self, msg: D) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static,
    {
        self.wrap_internal_err_with(|| msg)
    }
}

impl<T, E> Context<T, E> for Result<T, E>
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

impl<T> Context<T, Infallible> for Option<T> {
    fn wrap_request_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| ApiError::Request(eyre::Report::msg(f())))
    }

    fn wrap_internal_err_with<D>(
        self,
        f: impl FnOnce() -> D,
    ) -> Result<T, ApiError>
    where
        D: Debug + Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| ApiError::Internal(eyre::Report::msg(f())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sqlx_result() -> Result<(), sqlx::Error> {
        Err(sqlx::Error::RowNotFound)
    }

    // these just test that code written with the above API compiles
    fn propagating() -> Result<(), ApiError> {
        sqlx_result()
            .wrap_internal_err("failed to perform database operation")?;
        sqlx_result().wrap_request_err("invalid request parameter")?;

        None::<()>.wrap_internal_err("something is missing")?;

        Ok(())
    }

    // just so we don't get a dead code warning
    #[test]
    fn test_propagating() {
        _ = propagating();
    }
}
