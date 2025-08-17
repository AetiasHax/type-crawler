#[cfg(test)]
mod tests {
    use type_crawler::{Env, EnvOptions, TypeCrawler, TypeKind};

    #[test]
    fn test_simple() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/enum/simple.h").unwrap();
        assert_eq!(types.len(), 1);

        let enum_decl = types.get("MyEnum").unwrap();
        let TypeKind::Enum(enum_decl) = enum_decl else {
            panic!("Expected Enum type, found: {enum_decl:?}");
        };

        assert_eq!(enum_decl.size(), 1);
        assert_eq!(enum_decl.constants().len(), 3);
        assert!(enum_decl.get("Value1").is_some());
        assert!(enum_decl.get("Value2").is_some());
        assert!(enum_decl.get("Value3").is_some());
        assert!(enum_decl.get("Value4").is_none());
        assert_eq!(enum_decl.get_by_value(0).unwrap().name(), "Value1");
        assert_eq!(enum_decl.get_by_value(1).unwrap().name(), "Value2");
        assert_eq!(enum_decl.get_by_value(2).unwrap().name(), "Value3");
        assert!(enum_decl.get_by_value(3).is_none());
        assert_eq!(enum_decl.constants()[0].value(), 0);
        assert_eq!(enum_decl.constants()[1].value(), 1);
        assert_eq!(enum_decl.constants()[2].value(), 2);
    }

    #[test]
    fn test_short_enums() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/enum/short.h").unwrap();
        assert_eq!(types.len(), 4);

        let size1 = types.get("Size1").unwrap();
        let TypeKind::Enum(size1) = size1 else {
            panic!("Expected Enum type, found: {size1:?}");
        };
        assert_eq!(size1.size(), 1);

        let size2 = types.get("Size2").unwrap();
        let TypeKind::Enum(size2) = size2 else {
            panic!("Expected Enum type, found: {size2:?}");
        };
        assert_eq!(size2.size(), 2);

        let size4 = types.get("Size4").unwrap();
        let TypeKind::Enum(size4) = size4 else {
            panic!("Expected Enum type, found: {size4:?}");
        };
        assert_eq!(size4.size(), 4);

        let size8 = types.get("Size8").unwrap();
        let TypeKind::Enum(size8) = size8 else {
            panic!("Expected Enum type, found: {size8:?}");
        };
        assert_eq!(size8.size(), 8);
    }

    #[test]
    fn test_no_short_enums() {
        let crawler =
            TypeCrawler::new(Env::new(EnvOptions { short_enums: false, ..EnvOptions::default() }))
                .unwrap();
        let types = crawler.parse_file("tests/enum/short.h").unwrap();
        assert_eq!(types.len(), 4);

        let size1 = types.get("Size1").unwrap();
        let TypeKind::Enum(size1) = size1 else {
            panic!("Expected Enum type, found: {size1:?}");
        };
        assert_eq!(size1.size(), 4);

        let size2 = types.get("Size2").unwrap();
        let TypeKind::Enum(size2) = size2 else {
            panic!("Expected Enum type, found: {size2:?}");
        };
        assert_eq!(size2.size(), 4);

        let size4 = types.get("Size4").unwrap();
        let TypeKind::Enum(size4) = size4 else {
            panic!("Expected Enum type, found: {size4:?}");
        };
        assert_eq!(size4.size(), 4);

        let size8 = types.get("Size8").unwrap();
        let TypeKind::Enum(size8) = size8 else {
            panic!("Expected Enum type, found: {size8:?}");
        };
        assert_eq!(size8.size(), 8);
    }

    #[test]
    fn test_expr() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/enum/expr.h").unwrap();
        assert_eq!(types.len(), 2);

        let flags_enum = types.get("Flags").unwrap();
        let TypeKind::Enum(flags_enum) = flags_enum else {
            panic!("Expected Enum type, found: {flags_enum:?}");
        };
        assert_eq!(flags_enum.get("Flag1").unwrap().value(), 1);
        assert_eq!(flags_enum.get("Flag2").unwrap().value(), 2);
        assert_eq!(flags_enum.get("Flag3").unwrap().value(), 4);

        let thing_enum = types.get("Thing").unwrap();
        let TypeKind::Enum(thing_enum) = thing_enum else {
            panic!("Expected Enum type, found: {thing_enum:?}");
        };
        assert_eq!(thing_enum.get("Thing1").unwrap().value(), 100);
        assert_eq!(thing_enum.get("Thing2").unwrap().value(), 200);
        assert_eq!(thing_enum.get("Thing3").unwrap().value(), 300);
        assert_eq!(thing_enum.get("Thing3a").unwrap().value(), 301);
        assert_eq!(thing_enum.get("WeirdThing").unwrap().value(), 641);
    }
}
