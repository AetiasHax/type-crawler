#[cfg(test)]
mod tests {
    use type_crawler::{Env, EnvOptions, TypeCrawler, TypeKind, TypePath};

    #[test]
    fn test_simple() {
        let mut crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        crawler.parse_file("tests/struct/simple.h").unwrap();
        let types = crawler.into_types();
        assert_eq!(types.len(), 1);

        let my_struct = types.get("MyStruct").unwrap();
        let TypeKind::Struct(my_struct) = my_struct else {
            panic!("Expected Struct type, found: {my_struct:?}");
        };
        assert!(!my_struct.is_class());
        assert!(!my_struct.is_virtual());
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
        let mut crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        crawler.parse_file("tests/struct/bitfield.h").unwrap();
        let types = crawler.into_types();
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
        let mut crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        crawler.parse_file("tests/struct/inheritance.hpp").unwrap();
        let types = crawler.into_types();
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
        assert_eq!(derived.base_types(), &[TypePath::global("Base")]);
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
        let mut crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        crawler.parse_file("tests/struct/basic_types.hpp").unwrap();
        let types = crawler.into_types();
        assert_eq!(types.len(), 1);

        let basic_types = types.get("BasicTypes").unwrap();
        let TypeKind::Struct(basic_types) = basic_types else {
            panic!("Expected Struct type, found: {basic_types:?}");
        };
        assert_eq!(basic_types.fields().len(), 27);

        type FieldTest = (&'static str, usize, fn(&TypeKind) -> bool);
        let fields: &[FieldTest] = &[
            ("b", 0x0, |k| k == &TypeKind::Bool),
            // Chars
            ("ch", 0x1, |k| k == &TypeKind::S8),
            ("uch", 0x2, |k| k == &TypeKind::U8),
            ("ch16", 0x4, |k| k == &TypeKind::Char16),
            ("ch32", 0x8, |k| k == &TypeKind::Char32),
            ("wch", 0xc, |k| matches!(k, &TypeKind::WChar { .. })),
            // Integers
            ("s16", 0x10, |k| k == &TypeKind::S16),
            ("u16", 0x12, |k| k == &TypeKind::U16),
            ("s32", 0x14, |k| k == &TypeKind::S32),
            ("u32", 0x18, |k| k == &TypeKind::U32),
            ("ssize", 0x20, |k| matches!(k, &TypeKind::SSize { .. })),
            ("usize", 0x28, |k| matches!(k, &TypeKind::USize { .. })),
            ("s64", 0x30, |k| k == &TypeKind::S64),
            ("u64", 0x38, |k| k == &TypeKind::U64),
            // Floats
            ("f32", 0x40, |k| k == &TypeKind::F32),
            ("f64", 0x48, |k| k == &TypeKind::F64),
            ("ld", 0x50, |k| matches!(k, &TypeKind::LongDouble { .. })),
            // References
            ("ref", 0x60, |k| matches!(k, &TypeKind::Reference { .. })),
            ("ptr", 0x68, |k| matches!(k, &TypeKind::Pointer { .. })),
            ("funcptr", 0x70, |k| matches!(k, &TypeKind::Pointer { .. })),
            (
                "memptr",
                0x78,
                |k| matches!(&k, &TypeKind::MemberPointer { record_name, .. } if record_name == "BasicTypes"),
            ),
            (
                "memfuncptr",
                0x80,
                |k| matches!(&k, &TypeKind::MemberPointer { record_name, .. } if record_name == "BasicTypes"),
            ),
            ("arr", 0x90, |k| matches!(k, &TypeKind::Array { size: Some(10), .. })),
            // Compounds
            ("e", 0x9a, |k| matches!(k, &TypeKind::Enum { .. })),
            ("s", 0x9c, |k| matches!(k, &TypeKind::Struct { .. })),
            ("c", 0xa0, |k| matches!(k, &TypeKind::Class { .. })),
            ("u", 0xa4, |k| matches!(k, &TypeKind::Union { .. })),
        ];

        for (i, (name, offset, test)) in fields.iter().enumerate() {
            let struct_field = &basic_types.fields()[i];
            assert_eq!(struct_field.name(), Some(*name), "Field name mismatch for field {i}");
            assert_eq!(
                struct_field.offset_bytes(),
                *offset,
                "Field offset mismatch for field {name}"
            );
            assert!(
                test(struct_field.kind()),
                "Field type mismatch for field {}: found {:?}",
                name,
                struct_field.kind()
            );
        }
    }

    #[test]
    fn test_forward_decl() {
        let mut crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        crawler.parse_file("tests/struct/forward_decl.h").unwrap();
        let types = crawler.into_types();
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
        assert_eq!(pointee_type_name, &TypePath::global("ForwardDecl"));
    }

    #[test]
    fn test_incomplete_array() {
        let mut crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        crawler.parse_file("tests/struct/incomplete_array.h").unwrap();
        let types = crawler.into_types();
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
