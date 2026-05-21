// 统一错误类型；IPC 返回时序列化为 { code, message }。
use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Not implemented yet: {0}")]
    NotImplemented(&'static str),

    #[error("Unknown: {0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("AppError", 2)?;
        let code = match self {
            AppError::Io(_) => "io",
            AppError::Serde(_) => "serde",
            AppError::NotImplemented(_) => "not_implemented",
            AppError::Other(_) => "other",
        };
        st.serialize_field("code", code)?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
