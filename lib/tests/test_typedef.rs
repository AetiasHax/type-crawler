#[cfg(test)]
mod tests {
    use type_crawler::{Env, EnvOptions, TypeCrawler, TypeKind};

    #[test]
    fn test_simple() {
        let crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        let types = crawler.parse_file("tests/typedef/simple.h").unwrap();
        assert_eq!(types.len(), 4);

        let TypeKind::Typedef(u32_ty) = types.get("u32").unwrap() else {
            panic!("Expected Typedef type");
        };
        let TypeKind::Typedef(vu32_ty) = types.get("vu32").unwrap() else {
            panic!("Expected Typedef type");
        };
        let TypeKind::Typedef(cu32_ty) = types.get("cu32").unwrap() else {
            panic!("Expected Typedef type");
        };
        let TypeKind::Typedef(cvu32_ty) = types.get("cvu32").unwrap() else {
            panic!("Expected Typedef type");
        };

        assert_eq!(u32_ty.underlying_type(), &TypeKind::U32);
        assert!(!u32_ty.constant());
        assert!(!u32_ty.volatile());

        assert_eq!(vu32_ty.underlying_type(), &TypeKind::Named("u32".to_string()));
        assert!(!vu32_ty.constant());
        assert!(vu32_ty.volatile());

        assert_eq!(cu32_ty.underlying_type(), &TypeKind::Named("u32".to_string()));
        assert!(cu32_ty.constant());
        assert!(!cu32_ty.volatile());

        assert_eq!(cvu32_ty.underlying_type(), &TypeKind::Named("u32".to_string()));
        assert!(cvu32_ty.constant());
        assert!(cvu32_ty.volatile());
    }
}
