#[derive(Default, Debug, Clone, PartialEq)]
pub enum ColumnType {
    #[default]
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
    Emoji,
    Message,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Column {
    /// The ID of the column
    pub id: &'static str,
    
    /// The friendly name of the column
    pub name: &'static str,
    
    /// The type of the column
    pub column_type: ColumnType,
    
    /// Whether or not the column is nullable
    pub nullable: bool,
    
    /// Whether or not the column is unique
    pub unique: bool,
    
    /// Whether or not the column is an array
    pub array: bool,
    
    /// Internal: the column hint, may be used in autocomplete etc.
    pub hint: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ConfigOption {
    /// The ID of the option
    pub id: &'static str,

    /// The name of the option
    pub name: &'static str,
    
    /// The description of the option
    pub description: &'static str,
    
    /// The table name for the config option
    pub table: &'static str,

    /// The column name refering to the guild id of the config option    
    pub guild_id: &'static str,
            
    /// The columns for this option
    pub columns: Vec<Column>,
    
    /// Whether or not the row must exist before hand
    pub row_must_exist: bool,
    
    /// Config option hint, used internally for stuff like guild channel configuration
    pub hint: Option<String>,
}