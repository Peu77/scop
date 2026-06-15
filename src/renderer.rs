use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;

use crate::error::{AppError, Result};
use crate::math::Mat4;
use crate::mesh::{DrawBatch, Mesh, Vertex};
use crate::ppm::Image;
use crate::shader;

const VERTEX_SHADER: &str = include_str!("shaders/model.vert");
const FRAGMENT_SHADER: &str = include_str!("shaders/model.frag");

pub struct Renderer {
    program: u32,
    vertex_array: u32,
    vertex_buffer: u32,
    textures: Vec<u32>,
    batches: Vec<RenderBatch>,
    fallback_texture: usize,
    use_fallback_for_untextured: bool,
    mvp_location: i32,
    blend_location: i32,
}

#[derive(Clone, Copy)]
struct RenderBatch {
    first_vertex: i32,
    vertex_count: i32,
    texture: Option<usize>,
}

impl Renderer {
    pub fn new(mesh: &Mesh, material_textures: &[Image], fallback: &Image) -> Result<Self> {
        if material_textures.len() != mesh.textures.len() {
            return Err(AppError::OpenGl(
                "material texture count does not match the mesh".into(),
            ));
        }
        for image in material_textures.iter().chain(std::iter::once(fallback)) {
            validate_texture_dimensions(image)?;
        }
        let batches = mesh
            .batches
            .iter()
            .copied()
            .map(RenderBatch::try_from)
            .collect::<Result<Vec<_>>>()?;

        let program = shader::load_program(VERTEX_SHADER, FRAGMENT_SHADER)?;
        let mut vertex_array = 0;
        let mut vertex_buffer = 0;

        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array);
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::BindVertexArray(vertex_array);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of_val(mesh.vertices.as_slice()) as isize,
                mesh.vertices.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            let stride = size_of::<Vertex>() as i32;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (size_of::<f32>() * 3) as *const c_void,
            );
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (size_of::<f32>() * 6) as *const c_void,
            );

            gl::UseProgram(program);
            gl::Uniform1i(gl::GetUniformLocation(program, c"u_texture".as_ptr()), 0);
            gl::Enable(gl::DEPTH_TEST);
        }

        let mut textures = Vec::with_capacity(material_textures.len() + 1);
        for image in material_textures.iter().chain(std::iter::once(fallback)) {
            textures.push(upload_texture(image));
        }
        let fallback_texture = textures.len() - 1;

        let mvp_location = unsafe { gl::GetUniformLocation(program, c"u_mvp".as_ptr()) };
        let blend_location =
            unsafe { gl::GetUniformLocation(program, c"u_texture_blend".as_ptr()) };
        if mvp_location < 0 || blend_location < 0 {
            unsafe {
                gl::DeleteTextures(textures.len() as i32, textures.as_ptr());
                gl::DeleteBuffers(1, &vertex_buffer);
                gl::DeleteVertexArrays(1, &vertex_array);
                gl::DeleteProgram(program);
            }
            return Err(AppError::OpenGl(
                "a required shader uniform could not be found".into(),
            ));
        }

        Ok(Self {
            program,
            vertex_array,
            vertex_buffer,
            textures,
            batches,
            fallback_texture,
            use_fallback_for_untextured: !mesh.has_material_library,
            mvp_location,
            blend_location,
        })
    }

    pub fn draw(&self, mvp: &Mat4, texture_blend: f32, framebuffer_size: (i32, i32)) {
        unsafe {
            gl::Viewport(0, 0, framebuffer_size.0, framebuffer_size.1);
            gl::ClearColor(0.055, 0.06, 0.075, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UseProgram(self.program);
            gl::UniformMatrix4fv(self.mvp_location, 1, gl::FALSE, mvp.as_ptr());
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindVertexArray(self.vertex_array);
            for batch in &self.batches {
                let texture = batch.texture.or_else(|| {
                    self.use_fallback_for_untextured
                        .then_some(self.fallback_texture)
                });
                gl::Uniform1f(
                    self.blend_location,
                    if texture.is_some() {
                        texture_blend
                    } else {
                        0.0
                    },
                );
                if let Some(texture) = texture {
                    gl::BindTexture(gl::TEXTURE_2D, self.textures[texture]);
                }
                gl::DrawArrays(gl::TRIANGLES, batch.first_vertex, batch.vertex_count);
            }
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(self.textures.len() as i32, self.textures.as_ptr());
            gl::DeleteBuffers(1, &self.vertex_buffer);
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteProgram(self.program);
        }
    }
}

impl TryFrom<DrawBatch> for RenderBatch {
    type Error = AppError;

    fn try_from(batch: DrawBatch) -> Result<Self> {
        Ok(Self {
            first_vertex: i32::try_from(batch.first_vertex)
                .map_err(|_| AppError::OpenGl("the model has too many vertices".into()))?,
            vertex_count: i32::try_from(batch.vertex_count)
                .map_err(|_| AppError::OpenGl("a draw batch has too many vertices".into()))?,
            texture: batch.texture,
        })
    }
}

fn validate_texture_dimensions(image: &Image) -> Result<()> {
    i32::try_from(image.width).map_err(|_| AppError::OpenGl("a texture is too wide".into()))?;
    i32::try_from(image.height).map_err(|_| AppError::OpenGl("a texture is too tall".into()))?;
    Ok(())
}

fn upload_texture(image: &Image) -> u32 {
    let mut texture = 0;
    let pixels = flip_rows(image);
    unsafe {
        gl::GenTextures(1, &mut texture);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGB as i32,
            image.width as i32,
            image.height as i32,
            0,
            gl::RGB,
            gl::UNSIGNED_BYTE,
            pixels.as_ptr().cast(),
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }
    texture
}

fn flip_rows(image: &Image) -> Vec<u8> {
    let row_size = image.width as usize * 3;
    let mut pixels = Vec::with_capacity(image.pixels.len());
    for row in image.pixels.chunks_exact(row_size).rev() {
        pixels.extend_from_slice(row);
    }
    pixels
}

#[cfg(test)]
mod tests {
    use crate::ppm::Image;

    use super::flip_rows;

    #[test]
    fn flip_rows_converts_ppm_top_left_origin_for_opengl() {
        let image = Image {
            width: 1,
            height: 2,
            pixels: vec![1, 2, 3, 4, 5, 6],
        };

        assert_eq!(flip_rows(&image), [4, 5, 6, 1, 2, 3]);
    }
}
