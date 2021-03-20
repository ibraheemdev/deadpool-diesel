pub use rt::*;

#[cfg(all(feature = "rt-tokio", not(any(feature = "rt-async-std"))))]
mod rt {
    pub type JoinError = tokio::task::JoinError;

    pub async fn spawn_blocking<F, T>(f: F) -> Result<T, JoinError>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        tokio::task::spawn_blocking(f).await
    }
}

#[cfg(all(feature = "rt-async-std", not(any(feature = "rt-tokio"))))]
mod rt {
    crate::__unreachable_join_error!();

    pub async fn spawn_blocking<F, T>(f: F) -> Result<T, JoinError>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        Ok(async_std::task::spawn_blocking(f).await)
    }
}

#[cfg(not(any(feature = "rt-tokio", feature = "rt-async-std")))]
mod rt {
    compile_error!("one of 'rt-tokio', or 'rt-async-std' features must be enabled");

    crate::__unreachable_join_error!();

    pub async fn spawn_blocking<F, T>(f: F) -> Result<T, JoinError> {
        unreachable!()
    }
}

#[cfg(all(feature = "rt-tokio", feature = "rt-async-std"))]
mod rt {
    compile_error!("only one of 'rt-tokio', or 'rt-async-std' features can be enabled");

    crate::__unreachable_join_error!();

    pub async fn spawn_blocking<F, T>(f: F) -> Result<T, JoinError> {
        unreachable!()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __unreachable_join_error {
    () => {
        #[derive(Debug)]
        pub struct JoinError;

        impl ::std::fmt::Display for JoinError {
            fn fmt(&self, _: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                unreachable!()
            }
        }
    };
}
