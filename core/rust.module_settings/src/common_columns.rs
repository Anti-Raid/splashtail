use super::types::{
    Column, ColumnSuggestion, ColumnType, InnerColumnType, InnerColumnTypeStringKind, OperationType,
};

/// Standard created_at column
pub fn created_at() -> Column {
    Column {
        id: "created_at",
        name: "Created At",
        description: "The time the record was created.",
        column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
        nullable: false,
        default: None,
        unique: false,
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: false,
        suggestions: ColumnSuggestion::None {},
    }
}

/// Standard created_by column
pub fn created_by() -> Column {
    Column {
        id: "created_by",
        name: "Created By",
        description: "The user who created the record.",
        column_type: ColumnType::new_scalar(InnerColumnType::String {
            min_length: None,
            max_length: None,
            allowed_values: vec![],
            kind: InnerColumnTypeStringKind::User,
        }),
        default: None,
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: false,
        nullable: false,
        unique: false,
        suggestions: ColumnSuggestion::None {},
    }
}

/// Standard last_updated_at column
pub fn last_updated_at() -> Column {
    Column {
        id: "last_updated_at",
        name: "Last Updated At",
        description: "The time the record was last updated.",
        column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: false,
        nullable: false,
        default: None,
        unique: false,
        suggestions: ColumnSuggestion::None {},
    }
}

/// Standard last_updated_by column
pub fn last_updated_by() -> Column {
    Column {
        id: "last_updated_by",
        name: "Last Updated By",
        description: "The user who last updated the record.",
        column_type: ColumnType::new_scalar(InnerColumnType::String {
            min_length: None,
            max_length: None,
            allowed_values: vec![],
            kind: InnerColumnTypeStringKind::User,
        }),
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: false,
        nullable: false,
        default: None,
        unique: false,
        suggestions: ColumnSuggestion::None {},
    }
}

pub fn guild_id(id: &'static str, name: &'static str, description: &'static str) -> Column {
    Column {
        id,
        name,
        description,
        column_type: ColumnType::new_scalar(InnerColumnType::String {
            min_length: None,
            max_length: None,
            allowed_values: vec![],
            kind: InnerColumnTypeStringKind::Normal,
        }),
        nullable: false,
        default: None,
        unique: false,
        suggestions: ColumnSuggestion::None {},
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: false,
    }
}
