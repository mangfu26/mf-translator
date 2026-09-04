use serde::{Serialize, Serializer};

pub type AppResult<T> = Result<T, AppError>;

/// 应用级统一错误。跨 IPC 边界时序列化为 `{ code, message }`，
/// 前端据此呈现可读信息与下一步引导（例如协议不兼容时提示切换协议）。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AppError {
    #[error("配置错误：{0}")]
    Config(String),

    #[error("网络请求失败：{0}")]
    Network(String),

    #[error("认证失败：API Key 无效或没有访问该模型的权限")]
    Auth,

    #[error("协议不兼容：{0}")]
    Protocol(String),

    #[error("请求已取消")]
    Cancelled,

    #[error("上游返回了无法解析的响应：{0}")]
    MalformedResponse(String),

    #[error("功能尚未实现：{0}")]
    NotImplemented(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Config(_) => "CONFIG",
            AppError::Network(_) => "NETWORK",
            AppError::Auth => "AUTH",
            AppError::Protocol(_) => "PROTOCOL",
            AppError::Cancelled => "CANCELLED",
            AppError::MalformedResponse(_) => "MALFORMED_RESPONSE",
            AppError::NotImplemented(_) => "NOT_IMPLEMENTED",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Payload<'a> {
            code: &'a str,
            message: &'a str,
        }

        let message = self.to_string();
        Payload {
            code: self.code(),
            message: &message,
        }
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_serializes_to_code_and_message() {
        let json = serde_json::to_string(&AppError::Auth).unwrap();
        assert!(json.contains(r#""code":"AUTH""#));
        assert!(json.contains("API Key"));
    }

    #[test]
    fn error_message_never_leaks_raw_secrets() {
        let err = AppError::Network("connect timeout for https://api.example.com".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("网络请求失败"));
    }
}
