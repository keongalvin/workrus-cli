use std::fmt;

/// Stable application error categories and their process exit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Input,
    Operational,
}

impl ErrorKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Input => "invalid_input",
            Self::Operational => "operational",
        }
    }

    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Input => 2,
            Self::Operational => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppError {
    pub kind: ErrorKind,
    pub message: String,
    pub partial_result: Option<String>,
}

impl AppError {
    pub fn input(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Input,
            message: message.into(),
            partial_result: None,
        }
    }

    pub fn operational(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Operational,
            message: message.into(),
            partial_result: None,
        }
    }

    pub fn partial(message: impl Into<String>, result: String) -> Self {
        Self {
            kind: ErrorKind::Operational,
            message: message.into(),
            partial_result: Some(result),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for AppError {}
