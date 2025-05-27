use crate::utils::error::Result;
use serde::Serialize;

/// Print data as JSON
pub fn print_json<T: Serialize>(data: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}

/// Convert data to JSON string
pub fn to_json_string<T: Serialize>(data: &T) -> Result<String> {
    serde_json::to_string_pretty(data).map_err(Into::into)
}
