use super::IJsonUInt;
use crate::DefinitionError;

#[test]
fn accepts_zero() -> Result<(), DefinitionError> {
    let value = IJsonUInt::new(0)?;
    assert_eq!(value.get(), 0);
    Ok(())
}

#[test]
fn accepts_max_safe_integer() -> Result<(), DefinitionError> {
    let value = IJsonUInt::new(IJsonUInt::MAX)?;
    assert_eq!(value.get(), IJsonUInt::MAX);
    Ok(())
}

#[test]
fn rejects_values_above_max_safe_integer() {
    let result = IJsonUInt::new(IJsonUInt::MAX + 1);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(matches!(
            err,
            DefinitionError::InvalidIJsonNumber(ref msg) if msg.contains("0..=")
        ));
    }
}

#[test]
fn serde_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let value = IJsonUInt::new(42)?;
    let json = serde_json::to_string(&value)?;
    let parsed: IJsonUInt = serde_json::from_str(&json)?;
    assert_eq!(parsed, value);
    Ok(())
}
