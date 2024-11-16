/// Tooling to document the Anti-Raid Luau API.
//
use std::sync::Arc;

/// A primitive type in the Anti-Raid Luau API.
/// E.g. u64, string, etc.
///
/// These are typedefs to primitive Luau types
#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct Primitive {
    pub name: String,
    pub lua_type: String,
    pub description: String,
    pub constraints: Vec<PrimitiveConstraint>,
}

// Primitive builder code
impl Primitive {
    pub fn name(self, name: &str) -> Self {
        let mut p = self;
        p.name = name.to_string();
        p
    }

    pub fn lua_type(self, lua_type: &str) -> Self {
        let mut p = self;
        p.lua_type = lua_type.to_string();
        p
    }

    pub fn description(self, description: &str) -> Self {
        let mut p = self;
        p.description = description.to_string();
        p
    }

    pub fn add_constraint(self, name: &str, description: &str, accepted_values: &str) -> Self {
        let mut p = self;
        p.constraints.push(PrimitiveConstraint {
            name: name.to_string(),
            description: description.to_string(),
            accepted_values: accepted_values.to_string(),
        });

        p
    }

    pub fn build(self) -> Primitive {
        self
    }
}

impl Primitive {
    pub fn type_definition(&self) -> String {
        format!("type {} = {}", self.name, self.lua_type)
    }
}

/// A primitive constraint
#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct PrimitiveConstraint {
    pub name: String,
    pub description: String,
    pub accepted_values: String,
}

/// A special helper types for building a list of primitives.
#[derive(Default)]
pub struct PrimitiveListBuilder {
    primitives: Vec<Primitive>,
}

impl PrimitiveListBuilder {
    pub fn add(
        self,
        name: &str,
        lua_type: &str,
        description: &str,
        p_fn: impl Fn(Primitive) -> Primitive,
    ) -> Self {
        let mut p = self;

        let new_primitive = Primitive::default()
            .name(name)
            .lua_type(lua_type)
            .description(description);

        p.primitives.push(p_fn(new_primitive));

        p
    }

    pub fn build(self) -> Vec<Primitive> {
        self.primitives
    }
}

/// A plugin in the Anti-Raid Luau API.
#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub methods: Vec<Method>,
    pub fields: Vec<Field>,
    pub types: Vec<Type>,
}

#[allow(dead_code)]
// Plugin builder code
impl Plugin {
    pub fn name(self, name: &str) -> Self {
        let mut p = self;
        p.name = name.to_string();
        p
    }

    pub fn description(self, description: &str) -> Self {
        let mut p = self;
        p.description = description.to_string();
        p
    }

    pub fn add_method(self, methods: Method) -> Self {
        let mut p = self;
        p.methods.push(methods);
        p
    }

    pub fn method_mut(mut self, name: &str, f: impl FnOnce(Method) -> Method) -> Self {
        let method = self.methods.iter_mut().find(|m| m.name == name);

        if let Some(method) = method {
            let new_method = f(method.clone());

            *method = new_method;
        } else {
            let method = Method {
                name: name.to_string(),
                ..Default::default()
            };
            self.methods.push(f(method));
        }

        self
    }

    pub fn add_field(self, fields: Field) -> Self {
        let mut p = self;
        p.fields.push(fields);
        p
    }

    pub fn add_type(self, types: Type) -> Self {
        let mut p = self;
        p.types.push(types);
        p
    }

    pub fn type_mut(self, name: &str, description: &str, f: impl FnOnce(Type) -> Type) -> Self {
        let mut p = self;
        let new_typ = Type::new(name, description);
        p.types.push(f(new_typ));

        p
    }

    pub fn field_mut(self, name: &str, f: impl FnOnce(Field) -> Field) -> Self {
        let mut p = self;
        let field = p.fields.iter_mut().find(|f| f.name == name);

        if let Some(field) = field {
            let new_field = f(field.clone());

            *field = new_field;
        } else {
            let field = Field {
                name: name.to_string(),
                ..Default::default()
            };
            p.fields.push(f(field));
        }

        p
    }

    pub fn build(self) -> Plugin {
        self
    }
}

/*impl Plugin {
    pub fn to_markdown(&self) -> String {

    }
}*/

/// A method in a plugin.
#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct Method {
    pub name: String,
    pub generics: Vec<String>,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub returns: Vec<Parameter>,
}

// Method builder code
impl Method {
    pub fn name(self, name: &str) -> Self {
        let mut m = self;
        m.name = name.to_string();
        m
    }

    pub fn add_generic(self, param: String) -> Self {
        let mut m = self;
        m.generics.push(param);
        m
    }

    pub fn description(self, description: &str) -> Self {
        let mut m = self;
        m.description = description.to_string();
        m
    }

    pub fn add_parameter(self, parameters: Parameter) -> Self {
        let mut m = self;
        m.parameters.push(parameters);
        m
    }

    pub fn parameter(&mut self, name: &str, f: impl FnOnce(Parameter) -> Parameter) -> Self {
        let parameter = self.parameters.iter_mut().find(|p| p.name == name);

        if let Some(parameter) = parameter {
            let new_parameter = f(parameter.clone());

            *parameter = new_parameter;
        } else {
            let mut parameter = Parameter::default();
            parameter.name = name.to_string();
            self.parameters.push(f(parameter));
        }

        self.clone()
    }

    pub fn add_return(self, returns: Parameter) -> Self {
        let mut m = self;
        m.returns.push(returns);
        m
    }

