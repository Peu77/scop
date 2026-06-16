use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};
use crate::math::{Vec2, Vec3};
use crate::mesh::{DrawBatch, Mesh, Vertex};
use crate::mtl::{self, Material};

#[derive(Clone, Copy, Debug)]
struct FaceIndex {
    position: usize,
    texture: Option<usize>,
    normal: Option<usize>,
}

#[derive(Debug)]
struct Face {
    indices: Vec<FaceIndex>,
    material: Option<String>,
}

#[derive(Debug)]
struct ObjDocument {
    positions: Vec<Vec3>,
    texture_coordinates: Vec<Vec2>,
    normals: Vec<Vec3>,
    faces: Vec<Face>,
    material_libraries: Vec<PathBuf>,
}

pub fn load(path: &Path) -> Result<Mesh> {
    let source = fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let document = parse_document(&source)?;
    let directory = path.parent().unwrap_or_else(|| Path::new(""));
    let mut materials = HashMap::new();
    for library in &document.material_libraries {
        materials.extend(mtl::load(&directory.join(library))?);
    }
    build_mesh(document, &materials)
}

#[cfg(test)]
fn parse(source: &str) -> Result<Mesh> {
    build_mesh(parse_document(source)?, &HashMap::new())
}

fn parse_document(source: &str) -> Result<ObjDocument> {
    let mut positions = Vec::new();
    let mut texture_coordinates = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Vec::new();
    let mut material_libraries = Vec::new();
    let mut current_material = None;

    for (line_index, raw_line) in source.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let mut fields = line.split_whitespace();
        match fields.next() {
            Some("v") => positions.push(parse_position(fields, line_number)?),
            Some("vt") => texture_coordinates.push(parse_texture_coordinate(fields, line_number)?),
            Some("vn") => normals.push(parse_position(fields, line_number)?.normalized()),
            Some("mtllib") => {
                let libraries = fields.map(PathBuf::from).collect::<Vec<_>>();
                if libraries.is_empty() {
                    return Err(obj_error(line_number, "missing material library path"));
                }
                material_libraries.extend(libraries);
            }
            Some("usemtl") => {
                let name = fields.collect::<Vec<_>>().join(" ");
                if name.is_empty() {
                    return Err(obj_error(line_number, "missing material name"));
                }
                current_material = Some(name);
            }
            Some("f") => {
                let indices = fields
                    .map(|field| {
                        parse_face_index(
                            field,
                            positions.len(),
                            texture_coordinates.len(),
                            normals.len(),
                            line_number,
                        )
                    })
                    .collect::<Result<Vec<_>>>()?;
                if indices.len() < 3 {
                    return Err(obj_error(
                        line_number,
                        "a face needs at least three vertices",
                    ));
                }
                faces.push(Face {
                    indices,
                    material: current_material.clone(),
                });
            }
            _ => {}
        }
    }

    if positions.is_empty() {
        return Err(obj_error(0, "the model does not contain any vertices"));
    }
    if faces.is_empty() {
        return Err(obj_error(0, "the model does not contain any faces"));
    }

    Ok(ObjDocument {
        positions,
        texture_coordinates,
        normals,
        faces,
        material_libraries,
    })
}

