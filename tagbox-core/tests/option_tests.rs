use tagbox_core::errors::TagboxError;
use tagbox_core::utils::require_field;

#[test]
fn test_require_field_some() {
    let v = require_field(Some(42), "num").unwrap();
    assert_eq!(v, 42);
}

#[test]
fn test_require_field_none() {
    let err = require_field::<i32>(None, "num").unwrap_err();
    match err {
        TagboxError::MissingField { field } => assert_eq!(field, "num"),
        _ => panic!("unexpected error"),
    }
}
