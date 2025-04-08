use cheetah::compiler::types::*;
use inkwell::context::Context;

#[test]
fn test_primitive_type_creation() {
    // Test basic type creation
    let int_type = Type::Int;
    let float_type = Type::Float;
    let bool_type = Type::Bool;
    let none_type = Type::None;
    
    assert!(matches!(int_type, Type::Int));
    assert!(matches!(float_type, Type::Float));
    assert!(matches!(bool_type, Type::Bool));
    assert!(matches!(none_type, Type::None));
}

#[test]
fn test_collection_type_creation() {
    // Test collection type creation
    let list_int = Type::List(Box::new(Type::Int));
    let dict_str_float = Type::Dict(Box::new(Type::String), Box::new(Type::Float));
    let tuple_types = Type::Tuple(vec![Type::Int, Type::Bool, Type::String]);
    
    assert!(matches!(list_int, Type::List(_)));
    assert!(matches!(dict_str_float, Type::Dict(_, _)));
    assert!(matches!(tuple_types, Type::Tuple(_)));
    
    if let Type::List(elem_type) = list_int {
        assert!(matches!(*elem_type, Type::Int));
    }
    
    if let Type::Dict(key_type, val_type) = dict_str_float {
        assert!(matches!(*key_type, Type::String));
        assert!(matches!(*val_type, Type::Float));
    }
    
    if let Type::Tuple(elem_types) = tuple_types {
        assert_eq!(elem_types.len(), 3);
        assert!(matches!(elem_types[0], Type::Int));
        assert!(matches!(elem_types[1], Type::Bool));
        assert!(matches!(elem_types[2], Type::String));
    }
}

#[test]
fn test_type_compatibility() {
    // Test type compatibility rules
    let int_type = Type::Int;
    let float_type = Type::Float;
    let bool_type = Type::Bool;
    let none_type = Type::None;
    let string_type = Type::String;
    
    // Same types are compatible
    assert!(int_type.is_compatible_with(&int_type));
    assert!(float_type.is_compatible_with(&float_type));
    
    // None is compatible with reference types
    assert!(none_type.is_compatible_with(&string_type));
    
    // Type coercion rules
    assert!(bool_type.can_coerce_to(&int_type));
    assert!(int_type.can_coerce_to(&float_type));
    assert!(!float_type.can_coerce_to(&int_type)); // This requires explicit conversion
}

#[test]
fn test_llvm_type_generation() {
    // Test LLVM type generation
    let context = Context::create();
    
    let int_type = Type::Int;
    let float_type = Type::Float;
    let bool_type = Type::Bool;
    let string_type = Type::String;
    
    let llvm_int = int_type.to_llvm_type(&context);
    let llvm_float = float_type.to_llvm_type(&context);
    let llvm_bool = bool_type.to_llvm_type(&context);
    let llvm_string = string_type.to_llvm_type(&context);
    
    // Verify that the LLVM types have the expected properties
    assert!(llvm_int.is_int_type());
    assert!(llvm_float.is_float_type());
    assert!(llvm_bool.is_int_type()); // bools are i1 in LLVM
    assert!(llvm_string.is_pointer_type()); // strings are pointers to string structs
    
    // Fixed the type annotation issue
    let int_width = llvm_int.into_int_type().get_bit_width();
    assert_eq!(int_width, 64); // Assuming Int is i64
}

#[test]
fn test_type_unification() {
    // Test type unification
    let int_type = Type::Int;
    let float_type = Type::Float;
    let bool_type = Type::Bool;
    let any_type = Type::Any;
    
    // Unify same types
    assert_eq!(Type::unify(&int_type, &int_type), Some(int_type.clone()));
    
    // Unify with Any
    assert_eq!(Type::unify(&int_type, &any_type), Some(int_type.clone()));
    assert_eq!(Type::unify(&any_type, &float_type), Some(float_type.clone()));
    
    // Unify numeric types
    assert_eq!(Type::unify(&int_type, &float_type), Some(float_type.clone()));
    assert_eq!(Type::unify(&bool_type, &int_type), Some(int_type.clone()));
    
    // Unify collection types
    let list_int = Type::List(Box::new(Type::Int));
    let list_float = Type::List(Box::new(Type::Float));
    let list_any = Type::List(Box::new(Type::Any));
    
    assert_eq!(Type::unify(&list_int, &list_any), Some(list_int.clone()));
    assert_eq!(Type::unify(&list_int, &list_float), Some(list_float.clone()));
}