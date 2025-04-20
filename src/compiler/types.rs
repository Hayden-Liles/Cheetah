use crate::ast::{Expr, NameConstant, Number};
use inkwell::context::Context;
use inkwell::types::{BasicType, BasicTypeEnum, FunctionType};
use inkwell::AddressSpace;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Represents type errors that can occur during type checking
#[derive(Debug, Clone)]
pub enum TypeError {
    /// When incompatible types are used in an operation
    IncompatibleTypes {
        expected: Type,
        got: Type,
        operation: String,
    },

    /// When a variable is used without being defined
    UndefinedVariable(String),

    /// When an invalid operator is used with specific types
    InvalidOperator {
        operator: String,
        left_type: Type,
        right_type: Option<Type>,
    },

    /// When a function is called with wrong argument types
    InvalidArgument {
        function: String,
        param_index: usize,
        expected: Type,
        got: Type,
    },

    /// When a function is called with wrong number of arguments
    WrongArgumentCount {
        function: String,
        expected: usize,
        got: usize,
    },

    /// When a function is called with an invalid number of arguments
    InvalidArgumentCount {
        function: String,
        expected: String,
        got: usize,
    },

    /// When a member is accessed on a non-class type
    NotAClass { expr_type: Type, member: String },

    /// When an undefined member is accessed
    UndefinedMember { class_name: String, member: String },

    /// When a type cannot be inferred
    CannotInferType(String),

    /// When a type is not callable
    NotCallable(Type),

    /// When a type is not indexable
    NotIndexable(Type),
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::IncompatibleTypes {
                expected,
                got,
                operation,
            } => {
                write!(
                    f,
                    "Type error in {}: expected {}, got {}",
                    operation, expected, got
                )
            }
            TypeError::UndefinedVariable(name) => {
                write!(f, "Undefined variable: {}", name)
            }
            TypeError::InvalidOperator {
                operator,
                left_type,
                right_type,
            } => match right_type {
                Some(right) => write!(
                    f,
                    "Invalid operator '{}' for types {} and {}",
                    operator, left_type, right
                ),
                None => write!(f, "Invalid operator '{}' for type {}", operator, left_type),
            },
            TypeError::InvalidArgument {
                function,
                param_index,
                expected,
                got,
            } => {
                write!(
                    f,
                    "In call to '{}', argument {} has incompatible type: expected {}, got {}",
                    function, param_index, expected, got
                )
            }
            TypeError::WrongArgumentCount {
                function,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Wrong number of arguments in call to '{}': expected {}, got {}",
                    function, expected, got
                )
            }
            TypeError::NotAClass { expr_type, member } => {
                write!(
                    f,
                    "Cannot access member '{}' on non-class type {}",
                    member, expr_type
                )
            }
            TypeError::UndefinedMember { class_name, member } => {
                write!(f, "Class '{}' has no member '{}'", class_name, member)
            }
            TypeError::CannotInferType(expr) => {
                write!(f, "Cannot infer type for expression: {}", expr)
            }
            TypeError::NotCallable(ty) => {
                write!(f, "Type {} is not callable", ty)
            }
            TypeError::NotIndexable(ty) => {
                write!(f, "Type {} is not indexable", ty)
            }
            TypeError::InvalidArgumentCount {
                function,
                expected,
                got,
            } => {
                write!(
                    f,
                    "Invalid number of arguments in call to '{}': expected {}, got {}",
                    function, expected, got
                )
            }
        }
    }
}

/// Represents the types in the Cheetah language
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,
    Bool,
    None,

    String,
    Bytes,
    List(Box<Type>),
    Tuple(Vec<Type>),
    Dict(Box<Type>, Box<Type>),
    Set(Box<Type>),

    Function {
        param_types: Vec<Type>,
        param_names: Vec<String>,
        has_varargs: bool,
        has_kwargs: bool,
        default_values: Vec<bool>,
        return_type: Box<Type>,
    },

    Class {
        name: String,
        base_classes: Vec<String>,
        methods: HashMap<String, Box<Type>>,
        fields: HashMap<String, Type>,
    },

    Any,
    Void,
    Unknown,

    TypeParam(String),

    Generic {
        base_type: Box<Type>,
        type_args: Vec<Type>,
    },
}

