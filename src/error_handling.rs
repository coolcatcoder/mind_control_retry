use std::{error::Error, fmt::Debug};

use bevy::{ecs::error::ErrorContext, prelude::*};

pub fn error_handler(error: BevyError, context: ErrorContext) {
    if let Some(failure) = error.downcast_ref::<Failure>() {
        failure.handler(context);
    } else {
        panic!(
            "Encountered an error in {} `{}`: {}",
            context.kind(),
            context.name(),
            error
        );
    }
}

#[derive(Debug)]
pub enum Failure {
    Return,
    Warn(String),
    Error(String),
}

impl Failure {
    fn handler(&self, context: ErrorContext) {
        match self {
            Self::Return => debug!("Early return in {} `{}`.", context.kind(), context.name()),
            Self::Warn(warn) => warn!(
                "Warning in {} `{}`: {}",
                context.kind(),
                context.name(),
                warn
            ),
            Self::Error(error) => error!(
                "Error in {} `{}`: {}",
                context.kind(),
                context.name(),
                error
            ),
        }
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Return => write!(f, "Return"),
            Self::Warn(warn) => write!(f, "Warn: {warn}"),
            Self::Error(error) => write!(f, "Error: {error}"),
        }
    }
}

impl Error for Failure {}

pub trait ToFailure {
    type Inner;

    fn else_return(self) -> Result<Self::Inner, Failure>;
    fn else_error(self, error: impl ToString) -> Result<Self::Inner, Failure>;
}

impl<T> ToFailure for Option<T> {
    type Inner = T;

    fn else_return(self) -> Result<Self::Inner, Failure> {
        self.ok_or(Failure::Return)
    }
    fn else_error(self, error: impl ToString) -> Result<Self::Inner, Failure> {
        self.ok_or(Failure::Error(error.to_string()))
    }
}

impl<T, E: Debug> ToFailure for Result<T, E> {
    type Inner = T;

    fn else_return(self) -> Result<Self::Inner, Failure> {
        match self {
            Ok(value) => Ok(value),
            Err(_) => Err(Failure::Return),
        }
    }
    fn else_error(self, error: impl ToString) -> Result<Self::Inner, Failure> {
        match self {
            Ok(value) => Ok(value),
            Err(result_error) => Err(Failure::Error(format!(
                "{}\n{:?}",
                error.to_string(),
                result_error
            ))),
        }
    }
}
