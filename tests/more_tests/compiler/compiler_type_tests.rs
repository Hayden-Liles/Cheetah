use cheetah::compiler::types::*;
#[test]
fn test_complex_type_creation() {
    // Test nested collection types
    let list_of_lists = Type::List(Box::new(Type::List(Box::new(Type::Int))));
    let dict_with_tuple_keys = Type::Dict(
        Box::new(Type::Tuple(vec![Type::String, Type::Int])),
        Box::new(Type::Bool)
    );
    let tuple_with_collections = Type::Tuple(vec![
        Type::Int,
        Type::List(Box::new(Type::Float)),
        Type::Dict(Box::new(Type::String), Box::new(Type::Int))
    ]);

    // Verify the nested structure
    if let Type::List(inner) = list_of_lists {
        assert!(matches!(*inner, Type::List(_)));
        if let Type::List(innermost) = *inner {
            assert!(matches!(*innermost, Type::Int));
        } else {
            panic!("Expected List as inner type");
        }
    } else {
        panic!("Expected List as outer type");
    }

    if let Type::Dict(key_type, value_type) = dict_with_tuple_keys {
        assert!(matches!(*key_type, Type::Tuple(_)));
        assert!(matches!(*value_type, Type::Bool));
    } else {
        panic!("Expected Dict type");
    }

    if let Type::Tuple(elements) = tuple_with_collections {
        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0], Type::Int));
        assert!(matches!(elements[1], Type::List(_)));
        assert!(matches!(elements[2], Type::Dict(_, _)));
    } else {
        panic!("Expected Tuple type");
    }
}

#[test]
fn test_advanced_type_compatibility() {
    // Test complex type compatibility scenarios
    let list_int = Type::List(Box::new(Type::Int));
    let list_float = Type::List(Box::new(Type::Float));
    let list_any = Type::List(Box::new(Type::Any));

    // Tests with Any type
    assert!(list_int.is_compatible_with(&list_any));
    assert!(list_any.is_compatible_with(&list_int));

    // None compatibility
    let none_type = Type::None;
    assert!(none_type.is_compatible_with(&Type::String));
    assert!(none_type.is_compatible_with(&list_int));
    assert!(!none_type.is_compatible_with(&Type::Int)); // Non-reference type

    // Nested compatibility
    let nested_list_int = Type::List(Box::new(list_int.clone()));
    let nested_list_float = Type::List(Box::new(list_float.clone()));
    assert!(!nested_list_int.is_compatible_with(&nested_list_float));

    // Tuple compatibility
    let tuple1 = Type::Tuple(vec![Type::Int, Type::Float]);
    let tuple2 = Type::Tuple(vec![Type::Int, Type::Float]);
    let tuple3 = Type::Tuple(vec![Type::Int, Type::Int]);
    let tuple4 = Type::Tuple(vec![Type::Int, Type::Float, Type::Bool]);

    assert!(tuple1.is_compatible_with(&tuple2));
    assert!(!tuple1.is_compatible_with(&tuple3)); // Different element types
    assert!(!tuple1.is_compatible_with(&tuple4)); // Different length
}

#[test]
fn test_type_unification_complex() {
    // Test complex type unification scenarios

    // Unify nested collections
    let list_int = Type::List(Box::new(Type::Int));
    let list_float = Type::List(Box::new(Type::Float));
    let unified_list = Type::unify(&list_int, &list_float);

    assert!(unified_list.is_some());
    if let Some(Type::List(elem_type)) = unified_list {
        assert!(matches!(*elem_type, Type::Float));
    } else {
        panic!("Expected List type after unification");
    }

    // Unify complex tuples
    let tuple1 = Type::Tuple(vec![Type::Int, Type::Bool]);
    let tuple2 = Type::Tuple(vec![Type::Float, Type::Bool]);
    let unified_tuple = Type::unify(&tuple1, &tuple2);

    assert!(unified_tuple.is_some());
    if let Some(Type::Tuple(elems)) = unified_tuple {
        assert_eq!(elems.len(), 2);
        assert!(matches!(elems[0], Type::Float));
        assert!(matches!(elems[1], Type::Bool));
    } else {
        panic!("Expected Tuple type after unification");
    }

    // Test dictionary unification with different key types
    // With our enhanced dictionary support, we now allow unification of dictionaries with different key types
    // The result should be a dictionary with Any as the key type
    let dict1 = Type::Dict(Box::new(Type::String), Box::new(Type::Int));
    let dict2 = Type::Dict(Box::new(Type::Int), Box::new(Type::Int));
    let unified_dict = Type::unify(&dict1, &dict2);

    assert!(unified_dict.is_some()); // Different key types can now be unified
    if let Some(Type::Dict(key_type, val_type)) = unified_dict {
        assert!(matches!(*key_type, Type::Any)); // Key type should be Any
        assert!(matches!(*val_type, Type::Int)); // Value type should still be Int
    } else {
        panic!("Expected Dict type after unification");
    }
}

#[test]
fn test_function_types() {
    // Test function type creation and compatibility
    let fn_type1 = Type::function(
        vec![Type::Int, Type::String],
        Type::Bool
    );

    let fn_type2 = Type::function(
        vec![Type::Int, Type::String],
        Type::Bool
    );

    let fn_type3 = Type::function(
        vec![Type::Float, Type::String],
        Type::Bool
    );

    assert!(fn_type1.is_compatible_with(&fn_type2));
    assert!(!fn_type1.is_compatible_with(&fn_type3)); // Different param types

    if let Type::Function { param_types, return_type, .. } = fn_type1 {
        assert_eq!(param_types.len(), 2);
        assert!(matches!(param_types[0], Type::Int));
        assert!(matches!(param_types[1], Type::String));
        assert!(matches!(*return_type, Type::Bool));
    } else {
        panic!("Expected Function type");
    }
}

#[test]
fn test_type_indexability_and_callability() {
    // Test type indexability
    assert!(Type::List(Box::new(Type::Int)).is_indexable());
    assert!(Type::String.is_indexable());
    assert!(Type::Dict(Box::new(Type::String), Box::new(Type::Int)).is_indexable());
    // We now allow Int to be indexable for string character access
    assert!(Type::Int.is_indexable());
    assert!(!Type::Bool.is_indexable());

    // Test type callability
    assert!(Type::function(vec![], Type::Void).is_callable());
    assert!(!Type::Int.is_callable());
    assert!(!Type::List(Box::new(Type::Int)).is_callable());

    // Test indexed type
    let list_type = Type::List(Box::new(Type::String));
    let result = list_type.get_indexed_type(&Type::Int);
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Type::String));

    // Test Int indexing now returns String (for character access)
    let int_type = Type::Int;
    let result = int_type.get_indexed_type(&Type::Int);
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), Type::String));

    // Test error case with Bool
    let bool_type = Type::Bool;
    let result = bool_type.get_indexed_type(&Type::Int);
    assert!(result.is_err());
}