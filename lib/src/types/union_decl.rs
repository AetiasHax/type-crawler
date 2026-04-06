use std::fmt::Display;

use crate::{
    Env, Field, TypeKind, TypePath, Types,
    error::{ExnExt, OptionExt, ResultExt, bail_str, error_type},
};

error_type!(UnionDeclError);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnionDecl {
    pub(crate) path: Option<TypePath>,
    fields: Vec<Field>,
    size: usize,
    alignment: usize,
}

impl UnionDecl {
    pub fn new(
        env: &Env,
        types: &Types,
        path: Option<TypePath>,
        ty: clang::Type,
    ) -> exn::Result<Self, UnionDeclError> {
        if ty.get_kind() != clang::TypeKind::Record {
            bail_str!("Expected Record, found: {:?}", ty);
        }

        let display_name = path.clone().unwrap_or("<anon>".into());

        let record_fields =
            ty.get_fields().ok_or_raise_str(|| format!("Record type without fields: {:?}", ty))?;
        if record_fields.is_empty() {
            let declaration = ty
                .get_declaration()
                .ok_or_raise_str(|| format!("Record type without declaration: {:?}", ty))?;

            let decl_children = declaration.get_children();
            let invalid_fields = decl_children
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    c.get_kind() == clang::EntityKind::FieldDecl && c.is_invalid_declaration()
                })
                .collect::<Vec<_>>();
            if !invalid_fields.is_empty() {
                bail_str!(
                    "Invalid fields in {}: {:?}",
                    display_name,
                    invalid_fields
                        .iter()
                        .map(|(i, c)| c.get_name().unwrap_or_else(|| format!("<index#{i}>")))
                        .collect::<Vec<_>>()
                );
            }
        }

        let mut fields = Vec::new();
        for field in &record_fields {
            match field.get_kind() {
                clang::EntityKind::FieldDecl => {
                    fields.push(Field::new(env, types, field).or_raise_str(|| {
                        format!("Failed to parse AST for field in union {}", display_name)
                    })?);
                }
                _ => {
                    bail_str!("Unsupported entity in union: {:?}", field);
                }
            }
        }

        let size = ty.get_sizeof().or_else(|e| {
            if record_fields.is_empty() {
                Ok(1)
            } else {
                Err(e).or_raise_str(|| format!("Failed to get size of union {}", display_name))
            }
        })?;
        let alignment = ty.get_alignof().or_else(|e| {
            if record_fields.is_empty() {
                Ok(1)
            } else {
                Err(e).or_raise_str(|| format!("Failed to get alignment of union {}", display_name))
            }
        })?;

        Ok(UnionDecl { path, fields, size, alignment })
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn alignment(&self) -> usize {
        self.alignment
    }

    #[deprecated(note = "use path().map(|p| p.name()) instead")]
    pub fn name(&self) -> Option<&str> {
        self.path.as_ref().map(|p| p.name())
    }

    pub fn path(&self) -> Option<&TypePath> {
        self.path.as_ref()
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn get_field(&self, name: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.name() == Some(name))
    }

    pub fn replace_template_parameters<Cb>(
        &self,
        _types: &Types,
        _get_param_type: Cb,
    ) -> exn::Result<UnionDecl, UnionDeclError>
    where
        Cb: Fn(&str) -> Option<TypeKind>,
    {
        bail_str!(
            "Template specialization not implemented for template parameters in unions defined inside the template class",
        );
    }
}

impl Display for UnionDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {{", self.path.clone().unwrap_or("<anon>".into()))?;
        for field in &self.fields {
            writeln!(f, "  {field}")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}
