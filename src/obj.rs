use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use crate::error::{AppError, Result};
use crate::math::{Vec2, Vec3};
use crate::mesh::{Mesh, Vertex};

#[derive(Clone, Copy, Debug)]
struct FaceIndex {
    position: usize,
    texture: Option<usize>,
}

pub fn load(path: &Path) -> Result<Mesh> {
    let source = fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse(&source)
}

fn parse(source: &str) -> Result<Mesh> {
    let mut positions = Vec::new();
    let mut texture_coordinates = Vec::new();
    let mut faces = Vec::new();

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
            Some("f") => {
                let face = fields
                    .map(|field| {
                        parse_face_index(
                            field,
                            positions.len(),
                            texture_coordinates.len(),
                            line_number,
                        )
                    })
                    .collect::<Result<Vec<_>>>()?;
                if face.len() < 3 {
                    return Err(obj_error(
                        line_number,
                        "a face needs at least three vertices",
                    ));
                }
                faces.push(face);
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

    let (center, scale) = normalization(&positions)?;
    let normalized = positions
        .iter()
        .map(|position| (*position - center) * scale)
        .collect::<Vec<_>>();

    let triangle_count: usize = faces.iter().map(|face| face.len() - 2).sum();
    let mut vertices = Vec::with_capacity(triangle_count * 3);

    for (face_index, face) in faces.into_iter().enumerate() {
        let shade = face_shade(face_index);
        for index in 1..face.len() - 1 {
            let triangle = [face[0], face[index], face[index + 1]];
            for face_index in triangle {
                let position = normalized[face_index.position];
                let uv = face_index
                    .texture
                    .map(|texture| texture_coordinates[texture])
                    .unwrap_or_else(|| generated_uv(position));
                vertices.push(Vertex {
                    position,
                    color: Vec3::new(shade, shade, shade),
                    uv,
                });
            }
        }
    }

    Ok(Mesh { vertices })
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
    line: usize,
) -> Result<FaceIndex> {
    let mut parts = field.split('/');
    let position = resolve_index(parts.next().unwrap_or(""), position_count, line, "vertex")?;
    let texture = match parts.next() {
        Some("") | None => None,
        Some(value) => Some(resolve_index(value, texture_count, line, "texture")?),
    };

    Ok(FaceIndex { position, texture })
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

fn face_shade(index: usize) -> f32 {
    const SHADES: [f32; 6] = [0.28, 0.40, 0.52, 0.64, 0.76, 0.88];
    SHADES[index % SHADES.len()]
}

fn obj_error(line: usize, message: impl Into<String>) -> AppError {
    AppError::Obj {
        line,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::parse;

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
}
