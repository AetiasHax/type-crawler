#[cfg(test)]
mod tests {
    use type_crawler::{Env, EnvOptions, TypeCrawler, TypeKind};

    #[test]
    fn test_simple() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/union/simple.h").unwrap();
        assert_eq!(types.len(), 1);

        let TypeKind::Union(my_union) = types.get("MyUnion").unwrap() else {
            panic!("Expected Union type");
        };

        assert_eq!(my_union.fields().len(), 2);
        assert_eq!(my_union.fields()[0].name(), Some("value1"));
        assert_eq!(my_union.fields()[0].kind(), &TypeKind::S32);
        assert_eq!(my_union.fields()[1].name(), Some("value2"));
        assert_eq!(my_union.fields()[1].kind(), &TypeKind::S8);
    }

    #[test]
    fn test_bitfield() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/union/bitfield.h").unwrap();
        assert_eq!(types.len(), 1);

        let TypeKind::Union(bitfield) = types.get("BitField").unwrap() else {
            panic!("Expected Union type");
        };
        assert_eq!(bitfield.fields().len(), 3);
        assert_eq!(bitfield.fields()[0].bit_field_width(), Some(3));
        assert_eq!(bitfield.fields()[0].size(&types), 1);
        assert_eq!(bitfield.fields()[1].bit_field_width(), Some(5));
        assert_eq!(bitfield.fields()[1].size(&types), 1);
        assert_eq!(bitfield.fields()[2].bit_field_width(), Some(2));
        assert_eq!(bitfield.fields()[2].size(&types), 1);
    }
}