    pub fn return_(&mut self, name: &str, f: impl FnOnce(Parameter) -> Parameter) -> Self {
        let return_ = self.returns.iter_mut().find(|r| r.name == name);

        if let Some(return_) = return_ {
            let new_return_ = f(return_.clone());

            *return_ = new_return_;
        } else {
            let mut return_ = Parameter::default();
            return_.name = name.to_string();
            self.returns.push(f(return_));
        }

        self.clone()
    }

    pub fn build(self) -> Method {
        self
    }
}

impl Method {
    pub fn func_name(&self, cls: &Option<String>) -> String {
        if let Some(cls) = cls {
            format!("{}:{}", cls, self.name)
        } else {
            self.name.clone()
        }
    }

    /// Format: function name<GENERICS>(parameters) -> returns
    pub fn type_signature(&self, cls: &Option<String>) -> String {
        let mut out = String::new();
        out.push_str(&format!("function {}", self.func_name(cls)));

        // Add in the generics if they exist
        if !self.generics.is_empty() {
            out.push_str(&format!("<{}>", self.generics.join(", ")));
        }

        out.push_str(&format!(
            "({})",
            self.parameters
                .iter()
                .map(|p| p.type_signature())
                .collect::<Vec<_>>()
                .join(", ")
        ));

        if self.returns.len() == 1 {
            out.push_str(&format!(": {}", self.returns[0].r#type.clone()));
        } else if self.returns.len() > 2 {
            out.push_str(&format!(
                ": ({})",
                self.returns
                    .iter()
                    .map(|r| r.r#type.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        out
    }
}

/// A parameter in a method.
#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct Parameter {
    pub name: String,
    pub description: String,
    pub r#type: String,
}

// Parameter builder code
impl Parameter {
    pub fn name(self, name: &str) -> Self {
        let mut p = self;
        p.name = name.to_string();
        p
    }

    pub fn description(self, description: &str) -> Self {
        let mut p = self;
        p.description = description.to_string();
        p
    }

    pub fn typ(self, r#type: &str) -> Self {
        let mut p = self;
        p.r#type = r#type.to_string();
        p
    }

    pub fn build(self) -> Parameter {
        self
    }
}

impl Parameter {
    /// Format: <name>: <description>
    pub fn type_signature(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("{}: {}", self.name, self.r#type));
        out
    }
}

#[derive(Default, Debug, serde::Serialize, Clone)]
pub struct Field {
    pub name: String,
    pub description: String,
    pub r#type: String,
}

// Field builder code
impl Field {
    pub fn name(self, name: &str) -> Self {
        let mut f = self;
        f.name = name.to_string();
        f
    }

    pub fn description(self, description: &str) -> Self {
        let mut f = self;
        f.description = description.to_string();
        f
    }

    pub fn typ(self, r#type: &str) -> Self {
        let mut f = self;
        f.r#type = r#type.to_string();
        f
    }

    pub fn build(self) -> Field {
        self
    }
}

#[derive(Default, serde::Serialize, Clone)]
pub struct Type {
    pub name: String,
    pub description: String,
    pub generics: Vec<String>,
    pub example: Option<Arc<dyn erased_serde::Serialize + Send + Sync>>,
    pub fields: Vec<Field>, // Description of the fields in type
    pub methods: Vec<Method>,
}

impl std::fmt::Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Type")
            .field("description", &self.description)
            .field("methods", &self.methods)
            .finish()
    }
}

// Type builder code
impl Type {
    pub fn new(name: &str, description: &str) -> Self {
        Type {
            name: name.to_string(),
            description: description.to_string(),
            ..Default::default()
        }
    }

    pub fn example(self, example: Arc<dyn erased_serde::Serialize + Send + Sync>) -> Self {
        let mut t = self;
        t.example = Some(example);
        t
    }

    pub fn description(self, description: &str) -> Self {
        let mut t = self;
        t.description = description.to_string();
        t
    }

    pub fn add_method(self, methods: Method) -> Self {
        let mut t = self;
        t.methods.push(methods);
        t
    }

    pub fn method_mut(&mut self, name: &str, f: impl FnOnce(Method) -> Method) -> Self {
        let method = self.methods.iter_mut().find(|m| m.name == name);

        if let Some(method) = method {
            let new_method = f(method.clone());

            *method = new_method;
        } else {
            let mut method = Method::default();
            method.name = name.to_string();
            self.methods.push(f(method));
        }

        self.clone()
    }

    pub fn add_field(self, fields: Field) -> Self {
        let mut t = self;
        t.fields.push(fields);
        t
    }

    pub fn field(&mut self, name: &str, f: impl FnOnce(Field) -> Field) -> Self {
        let fields = self.fields.iter_mut().find(|p| p.name == name);

        if let Some(field) = fields {
            let new_field = f(field.clone());

            *field = new_field;
        } else {
            let mut field = Field::default();
            field.name = name.to_string();
            self.fields.push(f(field));
        }

        self.clone()
    }

    pub fn add_generic(self, param: &str) -> Self {
        let mut t = self;
        t.generics.push(param.to_string());
        t
    }

    pub fn build(self) -> Type {
        self
    }
}

// Other type code
impl Type {
    pub fn genericized_name(&self) -> String {
        let mut name = self.name.clone();

        if !self.generics.is_empty() {
            name.push_str(&format!("<{}>", self.generics.join(", ")));
        }

        name
    }
}