fn build_mesh(document: ObjDocument, materials: &HashMap<String, Material>) -> Result<Mesh> {
    let has_material_library = !document.material_libraries.is_empty();
    let (center, scale) = normalization(&document.positions)?;
    let normalized = document
        .positions
        .iter()
        .map(|position| (*position - center) * scale)
        .collect::<Vec<_>>();

    let triangle_count: usize = document
        .faces
        .iter()
        .map(|face| face.indices.len() - 2)
        .sum();
    let mut vertices = Vec::with_capacity(triangle_count * 3);
    let mut textures = Vec::new();
    let mut texture_indices = HashMap::new();
    let mut normal_maps = Vec::new();
    let mut normal_map_indices = HashMap::new();
    let mut batches = Vec::new();

    for (face_index, face) in document.faces.into_iter().enumerate() {
        let color = material_color(face.material.as_deref(), materials)
            .unwrap_or_else(|| generated_face_color(face_index));
        let texture = resolve_material_texture(
            face.material.as_deref(),
            materials,
            &mut textures,
            &mut texture_indices,
            MaterialTexture::Diffuse,
        )?;
        let normal_map = resolve_material_texture(
            face.material.as_deref(),
            materials,
            &mut normal_maps,
            &mut normal_map_indices,
            MaterialTexture::Normal,
        )?;
        let first_vertex = vertices.len();
        for index in 1..face.indices.len() - 1 {
            let triangle = [
                face.indices[0],
                face.indices[index],
                face.indices[index + 1],
            ];
            let positions = triangle.map(|face_index| normalized[face_index.position]);
            let uvs = triangle.map(|face_index| {
                face_index
                    .texture
                    .map(|texture| document.texture_coordinates[texture])
                    .unwrap_or_else(|| generated_uv(normalized[face_index.position]))
            });
            let face_normal = (positions[1] - positions[0])
                .cross(positions[2] - positions[0])
                .normalized();
            let (tangent, bitangent) = tangent_basis(positions, uvs, face_normal);

            for face_index in triangle {
                let position = normalized[face_index.position];
                let uv = face_index
                    .texture
                    .map(|texture| document.texture_coordinates[texture])
                    .unwrap_or_else(|| generated_uv(position));
                let normal = face_index
                    .normal
                    .map(|normal| document.normals[normal])
                    .unwrap_or(face_normal);
                vertices.push(Vertex {
                    position,
                    color,
                    uv,
                    normal,
                    tangent,
                    bitangent,
                });
            }
        }
        push_batch(
            &mut batches,
            DrawBatch {
                first_vertex,
                vertex_count: vertices.len() - first_vertex,
                texture,
                normal_map,
            },
        );
    }

    Ok(Mesh {
        vertices,
        textures,
        normal_maps,
        batches,
        has_material_library,
    })
}

fn resolve_material_texture(
    material_name: Option<&str>,
    materials: &HashMap<String, Material>,
    textures: &mut Vec<PathBuf>,
    texture_indices: &mut HashMap<PathBuf, usize>,
    kind: MaterialTexture,
) -> Result<Option<usize>> {
    let Some(name) = material_name else {
        return Ok(None);
    };
    let material = materials
        .get(name)
        .ok_or_else(|| obj_error(0, format!("material '{name}' is not defined")))?;
    let path = match kind {
        MaterialTexture::Diffuse => &material.diffuse_texture,
        MaterialTexture::Normal => &material.normal_texture,
    };
    let Some(path) = path else {
        return Ok(None);
    };
    if let Some(&index) = texture_indices.get(path) {
        return Ok(Some(index));
    }

    let index = textures.len();
    textures.push(path.clone());
    texture_indices.insert(path.clone(), index);
    Ok(Some(index))
}

fn material_color(
    material_name: Option<&str>,
    materials: &HashMap<String, Material>,
) -> Option<Vec3> {
    material_name
        .and_then(|name| materials.get(name))
        .and_then(|material| material.diffuse_color)
}

#[derive(Clone, Copy)]
enum MaterialTexture {
    Diffuse,
    Normal,
}

fn push_batch(batches: &mut Vec<DrawBatch>, batch: DrawBatch) {
    if let Some(previous) = batches.last_mut() {
        if previous.texture == batch.texture
            && previous.normal_map == batch.normal_map
            && previous.first_vertex + previous.vertex_count == batch.first_vertex
        {
            previous.vertex_count += batch.vertex_count;
            return;
        }
    }
    batches.push(batch);
}

fn parse_position<'a>(mut fields: impl Iterator<Item = &'a str>, line: usize) -> Result<Vec3> {
    let x = parse_float(fields.next(), line, "vertex x")?;
    let y = parse_float(fields.next(), line, "vertex y")?;
    let z = parse_float(fields.next(), line, "vertex z")?;
    Ok(Vec3::new(x, y, z))
}

fn parse_texture_coordinate<'a>(
    mut fields: impl Iterator<Item = &'a str>,
    line: usize,
) -> Result<Vec2> {
    let u = parse_float(fields.next(), line, "texture u")?;
    let v = parse_float(fields.next(), line, "texture v")?;
    Ok(Vec2::new(u, v))
}

fn parse_float(value: Option<&str>, line: usize, label: &str) -> Result<f32> {
    value
        .ok_or_else(|| obj_error(line, format!("missing {label}")))?
        .parse()
        .map_err(|_| obj_error(line, format!("invalid {label}")))
}

