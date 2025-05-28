use crate::utils::error::Result;
use serde::Serialize;
use std::io::{self, Write}; // 新增导入

/// Print data as JSON
pub fn print_json<T: Serialize>(data: &T) -> Result<()> {
    match serde_json::to_string_pretty(data) {
        Ok(json) => {
            let mut stdout = io::stdout(); // 获取 stdout 句柄
            if let Err(e) = writeln!(stdout, "{}", json) {
                // 使用 writeln!
                // 打印失败，返回错误
                return Err(crate::utils::error::CliError::Io(e).into());
            }
            Ok(())
        }
        Err(e) => {
            // 序列化失败，返回错误
            Err(e.into()) // CliError::Serialization
        }
    }
}

/// Convert data to JSON string
pub fn to_json_string<T: Serialize>(data: &T) -> Result<String> {
    serde_json::to_string_pretty(data).map_err(Into::into)
}
