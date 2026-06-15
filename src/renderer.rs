use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;

use crate::error::{AppError, Result};
use crate::math::Mat4;
use crate::mesh::{Mesh, Vertex};
use crate::ppm::Image;
use crate::shader;

const VERTEX_SHADER: &str = include_str!("shaders/model.vert");
const FRAGMENT_SHADER: &str = include_str!("shaders/model.frag");

pub struct Renderer {
    program: u32,
    vertex_array: u32,
    vertex_buffer: u32,
    texture: u32,
    vertex_count: i32,
    mvp_location: i32,
    blend_location: i32,
}

impl Renderer {
    pub fn new(mesh: &Mesh, texture: &Image) -> Result<Self> {
        let vertex_count = i32::try_from(mesh.vertices.len())
            .map_err(|_| AppError::OpenGl("the model has too many vertices".into()))?;
        let texture_width = i32::try_from(texture.width)
            .map_err(|_| AppError::OpenGl("the texture is too wide".into()))?;
        let texture_height = i32::try_from(texture.height)
            .map_err(|_| AppError::OpenGl("the texture is too tall".into()))?;

        let program = shader::load_program(VERTEX_SHADER, FRAGMENT_SHADER)?;
        let mut vertex_array = 0;
        let mut vertex_buffer = 0;
        let mut texture_id = 0;

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

            gl::GenTextures(1, &mut texture_id);
            gl::BindTexture(gl::TEXTURE_2D, texture_id);
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
                texture_width,
                texture_height,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                texture.pixels.as_ptr().cast(),
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);

            gl::UseProgram(program);
            gl::Uniform1i(gl::GetUniformLocation(program, c"u_texture".as_ptr()), 0);
            gl::Enable(gl::DEPTH_TEST);
        }

        let mvp_location = unsafe { gl::GetUniformLocation(program, c"u_mvp".as_ptr()) };
        let blend_location =
            unsafe { gl::GetUniformLocation(program, c"u_texture_blend".as_ptr()) };
        if mvp_location < 0 || blend_location < 0 {
            unsafe {
                gl::DeleteTextures(1, &texture_id);
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
            texture: texture_id,
            vertex_count,
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
            gl::Uniform1f(self.blend_location, texture_blend);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::BindVertexArray(self.vertex_array);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertex_count);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteBuffers(1, &self.vertex_buffer);
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteProgram(self.program);
        }
    }
}