// Custom implementation of Hash for Type that skips HashMap fields
impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Type::Int => {
                0.hash(state);
            }
            Type::Float => {
                1.hash(state);
            }
            Type::Bool => {
                2.hash(state);
            }
            Type::None => {
                3.hash(state);
            }
            Type::String => {
                4.hash(state);
            }
            Type::Bytes => {
                5.hash(state);
            }
            Type::List(elem_type) => {
                6.hash(state);
                elem_type.hash(state);
            }
            Type::Tuple(elem_types) => {
                7.hash(state);
                elem_types.len().hash(state);
                for elem in elem_types {
                    elem.hash(state);
                }
            }
            Type::Dict(key_type, val_type) => {
                8.hash(state);
                key_type.hash(state);
                val_type.hash(state);
            }
            Type::Set(elem_type) => {
                9.hash(state);
                elem_type.hash(state);
            }
            Type::Function {
                param_types,
                param_names,
                has_varargs,
                has_kwargs,
                default_values,
                return_type,
            } => {
                10.hash(state);
                param_types.hash(state);
                param_names.hash(state);
                has_varargs.hash(state);
                has_kwargs.hash(state);
                default_values.hash(state);
                return_type.hash(state);
            }
            Type::Class {
                name, base_classes, ..
            } => {
                11.hash(state);
                name.hash(state);
                base_classes.hash(state);
            }
            Type::Any => {
                12.hash(state);
            }
            Type::Void => {
                13.hash(state);
            }
            Type::Unknown => {
                14.hash(state);
            }
            Type::TypeParam(name) => {
                15.hash(state);
                name.hash(state);
            }
            Type::Generic {
                base_type,
                type_args,
            } => {
                16.hash(state);
                base_type.hash(state);
                type_args.hash(state);
            }
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::None => write!(f, "None"),
            Type::String => write!(f, "str"),
            Type::Bytes => write!(f, "bytes"),
            Type::List(elem_type) => write!(f, "list[{}]", elem_type),
            Type::Tuple(elem_types) => {
                write!(f, "tuple[")?;
                for (i, t) in elem_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, "]")
            }
            Type::Dict(key_type, value_type) => {
                write!(f, "dict[{}, {}]", key_type, value_type)
            }
            Type::Set(elem_type) => write!(f, "set[{}]", elem_type),
            Type::Function {
                param_types,
                return_type,
                ..
            } => {
                write!(f, "function(")?;
                for (i, param) in param_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
            Type::Class { name, .. } => write!(f, "class {}", name),
            Type::Any => write!(f, "Any"),
            Type::Void => write!(f, "void"),
            Type::Unknown => write!(f, "unknown"),
            Type::TypeParam(name) => write!(f, "{}", name),
            Type::Generic {
                base_type,
                type_args,
            } => {
                write!(f, "{}", base_type)?;
                write!(f, "[")?;
                for (i, arg) in type_args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, "]")
            }
        }
    }
}

