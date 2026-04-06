#[cfg(test)]
mod tests {
    use type_crawler::{Env, EnvOptions, TypeCrawler, TypeKind, TypePath};

    #[test]
    fn test_virtual() {
        let mut crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        crawler.parse_file("tests/class/virtual.hpp").unwrap();
        let types = crawler.into_types();
        assert_eq!(types.len(), 1);

        let TypeKind::Class(virtual_class) = types.get("VirtualClass").unwrap() else {
            panic!("Expected Class type");
        };
        assert!(virtual_class.is_virtual());

        assert_eq!(virtual_class.size(), 16);
        assert_eq!(virtual_class.alignment(), 8);

        assert_eq!(virtual_class.fields().len(), 1);
        assert_eq!(virtual_class.fields()[0].name(), Some("x"));
        assert_eq!(virtual_class.fields()[0].offset_bytes(), 8);
        assert_eq!(virtual_class.fields()[0].kind(), &TypeKind::S32);
    }

    #[test]
    fn test_template() {
        let mut crawler = TypeCrawler::new(Env::new(EnvOptions::default())).unwrap();
        crawler.parse_file("tests/class/template.hpp").unwrap();
        let types = crawler.into_types();

        let template_class =
            types.get_template_class(TypePath::new(vec!["std".into()], "vector", vec![])).unwrap();
        assert!(!template_class.is_virtual());

        assert_eq!(template_class.parameters().len(), 1);
        assert_eq!(template_class.parameters()[0], "T");

        assert_eq!(template_class.fields().len(), 3);
        assert_eq!(template_class.fields()[0].name(), Some("elements"));
        assert_eq!(template_class.fields()[0].kind(), &TypeKind::Pointer {
            size: 8,
            pointee_type: Box::new(TypeKind::TemplateParam("T".into()))
        });

        assert_eq!(template_class.fields()[1].name(), Some("size"));
        assert_eq!(template_class.fields()[1].kind(), &TypeKind::S32);

        assert_eq!(template_class.fields()[2].name(), Some("capacity"));
        assert_eq!(template_class.fields()[2].kind(), &TypeKind::S32);

        let TypeKind::Class(world_struct) = types.get("World").unwrap() else {
            panic!("Expected Class type");
        };
        assert!(!world_struct.is_virtual());

        assert_eq!(world_struct.fields().len(), 2);
        assert_eq!(world_struct.fields()[0].name(), Some("activeEntities"));
        assert_eq!(world_struct.fields()[0].offset_bytes(), 0);
        let TypeKind::TemplateClassSpec(entity_vector) = world_struct.fields()[0].kind() else {
            panic!("Expected template class specialization");
        };
        assert_eq!(entity_vector.fields().len(), 3);
        assert_eq!(entity_vector.fields()[0].name(), Some("elements"));
        assert_eq!(entity_vector.fields()[0].offset_bytes(), 0);
        assert_eq!(entity_vector.fields()[0].kind(), &TypeKind::Pointer {
            size: 8,
            pointee_type: Box::new(TypeKind::Pointer {
                size: 8,
                pointee_type: Box::new(TypeKind::Named("Entity".into()))
            })
        });

        assert_eq!(entity_vector.fields()[1].name(), Some("size"));
        assert_eq!(entity_vector.fields()[1].offset_bytes(), 8);
        assert_eq!(entity_vector.fields()[1].kind(), &TypeKind::S32);

        assert_eq!(entity_vector.fields()[2].name(), Some("capacity"));
        assert_eq!(entity_vector.fields()[2].offset_bytes(), 12);
        assert_eq!(entity_vector.fields()[2].kind(), &TypeKind::S32);

        assert_eq!(world_struct.fields()[1].name(), Some("frozenEntities"));
        assert_eq!(world_struct.fields()[1].offset_bytes(), 16);
        assert_eq!(world_struct.fields()[1].kind(), world_struct.fields()[0].kind());
    }
}