fn parse_face_index(
    field: &str,
    position_count: usize,
    texture_count: usize,
    normal_count: usize,
    line: usize,
) -> Result<FaceIndex> {
    let mut parts = field.split('/');
    let position = resolve_index(parts.next().unwrap_or(""), position_count, line, "vertex")?;
    let texture = match parts.next() {
        Some("") | None => None,
        Some(value) => Some(resolve_index(value, texture_count, line, "texture")?),
    };
    let normal = match parts.next() {
        Some("") | None => None,
        Some(value) => Some(resolve_index(value, normal_count, line, "normal")?),
    };

    Ok(FaceIndex {
        position,
        texture,
        normal,
    })
}

fn resolve_index(value: &str, count: usize, line: usize, label: &str) -> Result<usize> {
    let parsed = value
        .parse::<isize>()
        .map_err(|_| obj_error(line, format!("invalid {label} index '{value}'")))?;
    let resolved = match parsed.cmp(&0) {
        Ordering::Greater => parsed - 1,
        Ordering::Less => count as isize + parsed,
        Ordering::Equal => {
            return Err(obj_error(line, format!("{label} indices start at 1")));
        }
    };

    if resolved < 0 || resolved as usize >= count {
        return Err(obj_error(
            line,
            format!("{label} index '{value}' is out of bounds"),
        ));
    }
    Ok(resolved as usize)
}

fn normalization(positions: &[Vec3]) -> Result<(Vec3, f32)> {
    let mut minimum = positions[0];
    let mut maximum = positions[0];
    for &position in &positions[1..] {
        minimum = minimum.min(position);
        maximum = maximum.max(position);
    }

    let extent = maximum - minimum;
    let largest_extent = extent.x.max(extent.y).max(extent.z);
    if largest_extent <= f32::EPSILON {
        return Err(obj_error(0, "the model has no measurable size"));
    }

    Ok(((minimum + maximum) * 0.5, 2.0 / largest_extent))
}

fn generated_uv(position: Vec3) -> Vec2 {
    let longitude = position.z.atan2(position.x);
    let radius = (position.x * position.x + position.y * position.y + position.z * position.z)
        .sqrt()
        .max(f32::EPSILON);
    let latitude = (position.y / radius).clamp(-1.0, 1.0).asin();
    Vec2::new(
        longitude / (2.0 * std::f32::consts::PI) + 0.5,
        latitude / std::f32::consts::PI + 0.5,
    )
}

fn tangent_basis(positions: [Vec3; 3], uvs: [Vec2; 3], normal: Vec3) -> (Vec3, Vec3) {
    let edge1 = positions[1] - positions[0];
    let edge2 = positions[2] - positions[0];
    let delta_uv1 = Vec2::new(uvs[1].x - uvs[0].x, uvs[1].y - uvs[0].y);
    let delta_uv2 = Vec2::new(uvs[2].x - uvs[0].x, uvs[2].y - uvs[0].y);
    let determinant = delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y;

    if determinant.abs() <= f32::EPSILON {
        return fallback_tangent_basis(normal);
    }

    let scale = 1.0 / determinant;
    let tangent = ((edge1 * delta_uv2.y) - (edge2 * delta_uv1.y)) * scale;
    let bitangent = ((edge2 * delta_uv1.x) - (edge1 * delta_uv2.x)) * scale;
    (tangent.normalized(), bitangent.normalized())
}

fn fallback_tangent_basis(normal: Vec3) -> (Vec3, Vec3) {
    let helper = if normal.y.abs() < 0.9 {
        Vec3::new(0.0, 1.0, 0.0)
    } else {
        Vec3::new(1.0, 0.0, 0.0)
    };
    let tangent = helper.cross(normal).normalized();
    let bitangent = normal.cross(tangent).normalized();
    (tangent, bitangent)
}

fn generated_face_color(index: usize) -> Vec3 {
    const SHADES: [f32; 6] = [0.28, 0.40, 0.52, 0.64, 0.76, 0.88];
    let shade = SHADES[index % SHADES.len()];
    Vec3::new(shade, shade, shade)
}

