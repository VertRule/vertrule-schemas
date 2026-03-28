use super::*;

#[test]
fn valid_openclaw_schema() -> Result<(), DefinitionError> {
    let id = SchemaId::new("vr.openclaw.ingress@0.1".to_string())?;
    assert_eq!(id.domain(), "openclaw");
    assert_eq!(id.name(), "ingress");
    assert_eq!(id.version(), "0.1");
    assert_eq!(id.as_str(), "vr.openclaw.ingress@0.1");
    Ok(())
}

#[test]
fn valid_lab_schema() -> Result<(), DefinitionError> {
    let id = SchemaId::new("vr.lab.operator_graph@0.1".to_string())?;
    assert_eq!(id.domain(), "lab");
    assert_eq!(id.name(), "operator_graph");
    assert_eq!(id.version(), "0.1");
    Ok(())
}

#[test]
fn valid_hyphenated_domain() -> Result<(), DefinitionError> {
    let id = SchemaId::new("vr.zero-fiber.modular_fiber@0.1".to_string())?;
    assert_eq!(id.domain(), "zero-fiber");
    assert_eq!(id.name(), "modular_fiber");
    Ok(())
}

#[test]
fn valid_rsi_schema() -> Result<(), DefinitionError> {
    let id = SchemaId::new("vr.rsi.detection@0.1".to_string())?;
    assert_eq!(id.domain(), "rsi");
    assert_eq!(id.name(), "detection");
    Ok(())
}

#[test]
fn valid_higher_version() -> Result<(), DefinitionError> {
    let id = SchemaId::new("vr.adapter.payload@1.0".to_string())?;
    assert_eq!(id.version(), "1.0");
    Ok(())
}

#[test]
fn rejects_missing_vr_prefix() {
    let result = SchemaId::new("openclaw.ingress@0.1".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_missing_at_sign() {
    let result = SchemaId::new("vr.openclaw.ingress".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_missing_domain_name_separator() {
    let result = SchemaId::new("vr.openclaw@0.1".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_empty_domain() {
    let result = SchemaId::new("vr..ingress@0.1".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_empty_name() {
    let result = SchemaId::new("vr.openclaw.@0.1".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_uppercase_domain() {
    let result = SchemaId::new("vr.OpenClaw.ingress@0.1".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_uppercase_name() {
    let result = SchemaId::new("vr.openclaw.Ingress@0.1".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_missing_minor_version() {
    let result = SchemaId::new("vr.openclaw.ingress@0".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_triple_version() {
    let result = SchemaId::new("vr.openclaw.ingress@0.1.2".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_non_numeric_version() {
    let result = SchemaId::new("vr.openclaw.ingress@beta.1".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_extra_dots_in_name() {
    let result = SchemaId::new("vr.openclaw.sub.ingress@0.1".to_string());
    assert!(result.is_err());
}

#[test]
fn rejects_too_long() {
    let long_name = "a".repeat(120);
    let result = SchemaId::new(format!("vr.openclaw.{long_name}@0.1"));
    assert!(result.is_err());
}

#[test]
fn serde_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let id = SchemaId::new("vr.openclaw.ingress@0.1".to_string())?;
    let json = serde_json::to_string(&id)?;
    assert_eq!(json, "\"vr.openclaw.ingress@0.1\"");
    let parsed: SchemaId = serde_json::from_str(&json)?;
    assert_eq!(parsed, id);
    Ok(())
}

#[test]
fn deserialize_rejects_invalid() {
    let result: Result<SchemaId, _> = serde_json::from_str("\"not.a.schema\"");
    assert!(result.is_err());
}

#[test]
fn display_matches_inner_string() -> Result<(), DefinitionError> {
    let id = SchemaId::new("vr.rsi.verification@0.1".to_string())?;
    assert_eq!(format!("{id}"), "vr.rsi.verification@0.1");
    Ok(())
}

#[test]
fn ord_is_lexicographic() -> Result<(), DefinitionError> {
    let a = SchemaId::new("vr.a.x@0.1".to_string())?;
    let b = SchemaId::new("vr.b.x@0.1".to_string())?;
    assert!(a < b);
    Ok(())
}

/// Validate all 15 adapter schemas parse correctly.
#[test]
fn all_adapter_schemas_valid() -> Result<(), DefinitionError> {
    let schemas = [
        "vr.openclaw.ingress@0.1",
        "vr.openclaw.tool_invocation@0.1",
        "vr.openclaw.action_decision@0.1",
        "vr.openclaw.denial@0.1",
        "vr.openclaw.heartbeat@0.1",
        "vr.openclaw.dispatch_request@0.1",
        "vr.openclaw.dispatch_allow@0.1",
        "vr.openclaw.dispatch_denial@0.1",
        "vr.openclaw.dispatch_fault@0.1",
        "vr.openclaw.posture@0.1",
        "vr.openclaw.mutation_proposal@0.1",
        "vr.openclaw.evaluation@0.1",
        "vr.openclaw.promotion_decision@0.1",
        "vr.rsi.detection@0.1",
        "vr.rsi.verification@0.1",
    ];
    for s in schemas {
        let id = SchemaId::new(s.to_string())?;
        assert_eq!(id.as_str(), s);
    }
    Ok(())
}
