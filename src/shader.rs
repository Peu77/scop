use std::ffi::{CStr, CString};
use std::ptr;

use crate::error::{AppError, Result};

pub fn load_program(vertex_source: &str, fragment_source: &str) -> Result<u32> {
    let vertex = compile(gl::VERTEX_SHADER, vertex_source)?;
    let fragment = match compile(gl::FRAGMENT_SHADER, fragment_source) {
        Ok(shader) => shader,
        Err(error) => {
            unsafe { gl::DeleteShader(vertex) };
            return Err(error);
        }
    };

    let program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program, vertex);
        gl::AttachShader(program, fragment);
        gl::LinkProgram(program);
        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);
    }

    let mut success = 0;
    unsafe { gl::GetProgramiv(program, gl::LINK_STATUS, &mut success) };

    if success == gl::FALSE as i32 {
        let log = program_log(program);

        unsafe { gl::DeleteProgram(program) };
        return Err(AppError::OpenGl(format!(
            "could not link shader program: {log}"
        )));
    }

    Ok(program)
}

fn compile(kind: u32, source: &str) -> Result<u32> {
    let source = CString::new(source)
        .map_err(|_| AppError::OpenGl("shader source contains a NUL byte".into()))?;
    let shader = unsafe { gl::CreateShader(kind) };

    unsafe {
        gl::ShaderSource(shader, 1, &source.as_ptr(), ptr::null());
        gl::CompileShader(shader);
    }

    let mut success = 0;
    unsafe { gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success) };

    if success == gl::FALSE as i32 {
        let log = shader_log(shader);

        unsafe { gl::DeleteShader(shader) };
        return Err(AppError::OpenGl(format!("could not compile shader: {log}")));
    }

    Ok(shader)
}

fn shader_log(shader: u32) -> String {
    let mut length = 0;
    unsafe { gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut length) };

    read_log(length, |buffer| unsafe {
        gl::GetShaderInfoLog(shader, length, ptr::null_mut(), buffer)
    })
}

fn program_log(program: u32) -> String {
    let mut length = 0;
    unsafe { gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut length) };

    read_log(length, |buffer| unsafe {
        gl::GetProgramInfoLog(program, length, ptr::null_mut(), buffer)
    })
}

fn read_log(length: i32, read: impl FnOnce(*mut i8)) -> String {
    if length <= 1 {
        return "no driver log was provided".into();
    }
    let mut buffer = vec![0_i8; length as usize];
    read(buffer.as_mut_ptr());
    unsafe { CStr::from_ptr(buffer.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}
