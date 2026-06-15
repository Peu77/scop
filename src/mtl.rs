use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

#[derive(Clone, Debug, Default)]
pub struct Material {
    pub diffuse_texture: Option<PathBuf>,
}

pub fn load(path: &Path) -> Result<HashMap<String, Material>> {
    let source = fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse(&source, path.parent().unwrap_or_else(|| Path::new("")))
}

fn parse(source: &str, directory: &Path) -> Result<HashMap<String, Material>> {
    let mut materials: HashMap<String, Material> = HashMap::new();
    let mut current_name = None;

    for (line_index, raw_line) in source.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let mut fields = line.split_whitespace();
        match fields.next() {
            Some("newmtl") => {
                let name = fields.collect::<Vec<_>>().join(" ");
                if name.is_empty() {
                    return Err(mtl_error(line_number, "missing material name"));
                }
                materials.entry(name.clone()).or_default();
                current_name = Some(name);
            }
            Some("map_Kd") => {
                let name = current_name
                    .as_ref()
                    .ok_or_else(|| mtl_error(line_number, "map_Kd appears before newmtl"))?;
                let texture = parse_texture_path(fields.collect(), line_number)?;
                materials.entry(name.clone()).or_default().diffuse_texture =
                    Some(directory.join(texture));
            }
            _ => {}
        }
    }

    Ok(materials)
}

fn parse_texture_path(fields: Vec<&str>, line: usize) -> Result<PathBuf> {
    let Some(path) = fields.last() else {
        return Err(mtl_error(line, "missing map_Kd texture path"));
    };
    Ok(PathBuf::from(path))
}

fn mtl_error(line: usize, message: impl Into<String>) -> AppError {
    AppError::Mtl {
        line,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::parse;

    #[test]
    fn parse_resolves_diffuse_texture_relative_to_mtl_directory() {
        let materials = parse(
            "newmtl painted\nmap_Kd textures/paint.ppm\n",
            Path::new("models"),
        )
        .unwrap();

        assert_eq!(
            materials["painted"].diffuse_texture,
            Some(PathBuf::from("models/textures/paint.ppm"))
        );
    }

    #[test]
    fn parse_uses_last_map_kd_field_after_options() {
        let materials = parse(
            "newmtl painted\nmap_Kd -s 1 1 1 paint.ppm\n",
            Path::new("models"),
        )
        .unwrap();

        assert_eq!(
            materials["painted"].diffuse_texture,
            Some(PathBuf::from("models/paint.ppm"))
        );
    }

    #[test]
    fn parse_rejects_texture_before_material_declaration() {
        let error = parse("map_Kd paint.ppm\n", Path::new("models")).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid MTL at line 1: map_Kd appears before newmtl"
        );
    }
}
