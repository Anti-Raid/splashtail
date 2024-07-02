use crate::types::settings_wrap_precheck;

use super::types::{
    Column, ColumnSuggestion, ColumnType, InnerColumnType, InnerColumnTypeStringKind, OperationType,
};

/// Standard created_at column
pub fn created_at() -> Column {
    Column {
        id: "created_at",
        name: "Created At",
        column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
        nullable: false,
        unique: false,
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: None,
        suggestions: ColumnSuggestion::None {},
        pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
        default_pre_checks: settings_wrap_precheck(vec![]),
    }
}

/// Standard created_by column
pub fn created_by() -> Column {
    Column {
        id: "created_by",
        name: "Created By",
        column_type: ColumnType::new_scalar(InnerColumnType::String {
            min_length: None,
            max_length: None,
            allowed_values: vec![],
            kind: InnerColumnTypeStringKind::User,
        }),
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: None,
        nullable: false,
        unique: false,
        suggestions: ColumnSuggestion::None {},
        pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
        default_pre_checks: settings_wrap_precheck(vec![]),
    }
}

/// Standard last_updated_at column
pub fn last_updated_at() -> Column {
    Column {
        id: "last_updated_at",
        name: "Last Updated At",
        column_type: ColumnType::new_scalar(InnerColumnType::TimestampTz {}),
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: None,
        nullable: false,
        unique: false,
        suggestions: ColumnSuggestion::None {},
        pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
        default_pre_checks: settings_wrap_precheck(vec![]),
    }
}

/// Standard last_updated_by column
pub fn last_updated_by() -> Column {
    Column {
        id: "last_updated_by",
        name: "Last Updated By",
        column_type: ColumnType::new_scalar(InnerColumnType::String {
            min_length: None,
            max_length: None,
            allowed_values: vec![],
            kind: InnerColumnTypeStringKind::User,
        }),
        ignored_for: vec![OperationType::Create, OperationType::Update],
        secret: None,
        nullable: false,
        unique: false,
        suggestions: ColumnSuggestion::None {},
        pre_checks: settings_wrap_precheck(indexmap::indexmap! {}),
        default_pre_checks: settings_wrap_precheck(vec![]),
    }
}
