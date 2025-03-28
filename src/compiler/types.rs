use inkwell::context::Context;
use inkwell::types::{BasicTypeEnum, FunctionType, BasicType};
use inkwell::AddressSpace;
use crate::ast::{Expr, Number, NameConstant};
use std::collections::HashMap;
use std::fmt;

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
    
    /// When a member is accessed on a non-class type
    NotAClass {
        expr_type: Type,
        member: String,
    },
    
    /// When an undefined member is accessed
    UndefinedMember {
        class_name: String,
        member: String,
    },
    
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
            TypeError::IncompatibleTypes { expected, got, operation } => {
                write!(f, "Type error in {}: expected {}, got {}", operation, expected, got)
            },
            TypeError::UndefinedVariable(name) => {
                write!(f, "Undefined variable: {}", name)
            },
            TypeError::InvalidOperator { operator, left_type, right_type } => {
                match right_type {
                    Some(right) => write!(f, "Invalid operator '{}' for types {} and {}", operator, left_type, right),
                    None => write!(f, "Invalid operator '{}' for type {}", operator, left_type),
                }
            },
            TypeError::InvalidArgument { function, param_index, expected, got } => {
                write!(f, "In call to '{}', argument {} has incompatible type: expected {}, got {}", 
                      function, param_index, expected, got)
            },
            TypeError::WrongArgumentCount { function, expected, got } => {
                write!(f, "Wrong number of arguments in call to '{}': expected {}, got {}", 
                      function, expected, got)
            },
            TypeError::NotAClass { expr_type, member } => {
                write!(f, "Cannot access member '{}' on non-class type {}", member, expr_type)
            },
            TypeError::UndefinedMember { class_name, member } => {
                write!(f, "Class '{}' has no member '{}'", class_name, member)
            },
            TypeError::CannotInferType(expr) => {
                write!(f, "Cannot infer type for expression: {}", expr)
            },
            TypeError::NotCallable(ty) => {
                write!(f, "Type {} is not callable", ty)
            },
            TypeError::NotIndexable(ty) => {
                write!(f, "Type {} is not indexable", ty)
            },
        }
    }
}