fn obj_error(line: usize, message: impl Into<String>) -> AppError {
    AppError::Obj {
        line,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use crate::math::Vec3;
    use crate::mtl::Material;

    use super::{build_mesh, load, parse, parse_document};

    #[test]
    fn triangulates_polygons_and_centers_the_model() {
        let mesh = parse(
            "\
            v 10 0 0\n\
            v 12 0 0\n\
            v 12 2 0\n\
            v 10 2 0\n\
            f 1 2 3 4\n",
        )
        .unwrap();

        assert_eq!(mesh.vertices.len(), 6);
        assert_eq!(mesh.vertices[0].position.x, -1.0);
        assert_eq!(mesh.vertices[0].position.y, -1.0);
    }

    #[test]
    fn accepts_negative_indices_and_texture_coordinates() {
        let mesh = parse(
            "\
            v 0 0 0\n\
            v 1 0 0\n\
            v 0 1 0\n\
            vt 0.1 0.2\n\
            vt 0.9 0.2\n\
            vt 0.1 0.8\n\
            f -3/-3 -2/-2 -1/-1\n",
        )
        .unwrap();

        assert_eq!(mesh.vertices[0].uv.x, 0.1);
        assert_eq!(mesh.vertices[2].uv.y, 0.8);
    }

    #[test]
    fn usemtl_assigns_diffuse_texture_to_draw_batch() {
        let document = parse_document(
            "\
            mtllib painted.mtl\n\
            v 0 0 0\n\
            v 1 0 0\n\
            v 0 1 0\n\
            usemtl painted\n\
            f 1 2 3\n",
        )
        .unwrap();
        let materials = HashMap::from([(
            "painted".into(),
            Material {
                diffuse_color: None,
                diffuse_texture: Some(PathBuf::from("paint.ppm")),
                normal_texture: None,
            },
        )]);

        let mesh = build_mesh(document, &materials).unwrap();

        assert_eq!(mesh.batches[0].texture, Some(0));
        assert_eq!(mesh.textures, [PathBuf::from("paint.ppm")]);
        assert!(mesh.has_material_library);
    }

    #[test]
    fn consecutive_faces_with_same_material_share_draw_batch() {
        let document = parse_document(
            "\
            v 0 0 0\n\
            v 1 0 0\n\
            v 0 1 0\n\
            v 1 1 0\n\
            usemtl painted\n\
            f 1 2 3\n\
            f 2 4 3\n",
        )
        .unwrap();
        let materials = HashMap::from([(
            "painted".into(),
            Material {
                diffuse_color: None,
                diffuse_texture: Some(PathBuf::from("paint.ppm")),
                normal_texture: None,
            },
        )]);

        let mesh = build_mesh(document, &materials).unwrap();

        assert_eq!(mesh.batches.len(), 1);
        assert_eq!(mesh.batches[0].vertex_count, 6);
    }

    #[test]
    fn undefined_material_returns_obj_error() {
        let document = parse_document(
            "\
            v 0 0 0\n\
            v 1 0 0\n\
            v 0 1 0\n\
            usemtl missing\n\
            f 1 2 3\n",
        )
        .unwrap();

        let error = build_mesh(document, &HashMap::new()).unwrap_err();

        assert_eq!(
            error.to_string(),
            "invalid OBJ at line 0: material 'missing' is not defined"
        );
    }

    #[test]
    fn usemtl_assigns_diffuse_color_to_vertices() {
        let document = parse_document(
            "\
            mtllib tree.mtl\n\
            v 0 0 0\n\
            v 1 0 0\n\
            v 0 1 0\n\
            usemtl bark\n\
            f 1 2 3\n",
        )
        .unwrap();
        let materials = HashMap::from([(
            "bark".into(),
            Material {
                diffuse_color: Some(Vec3::new(0.2, 0.1, 0.05)),
                diffuse_texture: None,
                normal_texture: None,
            },
        )]);

        let mesh = build_mesh(document, &materials).unwrap();

        assert_eq!(mesh.vertices[0].color, Vec3::new(0.2, 0.1, 0.05));
    }

    #[test]
    fn load_resolves_42_material_texture_from_mtl_file() {
        let mesh = load(std::path::Path::new("assets/42.obj")).unwrap();

        assert_eq!(mesh.textures, [PathBuf::from("assets/unicorn.ppm")]);
    }

    #[test]
    fn load_resolves_cottage_diffuse_texture_from_mtl_file() {
        let mesh = load(std::path::Path::new("assets/cottage_obj.obj")).unwrap();

        assert_eq!(mesh.textures, [PathBuf::from("assets/cottage_diffuse.ppm")]);
        assert_eq!(
            mesh.normal_maps,
            [PathBuf::from("assets/cottage_normal.ppm")]
        );
    }

    #[test]
    fn load_resolves_teapot_diffuse_and_normal_textures_from_mtl_file() {
        let mesh = load(std::path::Path::new("assets/teapot2.obj")).unwrap();

        assert_eq!(mesh.textures, [PathBuf::from("assets/texture.ppm")]);
        assert_eq!(
            mesh.normal_maps,
            [PathBuf::from("assets/cottage_normal.ppm")]
        );
    }
}