impl Type {
    /// Convert a Cheetah type to an LLVM type
    pub fn to_llvm_type<'ctx>(&self, context: &'ctx Context) -> BasicTypeEnum<'ctx> {
        match self {
            Type::Int => context.i64_type().into(),
            Type::Float => context.f64_type().into(),
            Type::Bool => context.bool_type().into(),
            Type::None => context.ptr_type(AddressSpace::default()).into(),
            Type::String => {
                let _string_struct = context.struct_type(
                    &[
                        context.i64_type().into(),
                        context.ptr_type(AddressSpace::default()).into(),
                    ],
                    false,
                );
                context
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum()
            }
            Type::Bytes => {
                let _bytes_struct = context.struct_type(
                    &[
                        context.i64_type().into(),
                        context.ptr_type(AddressSpace::default()).into(),
                    ],
                    false,
                );
                context
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum()
            }
            Type::List(element_type) => {
                let _element_llvm_type = element_type.to_llvm_type(context);
                let _list_struct = context.struct_type(
                    &[
                        context.i64_type().into(),
                        context.i64_type().into(),
                        context.ptr_type(AddressSpace::default()).into(),
                    ],
                    false,
                );
                context
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum()
            }
            Type::Tuple(element_types) => {
                let element_llvm_types: Vec<_> = element_types
                    .iter()
                    .map(|ty| ty.to_llvm_type(context))
                    .collect();
                let tuple_struct = context.struct_type(&element_llvm_types, false);
                tuple_struct.into()
            }
            Type::Dict(key_type, value_type) => {
                let key_llvm_type = key_type.to_llvm_type(context);
                let value_llvm_type = value_type.to_llvm_type(context);
                let _entry_struct = context.struct_type(&[key_llvm_type, value_llvm_type], false);
                let _dict_struct = context.struct_type(
                    &[
                        context.i64_type().into(),
                        context.i64_type().into(),
                        context.ptr_type(AddressSpace::default()).into(),
                    ],
                    false,
                );
                context
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum()
            }
            Type::Set(element_type) => {
                let _element_llvm_type = element_type.to_llvm_type(context);
                let _set_struct = context.struct_type(
                    &[
                        context.i64_type().into(),
                        context.i64_type().into(),
                        context.ptr_type(AddressSpace::default()).into(),
                    ],
                    false,
                );
                context
                    .ptr_type(AddressSpace::default())
                    .as_basic_type_enum()
            }
            Type::Function { .. } => context
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum(),
            Type::Class { .. } => context
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum(),
            Type::Any | Type::Unknown | Type::TypeParam(_) | Type::Generic { .. } => context
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum(),
            Type::Void => context
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum(),
        }
    }

    /// Get the appropriate LLVM void type (use this instead of returning a pointer for Void)
    pub fn get_void_type<'ctx>(context: &'ctx Context) -> inkwell::types::VoidType<'ctx> {
        context.void_type()
    }

    /// Create a class type with fields and methods
    pub fn create_class_type<'ctx>(
        &self,
        context: &'ctx Context,
        name: &str,
        fields: &HashMap<String, Type>,
    ) -> inkwell::types::StructType<'ctx> {
        let field_types: Vec<BasicTypeEnum> =
            fields.values().map(|ty| ty.to_llvm_type(context)).collect();

        let struct_type = context.opaque_struct_type(name);
        struct_type.set_body(&field_types, false);

        struct_type
    }

    /// Create an LLVM function type with given parameter and return types
    pub fn get_function_type<'ctx>(
        context: &'ctx Context,
        param_types: &[Type],
        return_type: &Type,
    ) -> FunctionType<'ctx> {
        let param_llvm_types: Vec<_> = param_types
            .iter()
            .map(|ty| ty.to_llvm_type(context).into())
            .collect();

        match return_type {
            Type::Void => context.void_type().fn_type(&param_llvm_types, false),
            _ => return_type
                .to_llvm_type(context)
                .fn_type(&param_llvm_types, false),
        }
    }

    pub fn get_function_pointer_type<'ctx>(
        &self,
        context: &'ctx Context,
    ) -> inkwell::types::PointerType<'ctx> {
        if let Type::Function {
            param_types,
            return_type,
            ..
        } = self
        {
            let param_llvm_types: Vec<_> = param_types
                .iter()
                .map(|ty| ty.to_llvm_type(context).into())
                .collect();

            let _ret_type = if let Type::Void = **return_type {
                context.void_type().fn_type(&param_llvm_types, false)
            } else {
                return_type
                    .to_llvm_type(context)
                    .fn_type(&param_llvm_types, false)
            };

            context.ptr_type(inkwell::AddressSpace::default())
        } else {
            panic!("Not a function type")
        }
    }

    pub fn create_type_info<'ctx>(
        &self,
        context: &'ctx Context,
    ) -> inkwell::values::StructValue<'ctx> {
        let type_id = match self {
            Type::Int => 1,
            Type::Float => 2,
            Type::Bool => 3,
            Type::None => 4,
            Type::String => 5,
            Type::Bytes => 6,
            Type::List(_) => 7,
            Type::Tuple(_) => 8,
            Type::Dict(_, _) => 9,
            Type::Set(_) => 10,
            Type::Function { .. } => 11,
            Type::Class { .. } => 12,
            Type::Any => 13,
            Type::Void => 14,
            Type::Unknown => 15,
            Type::TypeParam(_) => 16,
            Type::Generic { .. } => 17,
        };

        let type_name = match self {
            Type::Int => "int",
            Type::Float => "float",
            Type::Bool => "bool",
            Type::None => "None",
            Type::String => "str",
            Type::Bytes => "bytes",
            Type::List(elem_type) => {
                return self.create_container_type_info(context, "list", &[elem_type])
            }
            Type::Tuple(items) => return self.create_tuple_type_info(context, items),
            Type::Dict(key_type, val_type) => {
                return self.create_container_type_info(context, "dict", &[key_type, val_type])
            }
            Type::Set(elem_type) => {
                return self.create_container_type_info(context, "set", &[elem_type])
            }
            Type::Function { return_type, .. } => {
                return self.create_function_type_info(context, return_type)
            }
            Type::Class { name, .. } => return self.create_class_type_info(context, name),
            Type::Any => "Any",
            Type::Void => "void",
            Type::Unknown => "unknown",
            Type::TypeParam(name) => {
                return self.create_named_type_info(context, "TypeParam", name)
            }
            Type::Generic { base_type, .. } => {
                return self.create_generic_type_info(context, base_type)
            }
        };

        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());

        let struct_type = context.struct_type(&[i32_type.into(), str_type.into()], false);

        let id_value = i32_type.const_int(type_id as u64, false);
        let name_value = context.const_string(type_name.as_bytes(), true);

        struct_type.const_named_struct(&[id_value.into(), name_value.into()])
    }

    pub fn create_tuple_type_info<'ctx>(
        &self,
        context: &'ctx Context,
        items: &Vec<Type>,
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

        let mut type_name = String::from("tuple[");

        for (i, elem_type) in items.iter().enumerate() {
            if i > 0 {
                type_name.push_str(", ");
            }
            type_name.push_str(&format!("{}", elem_type));
        }

        type_name.push(']');

        let struct_type = context.struct_type(
            &[
                i32_type.into(),
                str_type.into(),
                i32_type.into(),
                ptr_type.into(),
            ],
            false,
        );

        let id_value = i32_type.const_int(8, false);
        let name_value = context.const_string(type_name.as_bytes(), true);
        let count_value = i32_type.const_int(items.len() as u64, false);
        let elements_value = ptr_type.const_null();

        struct_type.const_named_struct(&[
            id_value.into(),
            name_value.into(),
            count_value.into(),
            elements_value.into(),
        ])
    }

    pub fn create_container_type_info<'ctx>(
        &self,
        context: &'ctx Context,
        container_type: &str,
        element_types: &[&Type],
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

        let type_id = match container_type {
            "list" => 7,
            "tuple" => 8,
            "dict" => 9,
            "set" => 10,
            _ => 0,
        };

        let mut type_name = String::from(container_type);
        type_name.push('[');

        for (i, elem_type) in element_types.iter().enumerate() {
            if i > 0 {
                type_name.push_str(", ");
            }
            type_name.push_str(&format!("{}", elem_type));
        }

        type_name.push(']');

        let struct_type = context.struct_type(
            &[
                i32_type.into(),
                str_type.into(),
                i32_type.into(),
                ptr_type.into(),
            ],
            false,
        );

        let id_value = i32_type.const_int(type_id as u64, false);
        let name_value = context.const_string(type_name.as_bytes(), true);
        let count_value = i32_type.const_int(element_types.len() as u64, false);

        let elements_value = ptr_type.const_null();

        struct_type.const_named_struct(&[
            id_value.into(),
            name_value.into(),
            count_value.into(),
            elements_value.into(),
        ])
    }

    pub fn create_function_type_info<'ctx>(
        &self,
        context: &'ctx Context,
        return_type: &Box<Type>,
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

        let type_name = format!("function() -> {}", return_type);

        let struct_type =
            context.struct_type(&[i32_type.into(), str_type.into(), ptr_type.into()], false);

        let id_value = i32_type.const_int(11 as u64, false);
        let name_value = context.const_string(type_name.as_bytes(), true);
        let return_value = ptr_type.const_null();

        struct_type.const_named_struct(&[id_value.into(), name_value.into(), return_value.into()])
    }

    pub fn create_class_type_info<'ctx>(
        &self,
        context: &'ctx Context,
        class_name: &str,
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());

        let struct_type =
            context.struct_type(&[i32_type.into(), str_type.into(), str_type.into()], false);

        let id_value = i32_type.const_int(12 as u64, false);
        let type_name = format!("class {}", class_name);
        let name_value = context.const_string(type_name.as_bytes(), true);
        let class_name_value = context.const_string(class_name.as_bytes(), true);

        struct_type.const_named_struct(&[
            id_value.into(),
            name_value.into(),
            class_name_value.into(),
        ])
    }

    pub fn create_named_type_info<'ctx>(
        &self,
        context: &'ctx Context,
        prefix: &str,
        name: &str,
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());

        let type_name = format!("{}<{}>", prefix, name);

        let struct_type = context.struct_type(&[i32_type.into(), str_type.into()], false);

        let id_value = i32_type.const_int(16 as u64, false);
        let name_value = context.const_string(type_name.as_bytes(), true);

        struct_type.const_named_struct(&[id_value.into(), name_value.into()])
    }

    pub fn create_generic_type_info<'ctx>(
        &self,
        context: &'ctx Context,
        base_type: &Box<Type>,
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());

        let type_name = format!("Generic<{}>", base_type);

        let struct_type =
            context.struct_type(&[i32_type.into(), str_type.into(), ptr_type.into()], false);

        let id_value = i32_type.const_int(17 as u64, false);
        let name_value = context.const_string(type_name.as_bytes(), true);
        let base_value = ptr_type.const_null();

        struct_type.const_named_struct(&[id_value.into(), name_value.into(), base_value.into()])
    }

    /// Infer the type of an AST expression
    pub fn from_expr(expr: &Expr) -> Self {
        match expr {
            Expr::Num { value, .. } => match value {
                Number::Integer(_) => Type::Int,
                Number::Float(_) => Type::Float,
                Number::Complex { .. } => Type::Float,
            },
            Expr::Str { .. } => Type::String,
            Expr::Bytes { .. } => Type::Bytes,
            Expr::NameConstant { value, .. } => match value {
                NameConstant::True | NameConstant::False => Type::Bool,
                NameConstant::None => Type::None,
            },
            Expr::List { elts, .. } => {
                if elts.is_empty() {
                    Type::List(Box::new(Type::Any))
                } else {
                    let element_type = Type::from_expr(&elts[0]);
                    Type::List(Box::new(element_type))
                }
            }
            Expr::Tuple { elts, .. } => {
                let element_types = elts.iter().map(|e| Type::from_expr(e)).collect();
                Type::Tuple(element_types)
            }
            Expr::Dict { keys, values, .. } => {
                if keys.is_empty() || values.is_empty() {
                    Type::Dict(Box::new(Type::Any), Box::new(Type::Any))
                } else {
                    let key_type = if let Some(key) = &keys[0] {
                        Type::from_expr(key)
                    } else {
                        Type::Any
                    };
                    let value_type = Type::from_expr(&values[0]);
                    Type::Dict(Box::new(key_type), Box::new(value_type))
                }
            }
            Expr::Set { elts, .. } => {
                if elts.is_empty() {
                    Type::Set(Box::new(Type::Any))
                } else {
                    let element_type = Type::from_expr(&elts[0]);
                    Type::Set(Box::new(element_type))
                }
            }
            Expr::Lambda { args, body, .. } => {
                let param_types = vec![Type::Any; args.len()];
                let param_names = args.iter().map(|param| param.name.clone()).collect();
                let default_values = args.iter().map(|param| param.default.is_some()).collect();
                let return_type = Type::from_expr(body);

                Type::Function {
                    param_types,
                    param_names,
                    has_varargs: args.iter().any(|p| p.is_vararg),
                    has_kwargs: args.iter().any(|p| p.is_kwarg),
                    default_values,
                    return_type: Box::new(return_type),
                }
            }
            _ => Type::Unknown,
        }
    }

    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        if self == other {
            return true;
        }

        if *self == Type::Any || *other == Type::Any {
            return true;
        }

        if *self == Type::None && is_reference_type(other) {
            return true;
        }
        if *other == Type::None && is_reference_type(self) {
            return true;
        }

        match (self, other) {
            (Type::List(self_elem), Type::List(other_elem)) => {
                self_elem.is_compatible_with(other_elem)
            }
            (Type::Tuple(self_elems), Type::Tuple(other_elems)) => {
                if self_elems.len() != other_elems.len() {
                    return false;
                }
                self_elems
                    .iter()
                    .zip(other_elems.iter())
                    .all(|(a, b)| a.is_compatible_with(b))
            }
            (Type::Dict(self_key, self_val), Type::Dict(other_key, other_val)) => {
                self_key.is_compatible_with(other_key) && self_val.is_compatible_with(other_val)
            }
            (Type::Set(self_elem), Type::Set(other_elem)) => {
                self_elem.is_compatible_with(other_elem)
            }
            _ => false,
        }
    }

    /// Check if this type can be automatically coerced to another type
    pub fn can_coerce_to(&self, target_type: &Type) -> bool {
        if self == target_type {
            return true;
        }

        if self.is_compatible_with(target_type) {
            return true;
        }

        match (self, target_type) {
            (_, Type::Any) => true,

            (Type::Int, Type::Float) => true,
            (Type::Int, Type::Bool) => true,
            (Type::Bool, Type::Int) => true,
            (Type::Bool, Type::Float) => true,
            (Type::Float, Type::Int) => false,
            (Type::Float, Type::Bool) => true,

            (Type::Int, Type::String) => true,
            (Type::Float, Type::String) => true,
            (Type::Bool, Type::String) => true,

            (Type::String, Type::Int) => true,
            (Type::String, Type::Float) => true,
            (Type::String, Type::Bool) => true,

            (Type::None, _) if is_reference_type(target_type) => true,

            (Type::List(from_elem), Type::List(to_elem)) => from_elem.can_coerce_to(to_elem),
            (Type::Set(from_elem), Type::Set(to_elem)) => from_elem.can_coerce_to(to_elem),
            (Type::Dict(from_key, from_val), Type::Dict(to_key, to_val)) => {
                from_key.can_coerce_to(to_key) && from_val.can_coerce_to(to_val)
            }

            (Type::Dict(_, from_val), to_type) => from_val.can_coerce_to(to_type),

            (_, Type::Dict(_, to_val)) if **to_val == Type::Any => true,

            (Type::Tuple(from_elems), Type::Tuple(to_elems)) => {
                if from_elems.len() != to_elems.len() {
                    return false;
                }
                from_elems
                    .iter()
                    .zip(to_elems.iter())
                    .all(|(from, to)| from.can_coerce_to(to))
            }

            (
                Type::Class {
                    name: from_name, ..
                },
                Type::Class { name: to_name, .. },
            ) => {
                if from_name == to_name {
                    return true;
                }

                false
            }

            (
                Type::Function {
                    param_types: from_params,
                    return_type: from_return,
                    ..
                },
                Type::Function {
                    param_types: to_params,
                    return_type: to_return,
                    ..
                },
            ) => {
                if from_params.len() != to_params.len() {
                    return false;
                }

                let params_ok = from_params
                    .iter()
                    .zip(to_params.iter())
                    .all(|(to_param, from_param)| to_param.can_coerce_to(from_param));
                let return_ok = from_return.can_coerce_to(to_return);

                params_ok && return_ok
            }

            (Type::TypeParam(_), _) | (_, Type::TypeParam(_)) => true,

            _ => false,
        }
    }

    /// Unify two types, if possible
    pub fn unify(type1: &Type, type2: &Type) -> Option<Type> {
        if type1 == type2 {
            return Some(type1.clone());
        }

        if *type1 == Type::Any {
            return Some(type2.clone());
        }
        if *type2 == Type::Any {
            return Some(type1.clone());
        }

        if *type1 == Type::Unknown {
            return Some(type2.clone());
        }
        if *type2 == Type::Unknown {
            return Some(type1.clone());
        }

        if *type1 == Type::None && is_reference_type(type2) {
            return Some(type2.clone());
        }
        if *type2 == Type::None && is_reference_type(type1) {
            return Some(type1.clone());
        }

        match (type1, type2) {
            (Type::List(elem1), Type::List(elem2)) => match Type::unify(elem1, elem2) {
                Some(unified_elem) => Some(Type::List(Box::new(unified_elem))),
                None => Some(Type::List(Box::new(Type::Any))),
            },

            (Type::Tuple(elems1), Type::Tuple(elems2)) => {
                if elems1.len() != elems2.len() {
                    return None;
                }

                let mut unified_elems = Vec::with_capacity(elems1.len());
                for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                    if let Some(unified) = Type::unify(e1, e2) {
                        unified_elems.push(unified);
                    } else {
                        return None;
                    }
                }
                Some(Type::Tuple(unified_elems))
            }

            (Type::Dict(key1, val1), Type::Dict(key2, val2)) => {
                if matches!(**val2, Type::Dict(_, _)) {
                    println!("Special case: Unifying dictionary with nested dictionary: {:?} and {:?} -> {:?}",
                             Type::Dict(key1.clone(), val1.clone()),
                             Type::Dict(key2.clone(), val2.clone()),
                             Type::Dict(key1.clone(), val1.clone()));
                    return Some(Type::Dict(key1.clone(), val1.clone()));
                } else if matches!(**val1, Type::Dict(_, _)) {
                    println!("Special case: Unifying dictionary with nested dictionary: {:?} and {:?} -> {:?}",
                             Type::Dict(key1.clone(), val1.clone()),
                             Type::Dict(key2.clone(), val2.clone()),
                             Type::Dict(key2.clone(), val2.clone()));
                    return Some(Type::Dict(key2.clone(), val2.clone()));
                }

                let unified_key = Type::unify(key1, key2).unwrap_or(Type::Any);
                let unified_val = Type::unify(val1, val2).unwrap_or(Type::Any);
                println!(
                    "Unifying dictionary types: {:?} and {:?} -> {:?}",
                    Type::Dict(key1.clone(), val1.clone()),
                    Type::Dict(key2.clone(), val2.clone()),
                    Type::Dict(Box::new(unified_key.clone()), Box::new(unified_val.clone()))
                );
                Some(Type::Dict(Box::new(unified_key), Box::new(unified_val)))
            }

            (Type::Dict(_, val1), other) | (other, Type::Dict(_, val1)) => {
                if let Some(unified) = Type::unify(val1, other) {
                    return Some(unified);
                }
                Some(Type::Any)
            }

            (Type::Set(elem1), Type::Set(elem2)) => {
                Type::unify(elem1, elem2).map(|unified_elem| Type::Set(Box::new(unified_elem)))
            }

            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Some(Type::Float),
            (Type::Bool, Type::Int) | (Type::Int, Type::Bool) => Some(Type::Int),
            (Type::Bool, Type::Float) | (Type::Float, Type::Bool) => Some(Type::Float),

            (Type::TypeParam(name), other) | (other, Type::TypeParam(name)) => {
                if !matches!(other, Type::TypeParam(_)) {
                    return Some(other.clone());
                } else {
                    return Some(Type::TypeParam(name.clone()));
                }
            }

            _ => None,
        }
    }

    /// Check if this type is indexable (supports [] operator)
    pub fn is_indexable(&self) -> bool {
        matches!(
            self,
            Type::List(_)
                | Type::Tuple(_)
                | Type::Dict(_, _)
                | Type::String
                | Type::Bytes
                | Type::Int
        )
    }

    /// Get the element type when this type is indexed
    pub fn get_indexed_type(&self, index_type: &Type) -> Result<Type, TypeError> {
        match self {
            Type::List(elem_type) => {
                if index_type.can_coerce_to(&Type::Int) {
                    Ok(*elem_type.clone())
                } else {
                    Err(TypeError::InvalidOperator {
                        operator: "[]".to_string(),
                        left_type: self.clone(),
                        right_type: Some(index_type.clone()),
                    })
                }
            }
            Type::String => {
                if index_type.can_coerce_to(&Type::Int) {
                    Ok(Type::String)
                } else {
                    Err(TypeError::InvalidOperator {
                        operator: "[]".to_string(),
                        left_type: self.clone(),
                        right_type: Some(index_type.clone()),
                    })
                }
            }
            Type::Tuple(elem_types) => {
                if matches!(index_type, Type::Int) {
                    if elem_types.is_empty() {
                        Ok(Type::Any)
                    } else if elem_types.len() == 1 {
                        Ok(elem_types[0].clone())
                    } else {
                        Ok(Type::Any)
                    }
                } else {
                    Err(TypeError::InvalidOperator {
                        operator: "[]".to_string(),
                        left_type: self.clone(),
                        right_type: Some(index_type.clone()),
                    })
                }
            }
            Type::Dict(key_type, value_type) => {
                if matches!(**key_type, Type::String) {
                    if matches!(index_type, Type::String) {
                        println!("Dictionary access with string key: {:?}", value_type);
                        return Ok(*value_type.clone());
                    }
                }

                if !index_type.can_coerce_to(key_type) {
                    return Err(TypeError::InvalidOperator {
                        operator: "[]".to_string(),
                        left_type: self.clone(),
                        right_type: Some(index_type.clone()),
                    });
                }

                println!(
                    "Dictionary access with compatible key type: {:?} -> {:?}",
                    index_type, value_type
                );
                Ok(*value_type.clone())
            }

            Type::Bytes => {
                if matches!(index_type, Type::Int) {
                    Ok(Type::Int)
                } else {
                    Err(TypeError::InvalidOperator {
                        operator: "[]".to_string(),
                        left_type: self.clone(),
                        right_type: Some(index_type.clone()),
                    })
                }
            }
            Type::Int => {
                if matches!(index_type, Type::Int) {
                    Ok(Type::String)
                } else {
                    Err(TypeError::InvalidOperator {
                        operator: "[]".to_string(),
                        left_type: self.clone(),
                        right_type: Some(index_type.clone()),
                    })
                }
            }
            _ => Err(TypeError::NotIndexable(self.clone())),
        }
    }

    /// Check if this type is callable
    pub fn is_callable(&self) -> bool {
        matches!(self, Type::Function { .. }) || matches!(self, Type::Class { .. })
    }

    /// Get the return type when this type is called with the given argument types
    pub fn get_call_return_type(&self, arg_types: &[Type]) -> Result<Type, TypeError> {
        match self {
            Type::Function {
                param_types,
                return_type,
                has_varargs,
                default_values,
                ..
            } => {
                let min_args = param_types.len()
                    - default_values
                        .iter()
                        .filter(|&&has_default| has_default)
                        .count();
                let max_args = if *has_varargs {
                    usize::MAX
                } else {
                    param_types.len()
                };

                if arg_types.len() < min_args || (!has_varargs && arg_types.len() > max_args) {
                    return Err(TypeError::WrongArgumentCount {
                        function: "function".to_string(),
                        expected: param_types.len(),
                        got: arg_types.len(),
                    });
                }

                for (i, (param_type, arg_type)) in
                    param_types.iter().zip(arg_types.iter()).enumerate()
                {
                    if !arg_type.can_coerce_to(param_type) {
                        return Err(TypeError::InvalidArgument {
                            function: "function".to_string(),
                            param_index: i,
                            expected: param_type.clone(),
                            got: arg_type.clone(),
                        });
                    }
                }

                Ok(*return_type.clone())
            }
            Type::Class { name, .. } => Ok(Type::Class {
                name: name.clone(),
                base_classes: vec![],
                methods: HashMap::new(),
                fields: HashMap::new(),
            }),
            _ => Err(TypeError::NotCallable(self.clone())),
        }
    }

    /// Create a simple function type
    pub fn function(param_types: Vec<Type>, return_type: Type) -> Self {
        let param_count = param_types.len();
        Type::Function {
            param_types,
            param_names: vec!["".to_string(); param_count],
            has_varargs: false,
            has_kwargs: false,
            default_values: vec![false; param_count],
            return_type: Box::new(return_type),
        }
    }

    /// Create a simple class type
    pub fn class(name: &str) -> Self {
        Type::Class {
            name: name.to_string(),
            base_classes: vec![],
            methods: HashMap::new(),
            fields: HashMap::new(),
        }
    }

    /// Create a simple list type
    pub fn list(element_type: Type) -> Self {
        Type::List(Box::new(element_type))
    }

    /// Create a simple dict type
    pub fn dict(key_type: Type, value_type: Type) -> Self {
        Type::Dict(Box::new(key_type), Box::new(value_type))
    }

    /// Returns `true` if the type is [`Class`].
    ///
    /// [`Class`]: Type::Class
    #[must_use]
    pub fn is_class(&self) -> bool {
        matches!(self, Self::Class { .. })
    }

    /// Get the type of a member (attribute or method) of this type
    pub fn get_member_type(&self, member: &str) -> Result<Type, TypeError> {
        match self {
            Type::Class {
                name,
                methods,
                fields,
                ..
            } => {
                if let Some(method_type) = methods.get(member) {
                    Ok(*method_type.clone())
                } else if let Some(field_type) = fields.get(member) {
                    Ok(field_type.clone())
                } else {
                    Err(TypeError::UndefinedMember {
                        class_name: name.clone(),
                        member: member.to_string(),
                    })
                }
            }
            Type::List(_element_type) => match member {
                "append" => {
                    // append takes one argument of any type and returns None
                    println!("List append method return type: None");
                    Ok(Type::Function {
                        param_types: vec![Type::Any],
                        param_names: vec!["item".to_string()],
                        has_varargs: false,
                        has_kwargs: false,
                        default_values: vec![false],
                        return_type: Box::new(Type::None),
                    })
                }
                _ => Err(TypeError::NotAClass {
                    expr_type: self.clone(),
                    member: member.to_string(),
                }),
            },
            Type::Dict(key_type, value_type) => match member {
                "keys" => {
                    let return_type = Type::List(key_type.clone());
                    println!("Dictionary keys method return type: {:?}", return_type);
                    Ok(Type::Function {
                        param_types: vec![],
                        param_names: vec![],
                        has_varargs: false,
                        has_kwargs: false,
                        default_values: vec![],
                        return_type: Box::new(return_type),
                    })
                }
                "values" => {
                    let return_type = Type::List(value_type.clone());
                    println!("Dictionary values method return type: {:?}", return_type);
                    Ok(Type::Function {
                        param_types: vec![],
                        param_names: vec![],
                        has_varargs: false,
                        has_kwargs: false,
                        default_values: vec![],
                        return_type: Box::new(return_type),
                    })
                }
                "items" => {
                    let tuple_type = Type::Tuple(vec![*key_type.clone(), *value_type.clone()]);
                    let return_type = Type::List(Box::new(tuple_type));
                    println!("Dictionary items method return type: {:?}", return_type);
                    Ok(Type::Function {
                        param_types: vec![],
                        param_names: vec![],
                        has_varargs: false,
                        has_kwargs: false,
                        default_values: vec![],
                        return_type: Box::new(return_type),
                    })
                }
                _ => Err(TypeError::NotAClass {
                    expr_type: self.clone(),
                    member: member.to_string(),
                }),
            },
            _ => Err(TypeError::NotAClass {
                expr_type: self.clone(),
                member: member.to_string(),
            }),
        }
    }
}

