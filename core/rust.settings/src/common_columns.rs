use super::types::{
    Column, ColumnSuggestion, ColumnType, InnerColumnType, InnerColumnTypeStringKind, OperationType,
};

/// Standard created_at column
pub fn created_at() -> Column {
    Column {
        id: "created_at".to_string(),
        name: "Created At".to_string(),
        description: "The time the record was created.".to_string(),
        column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
        nullable: false,
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: false,
        suggestions: ColumnSuggestion::None {},
    }
}

/// Standard created_by column
pub fn created_by() -> Column {
    Column {
        id: "created_by".to_string(),
        name: "Created By".to_string(),
        description: "The user who created the record.".to_string(),
        column_type: ColumnType::new_scalar(InnerColumnType::String {
            min_length: None,
            max_length: None,
            allowed_values: vec![],
            kind: InnerColumnTypeStringKind::User {},
        }),
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: false,
        nullable: false,
        suggestions: ColumnSuggestion::None {},
    }
}

/// Standard last_updated_at column
pub fn last_updated_at() -> Column {
    Column {
        id: "last_updated_at".to_string(),
        name: "Last Updated At".to_string(),
        description: "The time the record was last updated.".to_string(),
        column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: false,
        nullable: false,
        suggestions: ColumnSuggestion::None {},
    }
}

/// Standard last_updated_by column
pub fn last_updated_by() -> Column {
    Column {
        id: "last_updated_by".to_string(),
        name: "Last Updated By".to_string(),
        description: "The user who last updated the record.".to_string(),
        column_type: ColumnType::new_scalar(InnerColumnType::String {
            min_length: None,
            max_length: None,
            allowed_values: vec![],
            kind: InnerColumnTypeStringKind::User {},
        }),
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: false,
        nullable: false,
        suggestions: ColumnSuggestion::None {},
    }
}

pub fn guild_id(id: &'static str, name: &'static str, description: &'static str) -> Column {
    Column {
        id: id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        column_type: ColumnType::new_scalar(InnerColumnType::String {
            min_length: None,
            max_length: None,
            allowed_values: vec![],
            kind: InnerColumnTypeStringKind::Normal {},
        }),
        nullable: false,
        suggestions: ColumnSuggestion::None {},
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: false,
    }
}
