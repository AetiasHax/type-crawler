#[cfg(test)]
mod tests {
    use type_crawler::{Env, EnvOptions, TypeCrawler, TypeKind};

    #[test]
    fn test_virtual() {
        let mut crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        crawler.parse_file("tests/class/virtual.hpp").unwrap();
        let types = crawler.into_types();
        assert_eq!(types.len(), 1);

        let TypeKind::Class(virtual_class) = types.get("VirtualClass").unwrap() else {
            panic!("Expected Class type");
        };
        assert!(virtual_class.is_class());
        assert!(virtual_class.is_virtual());

        assert_eq!(virtual_class.size(), 16);
        assert_eq!(virtual_class.alignment(), 8);

        assert_eq!(virtual_class.fields().len(), 1);
        assert_eq!(virtual_class.fields()[0].name(), Some("x"));
        assert_eq!(virtual_class.fields()[0].offset_bytes(), 8);
        assert_eq!(virtual_class.fields()[0].kind(), &TypeKind::S32);
    }
}
