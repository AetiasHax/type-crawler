#[cfg(test)]
mod tests {
    use type_crawler::{Env, EnvOptions, TypeCrawler, TypeKind};

    #[test]
    fn test_simple() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/struct/simple.h").unwrap();
        assert_eq!(types.len(), 1);

        let my_struct = types.get("MyStruct").unwrap();
        let TypeKind::Struct(my_struct) = my_struct else {
            panic!("Expected Struct type, found: {my_struct:?}");
        };
        assert!(!my_struct.is_class());
        assert_eq!(my_struct.size(), 8);
        assert_eq!(my_struct.alignment(), 4);
        assert!(my_struct.base_types().is_empty());
        assert_eq!(my_struct.fields().len(), 2);
        assert_eq!(my_struct.fields()[0].name(), Some("value1"));
        assert_eq!(my_struct.fields()[0].kind(), &TypeKind::S32);
        assert_eq!(my_struct.fields()[1].name(), Some("value2"));
        assert_eq!(my_struct.fields()[1].kind(), &TypeKind::S8);
    }

    #[test]
    fn test_bitfield() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/struct/bitfield.h").unwrap();
        assert_eq!(types.len(), 1);

        let bitfield = types.get("BitField").unwrap();
        let TypeKind::Struct(bitfield) = bitfield else {
            panic!("Expected Struct type, found: {bitfield:?}");
        };
        assert_eq!(bitfield.fields().len(), 3);
        assert_eq!(bitfield.fields()[0].offset_bytes(), 0);
        assert_eq!(bitfield.fields()[0].offset_bits(), 0);
        assert_eq!(bitfield.fields()[0].bit_field_width(), Some(3));
        assert_eq!(bitfield.fields()[0].size(&types), 1);
        assert_eq!(bitfield.fields()[1].offset_bytes(), 0);
        assert_eq!(bitfield.fields()[1].offset_bits(), 3);
        assert_eq!(bitfield.fields()[1].bit_field_width(), Some(5));
        assert_eq!(bitfield.fields()[1].size(&types), 1);
        assert_eq!(bitfield.fields()[2].offset_bytes(), 1);
        assert_eq!(bitfield.fields()[2].offset_bits(), 8);
        assert_eq!(bitfield.fields()[2].bit_field_width(), Some(2));
        assert_eq!(bitfield.fields()[2].size(&types), 1);
    }

    #[test]
    fn test_inheritance() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/struct/inheritance.hpp").unwrap();
        assert_eq!(types.len(), 2);

        let base = types.get("Base").unwrap();
        let TypeKind::Struct(base) = base else {
            panic!("Expected Struct type, found: {base:?}");
        };
        assert_eq!(base.size(), 4);
        assert_eq!(base.alignment(), 4);

        assert!(base.base_types().is_empty());
        assert_eq!(base.fields().len(), 1);
        assert_eq!(base.fields()[0].name(), Some("baseValue"));
        assert_eq!(base.fields()[0].offset_bytes(), 0);
        assert_eq!(base.fields()[0].kind(), &TypeKind::S32);

        let derived = types.get("Derived").unwrap();
        let TypeKind::Struct(derived) = derived else {
            panic!("Expected Struct type, found: {derived:?}");
        };
        assert_eq!(derived.size(), 8);
        assert_eq!(derived.alignment(), 4);
        assert_eq!(derived.base_types(), &["Base"]);
        assert_eq!(derived.fields().len(), 1);
        assert_eq!(derived.fields()[0].name(), Some("derivedValue"));
        assert_eq!(derived.fields()[0].offset_bytes(), 4);
        assert_eq!(derived.fields()[0].kind(), &TypeKind::S32);

        let base_field = derived.get_field(&types, "baseValue").unwrap();
        assert_eq!(base_field.name(), Some("baseValue"));
        assert_eq!(base_field.kind(), &TypeKind::S32);

        let derived_field = derived.get_field(&types, "derivedValue").unwrap();
        assert_eq!(derived_field.name(), Some("derivedValue"));
        assert_eq!(derived_field.kind(), &TypeKind::S32);
    }

    #[test]
    fn test_basic_types() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/struct/basic_types.hpp").unwrap();
        assert_eq!(types.len(), 1);

        let basic_types = types.get("BasicTypes").unwrap();
        let TypeKind::Struct(basic_types) = basic_types else {
            panic!("Expected Struct type, found: {basic_types:?}");
        };
        assert_eq!(basic_types.fields().len(), 27);

        type FieldTest = (&'static str, fn(&TypeKind) -> bool);
        let fields: &[FieldTest] = &[
            ("b", |k| k == &TypeKind::Bool),
            // Chars
            ("ch", |k| k == &TypeKind::S8),
            ("uch", |k| k == &TypeKind::U8),
            ("ch16", |k| k == &TypeKind::Char16),
            ("ch32", |k| k == &TypeKind::Char32),
            ("wch", |k| matches!(k, &TypeKind::WChar { .. })),
            // Integers
            ("s16", |k| k == &TypeKind::S16),
            ("u16", |k| k == &TypeKind::U16),
            ("s32", |k| k == &TypeKind::S32),
            ("u32", |k| k == &TypeKind::U32),
            ("ssize", |k| matches!(k, &TypeKind::SSize { .. })),
            ("usize", |k| matches!(k, &TypeKind::USize { .. })),
            ("s64", |k| k == &TypeKind::S64),
            ("u64", |k| k == &TypeKind::U64),
            // Floats
            ("f32", |k| k == &TypeKind::F32),
            ("f64", |k| k == &TypeKind::F64),
            ("ld", |k| matches!(k, &TypeKind::LongDouble { .. })),
            // References
            ("ref", |k| matches!(k, &TypeKind::Reference { .. })),
            ("ptr", |k| matches!(k, &TypeKind::Pointer { .. })),
            ("funcptr", |k| matches!(k, &TypeKind::Pointer { .. })),
            (
                "memptr",
                |k| matches!(&k, &TypeKind::MemberPointer { record_name, .. } if record_name == "BasicTypes"),
            ),
            (
                "memfuncptr",
                |k| matches!(&k, &TypeKind::MemberPointer { record_name, .. } if record_name == "BasicTypes"),
            ),
            ("arr", |k| matches!(k, &TypeKind::Array { size: Some(10), .. })),
            // Compounds
            ("e", |k| matches!(k, &TypeKind::Enum { .. })),
            ("s", |k| matches!(k, &TypeKind::Struct { .. })),
            ("c", |k| matches!(k, &TypeKind::Class { .. })),
            ("u", |k| matches!(k, &TypeKind::Union { .. })),
        ];

        for (i, field) in fields.iter().enumerate() {
            let struct_field = &basic_types.fields()[i];
            assert_eq!(struct_field.name(), Some(field.0), "Field name mismatch for field {i}");
            assert!(
                (field.1)(struct_field.kind()),
                "Field type mismatch for field {}: found {:?}",
                field.0,
                struct_field.kind()
            );
        }
    }

    #[test]
    fn test_forward_decl() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/struct/forward_decl.h").unwrap();
        assert_eq!(types.len(), 2);

        let my_struct = types.get("MyStruct").unwrap();
        let TypeKind::Struct(my_struct) = my_struct else {
            panic!("Expected Struct type, found: {my_struct:?}");
        };
        assert_eq!(my_struct.fields().len(), 1);
        let TypeKind::Pointer { pointee_type, .. } = my_struct.fields()[0].kind() else {
            panic!("Expected Pointer type, found: {:?}", my_struct.fields()[0].kind());
        };
        let TypeKind::Named(pointee_type_name) = &**pointee_type else {
            panic!("Expected Named type, found: {:?}", pointee_type);
        };
        assert_eq!(pointee_type_name, "ForwardDecl");
    }

    #[test]
    fn test_incomplete_array() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/struct/incomplete_array.h").unwrap();
        assert_eq!(types.len(), 1);

        let my_struct = types.get("MyStruct").unwrap();
        let TypeKind::Struct(my_struct) = my_struct else {
            panic!("Expected Struct type, found: {my_struct:?}");
        };
        assert_eq!(my_struct.size(), 4);

        assert_eq!(my_struct.fields().len(), 2);
        assert_eq!(my_struct.fields()[0].name(), Some("x"));
        assert_eq!(my_struct.fields()[0].kind(), &TypeKind::S32);
        assert_eq!(my_struct.fields()[1].name(), Some("arr"));
        let TypeKind::Array { element_type, size } = my_struct.fields()[1].kind() else {
            panic!("Expected Array type, found: {:?}", my_struct.fields()[1].kind());
        };
        assert_eq!(**element_type, TypeKind::S32);
        assert_eq!(*size, None);
    }
}