/// Represents the types in the Cheetah language
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitive types
    Int,
    Float,
    Bool,
    None,
    
    // Collection types
    String,
    Bytes,
    List(Box<Type>),
    Tuple(Vec<Type>),
    Dict(Box<Type>, Box<Type>),
    Set(Box<Type>),
    
    // Function type
    Function {
        param_types: Vec<Type>,
        param_names: Vec<String>,
        has_varargs: bool,
        has_kwargs: bool,
        default_values: Vec<bool>,
        return_type: Box<Type>,
    },
    
    // Class type
    Class {
        name: String,
        base_classes: Vec<String>,
        methods: HashMap<String, Box<Type>>,
        fields: HashMap<String, Type>,
    },
    
    // Special types
    Any,
    Void,
    Unknown,
    
    // Type parameter for generics
    TypeParam(String),
    
    // Generic type
    Generic {
        base_type: Box<Type>,
        type_args: Vec<Type>,
    },
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
            },
            Type::Dict(key_type, value_type) => {
                write!(f, "dict[{}, {}]", key_type, value_type)
            },
            Type::Set(elem_type) => write!(f, "set[{}]", elem_type),
            Type::Function { param_types, return_type, .. } => {
                write!(f, "function(")?;
                for (i, param) in param_types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            },
            Type::Class { name, .. } => write!(f, "class {}", name),
            Type::Any => write!(f, "Any"),
            Type::Void => write!(f, "void"),
            Type::Unknown => write!(f, "unknown"),
            Type::TypeParam(name) => write!(f, "{}", name),
            Type::Generic { base_type, type_args } => {
                write!(f, "{}", base_type)?;
                write!(f, "[")?;
                for (i, arg) in type_args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, "]")
            },
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
                // String is a pointer to a structure containing length and data
                let _string_struct = context.struct_type(
                    &[
                        context.i64_type().into(), // length
                        context.ptr_type(AddressSpace::default()).into(), // data
                    ],
                    false,
                );
                context.ptr_type(AddressSpace::default()).as_basic_type_enum()
            },
            Type::Bytes => {
                // Bytes is similar to String
                let _bytes_struct = context.struct_type(
                    &[
                        context.i64_type().into(), // length
                        context.ptr_type(AddressSpace::default()).into(), // data
                    ],
                    false,
                );
                context.ptr_type(AddressSpace::default()).as_basic_type_enum()
            },
            Type::List(element_type) => {
                // List is a pointer to a structure containing length, capacity, and data
                let _element_llvm_type = element_type.to_llvm_type(context);
                let _list_struct = context.struct_type(
                    &[
                        context.i64_type().into(), // length
                        context.i64_type().into(), // capacity
                        context.ptr_type(AddressSpace::default()).into(), // data
                    ],
                    false,
                );
                context.ptr_type(AddressSpace::default()).as_basic_type_enum()
            },
            Type::Tuple(element_types) => {
                // Tuple is a structure containing the elements
                let element_llvm_types: Vec<_> = element_types
                    .iter()
                    .map(|ty| ty.to_llvm_type(context))
                    .collect();
                let tuple_struct = context.struct_type(&element_llvm_types, false);
                tuple_struct.into()
            },
            Type::Dict(key_type, value_type) => {
                // Dict is a pointer to a complex structure with entries
                let key_llvm_type = key_type.to_llvm_type(context);
                let value_llvm_type = value_type.to_llvm_type(context);
                let _entry_struct = context.struct_type(
                    &[
                        key_llvm_type,
                        value_llvm_type,
                    ],
                    false,
                );
                let _dict_struct = context.struct_type(
                    &[
                        context.i64_type().into(), // count
                        context.i64_type().into(), // capacity
                        context.ptr_type(AddressSpace::default()).into(), // entries
                    ],
                    false,
                );
                context.ptr_type(AddressSpace::default()).as_basic_type_enum()
            },
            Type::Set(element_type) => {
                // Set is similar to Dict but with only keys
                let _element_llvm_type = element_type.to_llvm_type(context);
                let _set_struct = context.struct_type(
                    &[
                        context.i64_type().into(), // count
                        context.i64_type().into(), // capacity
                        context.ptr_type(AddressSpace::default()).into(), // elements
                    ],
                    false,
                );
                context.ptr_type(AddressSpace::default()).as_basic_type_enum()
            },
            Type::Function { .. } => {
                // For now, represent function as a generic pointer
                context.ptr_type(AddressSpace::default()).as_basic_type_enum()
            },
            Type::Class { .. } => {
                // For now, represent class as a generic pointer
                context.ptr_type(AddressSpace::default()).as_basic_type_enum()
            },
            Type::Any | Type::Unknown | Type::TypeParam(_) | Type::Generic { .. } => {
                // Generic pointer type for Any and Unknown
                context.ptr_type(AddressSpace::default()).as_basic_type_enum()
            },
            Type::Void => {
                // For completeness, though void should be handled separately
                context.ptr_type(AddressSpace::default()).as_basic_type_enum()
            },
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
        let field_types: Vec<BasicTypeEnum> = fields
            .values()
            .map(|ty| ty.to_llvm_type(context))
            .collect();
        
        // Create named struct type for the class
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
            _ => return_type.to_llvm_type(context).fn_type(&param_llvm_types, false),
        }
    }

    // Add to the Type impl block
    pub fn get_function_pointer_type<'ctx>(
        &self, 
        context: &'ctx Context
    ) -> inkwell::types::PointerType<'ctx> {
        if let Type::Function { param_types, return_type, .. } = self {
            let param_llvm_types: Vec<_> = param_types
                .iter()
                .map(|ty| ty.to_llvm_type(context).into())
                .collect();
                
            let _ret_type = if let Type::Void = **return_type {
                context.void_type().fn_type(&param_llvm_types, false)
            } else {
                return_type.to_llvm_type(context).fn_type(&param_llvm_types, false)
            };
            
            context.ptr_type(inkwell::AddressSpace::default())
        } else {
            panic!("Not a function type")
        }
    }

    pub fn create_type_info<'ctx>(
        &self,
        context: &'ctx Context
    ) -> inkwell::values::StructValue<'ctx> {
        // Create a struct containing type information
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
            Type::List(elem_type) => return self.create_container_type_info(context, "list", &[elem_type]),
            Type::Tuple(items) => return self.create_tuple_type_info(context, items),
            Type::Dict(key_type, val_type) => return self.create_container_type_info(context, "dict", &[key_type, val_type]),
            Type::Set(elem_type) => return self.create_container_type_info(context, "set", &[elem_type]),
            Type::Function { return_type, .. } => return self.create_function_type_info(context, return_type),
            Type::Class { name, .. } => return self.create_class_type_info(context, name),
            Type::Any => "Any",
            Type::Void => "void",
            Type::Unknown => "unknown",
            Type::TypeParam(name) => return self.create_named_type_info(context, "TypeParam", name),
            Type::Generic { base_type, .. } => return self.create_generic_type_info(context, base_type),
        };
        
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        
        let struct_type = context.struct_type(&[
            i32_type.into(),
            str_type.into()
        ], false);
        
        // Create values for the type info fields
        let id_value = i32_type.const_int(type_id as u64, false);
        let name_value = context.const_string(type_name.as_bytes(), true);
        
        struct_type.const_named_struct(&[
            id_value.into(),
            name_value.into()
        ])
    }
    
    // New function to handle tuple types specifically
    pub fn create_tuple_type_info<'ctx>(
        &self,
        context: &'ctx Context,
        items: &Vec<Type>
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        
        // Create type name (e.g., "tuple[int, str]")
        let mut type_name = String::from("tuple[");
        
        for (i, elem_type) in items.iter().enumerate() {
            if i > 0 {
                type_name.push_str(", ");
            }
            type_name.push_str(&format!("{}", elem_type));
        }
        
        type_name.push(']');
        
        // Create struct type (id, name, element_count, elements[])
        let struct_type = context.struct_type(&[
            i32_type.into(),              // type id
            str_type.into(),              // type name
            i32_type.into(),              // element count
            ptr_type.into(),              // element types array
        ], false);
        
        // Create values for the struct fields
        let id_value = i32_type.const_int(8, false);  // Tuple type ID
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
        element_types: &[&Type]
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        
        // Type ID based on container type
        let type_id = match container_type {
            "list" => 7,
            "tuple" => 8,
            "dict" => 9,
            "set" => 10,
            _ => 0,
        };
        
        // Create type name string (e.g., "list[int]", "tuple[int, str]")
        let mut type_name = String::from(container_type);
        type_name.push('[');
        
        for (i, elem_type) in element_types.iter().enumerate() {
            if i > 0 {
                type_name.push_str(", ");
            }
            type_name.push_str(&format!("{}", elem_type));
        }
        
        type_name.push(']');
        
        // Create struct type (id, name, element_count, elements[])
        let struct_type = context.struct_type(&[
            i32_type.into(),              // type id
            str_type.into(),              // type name
            i32_type.into(),              // element count
            ptr_type.into(),              // element types array
        ], false);
        
        // Create values for the struct fields
        let id_value = i32_type.const_int(type_id as u64, false);
        let name_value = context.const_string(type_name.as_bytes(), true);
        let count_value = i32_type.const_int(element_types.len() as u64, false);
        
        // We'd need to create an array of element type infos here
        // For simplicity, we'll just use a null pointer and handle this in a more complete implementation
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
        return_type: &Box<Type>
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        
        // Create function type name (e.g., "function() -> int")
        let type_name = format!("function() -> {}", return_type);
        
        // Create struct type (id, name, return_type)
        let struct_type = context.struct_type(&[
            i32_type.into(),              // type id
            str_type.into(),              // type name
            ptr_type.into(),              // return type (would be a type_info in a real implementation)
        ], false);
        
        // Create values for the struct fields
        let id_value = i32_type.const_int(11 as u64, false);  // Function type ID
        let name_value = context.const_string(type_name.as_bytes(), true);
        let return_value = ptr_type.const_null();  // In a complete implementation, this would be a pointer to return type's type_info
        
        struct_type.const_named_struct(&[
            id_value.into(),
            name_value.into(),
            return_value.into(),
        ])
    }
    
    pub fn create_class_type_info<'ctx>(
        &self,
        context: &'ctx Context,
        class_name: &str
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        
        // Create struct type (id, name, class_name)
        let struct_type = context.struct_type(&[
            i32_type.into(),              // type id
            str_type.into(),              // type name 
            str_type.into(),              // class name
        ], false);
        
        // Create values for the struct fields
        let id_value = i32_type.const_int(12 as u64, false);  // Class type ID
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
        name: &str
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        
        // Create type name (e.g., "TypeParam<T>")
        let type_name = format!("{}<{}>", prefix, name);
        
        // Create struct type (id, name)
        let struct_type = context.struct_type(&[
            i32_type.into(),              // type id
            str_type.into(),              // type name
        ], false);
        
        // Create values for the struct fields
        let id_value = i32_type.const_int(16 as u64, false);  // TypeParam type ID
        let name_value = context.const_string(type_name.as_bytes(), true);
        
        struct_type.const_named_struct(&[
            id_value.into(),
            name_value.into(),
        ])
    }
    
    pub fn create_generic_type_info<'ctx>(
        &self,
        context: &'ctx Context,
        base_type: &Box<Type>
    ) -> inkwell::values::StructValue<'ctx> {
        let i32_type = context.i32_type();
        let str_type = context.ptr_type(inkwell::AddressSpace::default());
        let ptr_type = context.ptr_type(inkwell::AddressSpace::default());
        
        // Create generic type name (e.g., "Generic<list>")
        let type_name = format!("Generic<{}>", base_type);
        
        // Create struct type (id, name, base_type)
        let struct_type = context.struct_type(&[
            i32_type.into(),              // type id
            str_type.into(),              // type name
            ptr_type.into(),              // base type (would be a type_info in a real implementation)
        ], false);
        
        // Create values for the struct fields
        let id_value = i32_type.const_int(17 as u64, false);  // Generic type ID
        let name_value = context.const_string(type_name.as_bytes(), true);
        let base_value = ptr_type.const_null();  // In a complete implementation, this would be a pointer to base type's type_info
        
        struct_type.const_named_struct(&[
            id_value.into(),
            name_value.into(),
            base_value.into(),
        ])
    }
    
    /// Infer the type of an AST expression
    pub fn from_expr(expr: &Expr) -> Self {
        match expr {
            Expr::Num { value, .. } => match value {
                Number::Integer(_) => Type::Int,
                Number::Float(_) => Type::Float,
                Number::Complex { .. } => Type::Float, // Simplification
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
                    // Try to infer a common type for all elements
                    // Simplified: just use the type of the first element
                    let element_type = Type::from_expr(&elts[0]);
                    Type::List(Box::new(element_type))
                }
            },
            Expr::Tuple { elts, .. } => {
                let element_types = elts.iter().map(|e| Type::from_expr(e)).collect();
                Type::Tuple(element_types)
            },
            Expr::Dict { keys, values, .. } => {
                if keys.is_empty() || values.is_empty() {
                    Type::Dict(Box::new(Type::Any), Box::new(Type::Any))
                } else {
                    // Simplified: just use the type of the first key and value
                    let key_type = if let Some(key) = &keys[0] {
                        Type::from_expr(key)
                    } else {
                        Type::Any
                    };
                    let value_type = Type::from_expr(&values[0]);
                    Type::Dict(Box::new(key_type), Box::new(value_type))
                }
            },
            Expr::Set { elts, .. } => {
                if elts.is_empty() {
                    Type::Set(Box::new(Type::Any))
                } else {
                    // Simplified: just use the type of the first element
                    let element_type = Type::from_expr(&elts[0]);
                    Type::Set(Box::new(element_type))
                }
            },
            Expr::Lambda { args, body, .. } => {
                // Infer parameter types (simplification: all Any for now)
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
            },
            // For all other expressions, return Unknown for now
            _ => Type::Unknown,
        }
    }
    
    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        // Any type is compatible with itself
        if self == other {
            return true;
        }

        // Any is compatible with any other type
        if *self == Type::Any || *other == Type::Any {
            return true;
        }

        // None is compatible with any reference type
        if *self == Type::None && is_reference_type(other) {
            return true;
        }
        if *other == Type::None && is_reference_type(self) {
            return true;
        }

        // Check for container type compatibility
        match (self, other) {
            (Type::List(self_elem), Type::List(other_elem)) => {
                self_elem.is_compatible_with(other_elem)
            },
            (Type::Tuple(self_elems), Type::Tuple(other_elems)) => {
                if self_elems.len() != other_elems.len() {
                    return false;
                }
                self_elems.iter().zip(other_elems.iter()).all(|(a, b)| a.is_compatible_with(b))
            },
            (Type::Dict(self_key, self_val), Type::Dict(other_key, other_val)) => {
                self_key.is_compatible_with(other_key) && self_val.is_compatible_with(other_val)
            },
            (Type::Set(self_elem), Type::Set(other_elem)) => {
                self_elem.is_compatible_with(other_elem)
            },
            // Handle Class inheritance here when implemented
            _ => false,
        }
    }
    
    /// Check if this type can be automatically coerced to another type
    pub fn can_coerce_to(&self, target_type: &Type) -> bool {
        // Same type - no coercion needed
        if self == target_type {
            return true;
        }
        
        // Already compatible
        if self.is_compatible_with(target_type) {
            return true;
        }
        
        // Define type coercion rules
        match (self, target_type) {
            // Int can be coerced to Float
            (Type::Int, Type::Float) => true,
            
            // Bool can be coerced to Int
            (Type::Bool, Type::Int) => true,
            
            // Bool can be coerced to Float
            (Type::Bool, Type::Float) => true,
            
            // Any numeric type can be coerced to String
            (Type::Int, Type::String) | (Type::Float, Type::String) | (Type::Bool, Type::String) => true,
            
            // Coerce elements in containers
            (Type::List(from_elem), Type::List(to_elem)) => from_elem.can_coerce_to(to_elem),
            
            (Type::Set(from_elem), Type::Set(to_elem)) => from_elem.can_coerce_to(to_elem),
            
            (Type::Dict(from_key, from_val), Type::Dict(to_key, to_val)) => 
                from_key.can_coerce_to(to_key) && from_val.can_coerce_to(to_val),
            
            // Tuples need all elements to be coercible
            (Type::Tuple(from_elems), Type::Tuple(to_elems)) => {
                if from_elems.len() != to_elems.len() {
                    return false;
                }
                
                from_elems.iter().zip(to_elems.iter()).all(|(from, to)| from.can_coerce_to(to))
            },
            
            // Type parameters can potentially be coerced (need more context)
            (Type::TypeParam(_), _) | (_, Type::TypeParam(_)) => true,
            
            // Add other coercion rules here
            
            // Default is no coercion
            _ => false,
        }
    }
    
    /// Unify two types, if possible
    pub fn unify(type1: &Type, type2: &Type) -> Option<Type> {
        // If types are identical, return either
        if type1 == type2 {
            return Some(type1.clone());
        }
        
        // Handle Any type
        if *type1 == Type::Any {
            return Some(type2.clone());
        }
        if *type2 == Type::Any {
            return Some(type1.clone());
        }
        
        // Handle Unknown type
        if *type1 == Type::Unknown {
            return Some(type2.clone());
        }
        if *type2 == Type::Unknown {
            return Some(type1.clone());
        }
        
        // Handle None with reference types
        if *type1 == Type::None && is_reference_type(type2) {
            return Some(type2.clone());
        }
        if *type2 == Type::None && is_reference_type(type1) {
            return Some(type1.clone());
        }
        
        // Handle collection types
        match (type1, type2) {
            (Type::List(elem1), Type::List(elem2)) => {
                Type::unify(elem1, elem2)
                    .map(|unified_elem| Type::List(Box::new(unified_elem)))
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
                        return None; // Cannot unify tuples if any element can't be unified
                    }
                }
                Some(Type::Tuple(unified_elems))
            },
            
            (Type::Dict(key1, val1), Type::Dict(key2, val2)) => {
                match (Type::unify(key1, key2), Type::unify(val1, val2)) {
                    (Some(unified_key), Some(unified_val)) => 
                        Some(Type::Dict(Box::new(unified_key), Box::new(unified_val))),
                    _ => None,
                }
            },
            
            (Type::Set(elem1), Type::Set(elem2)) => {
                Type::unify(elem1, elem2)
                    .map(|unified_elem| Type::Set(Box::new(unified_elem)))
            },
            
            // Handle numeric types - prefer more general type
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Some(Type::Float),
            (Type::Bool, Type::Int) | (Type::Int, Type::Bool) => Some(Type::Int),
            (Type::Bool, Type::Float) | (Type::Float, Type::Bool) => Some(Type::Float),
            
            // Handle type parameters
            (Type::TypeParam(name), other) | (other, Type::TypeParam(name)) => {
                // In a real type checker, you'd record a constraint on the type parameter
                // For now, just return the concrete type
                if !matches!(other, Type::TypeParam(_)) {
                    return Some(other.clone());
                } else {
                    // If both are type parameters, keep the first one
                    return Some(Type::TypeParam(name.clone()));
                }
            },
            
            // Add more unification rules as needed
            
            // Types cannot be unified
            _ => None,
        }
    }
    
    /// Check if this type is indexable (supports [] operator)
    pub fn is_indexable(&self) -> bool {
        matches!(self, 
            Type::List(_) | Type::Tuple(_) | Type::Dict(_, _) | Type::String | Type::Bytes
        )
    }
    
    /// Get the element type when this type is indexed
    pub fn get_indexed_type(&self, index_type: &Type) -> Result<Type, TypeError> {
        match self {
            Type::List(elem_type) => {
                if matches!(index_type, Type::Int) {
                    Ok(*elem_type.clone())
                } else {
                    Err(TypeError::InvalidOperator {
                        operator: "[]".to_string(),
                        left_type: self.clone(),
                        right_type: Some(index_type.clone()),
                    })
                }
            },
            Type::Tuple(elem_types) => {
                // If index is a literal integer constant, we can get exact type
                // For now, just return the first element type or Any if empty
                if matches!(index_type, Type::Int) {
                    if elem_types.is_empty() {
                        Ok(Type::Any)
                    } else {
                        Ok(elem_types[0].clone())
                    }
                } else {
                    Err(TypeError::InvalidOperator {
                        operator: "[]".to_string(),
                        left_type: self.clone(),
                        right_type: Some(index_type.clone()),
                    })
                }
            },
            Type::Dict(_, value_type) => {
                // Index type should be compatible with key type
                Ok(*value_type.clone())
            },
            Type::String => {
                if matches!(index_type, Type::Int) {
                    Ok(Type::String)  // Indexing a string gives a single-character string
                } else {
                    Err(TypeError::InvalidOperator {
                        operator: "[]".to_string(),
                        left_type: self.clone(),
                        right_type: Some(index_type.clone()),
                    })
                }
            },
            Type::Bytes => {
                if matches!(index_type, Type::Int) {
                    Ok(Type::Int)  // Indexing bytes gives an integer
                } else {
                    Err(TypeError::InvalidOperator {
                        operator: "[]".to_string(),
                        left_type: self.clone(),
                        right_type: Some(index_type.clone()),
                    })
                }
            },
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
            Type::Function { param_types, return_type, has_varargs, default_values, .. } => {
                // Check number of arguments
                let min_args = param_types.len() - default_values.iter().filter(|&&has_default| has_default).count();
                let max_args = if *has_varargs { usize::MAX } else { param_types.len() };
                
                if arg_types.len() < min_args || (!has_varargs && arg_types.len() > max_args) {
                    return Err(TypeError::WrongArgumentCount {
                        function: "function".to_string(),  // No name available here
                        expected: param_types.len(),
                        got: arg_types.len(),
                    });
                }
                
                // Check argument types
                for (i, (param_type, arg_type)) in param_types.iter().zip(arg_types.iter()).enumerate() {
                    if !arg_type.can_coerce_to(param_type) {
                        return Err(TypeError::InvalidArgument {
                            function: "function".to_string(),  // No name available here
                            param_index: i,
                            expected: param_type.clone(),
                            got: arg_type.clone(),
                        });
                    }
                }
                
                // Return the function's return type
                Ok(*return_type.clone())
            },
            Type::Class { name, .. } => {
                // When a class is called, it creates an instance of that class
                Ok(Type::Class { 
                    name: name.clone(), 
                    base_classes: vec![], 
                    methods: HashMap::new(),
                    fields: HashMap::new() 
                })
            },
            _ => Err(TypeError::NotCallable(self.clone())),
        }
    }
    
    /// Create a simple function type
    pub fn function(param_types: Vec<Type>, return_type: Type) -> Self {
        let param_count = param_types.len();
        Type::Function {
            param_types,
            param_names: vec!["".to_string(); param_count],  // No names available
            has_varargs: false,
            has_kwargs: false,
            default_values: vec![false; param_count],  // No default values
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
    // Maps variable names to their types
    variables: HashMap<String, Type>,
    // Maps function names to their types
    functions: HashMap<String, Type>,
    // Maps class names to their types
    classes: HashMap<String, Type>,
    // Parent scope (if any)
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
        self.variables.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.get_variable_type(name))
        })
    }

    /// Set the type of a variable
    pub fn set_variable_type(&mut self, name: &str, ty: Type) {
        self.variables.insert(name.to_string(), ty);
    }
    
    /// Get the type of a function
    pub fn get_function_type(&self, name: &str) -> Option<&Type> {
        self.functions.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.get_function_type(name))
        })
    }
    
    /// Set the type of a function
    pub fn set_function_type(&mut self, name: &str, ty: Type) {
        self.functions.insert(name.to_string(), ty);
    }
    
    /// Get the type of a class
    pub fn get_class_type(&self, name: &str) -> Option<&Type> {
        self.classes.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| p.get_class_type(name))
        })
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