/// Determine if a type is a reference type (pointer to an object)
pub(crate) fn is_reference_type(ty: &Type) -> bool {
    matches!(
        ty,
        Type::String
            | Type::Bytes
            | Type::List(_)
            | Type::Dict(_, _)
            | Type::Set(_)
            | Type::Function { .. }
            | Type::Class { .. }
    )
}

/// Type context for tracking variable types during compilation
pub struct TypeContext {
    variables: HashMap<String, Type>,
    functions: HashMap<String, Type>,
    classes: HashMap<String, Type>,
    parent: Option<Box<TypeContext>>,
}

impl TypeContext {
    /// Create a new empty type context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            parent: None,
        }
    }

    /// Create a new type context with a parent scope
    pub fn with_parent(parent: TypeContext) -> Self {
        Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
            classes: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Get the type of a variable
    pub fn get_variable_type(&self, name: &str) -> Option<&Type> {
        self.variables
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_variable_type(name)))
    }

    /// Set the type of a variable
    pub fn set_variable_type(&mut self, name: &str, ty: Type) {
        self.variables.insert(name.to_string(), ty);
    }

    /// Get the type of a function
    pub fn get_function_type(&self, name: &str) -> Option<&Type> {
        self.functions
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_function_type(name)))
    }

    /// Set the type of a function
    pub fn set_function_type(&mut self, name: &str, ty: Type) {
        self.functions.insert(name.to_string(), ty);
    }

    /// Get the type of a class
    pub fn get_class_type(&self, name: &str) -> Option<&Type> {
        self.classes
            .get(name)
            .or_else(|| self.parent.as_ref().and_then(|p| p.get_class_type(name)))
    }

    /// Set the type of a class
    pub fn set_class_type(&mut self, name: &str, ty: Type) {
        self.classes.insert(name.to_string(), ty);
    }

    /// Look up any symbol (variable, function, or class)
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.get_variable_type(name)
            .or_else(|| self.get_function_type(name))
            .or_else(|| self.get_class_type(name))
    }
}
