use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Glfw(String),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Obj {
        line: usize,
        message: String,
    },
    OpenGl(String),
    Ppm(String),
    Usage(String),
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Glfw(message) => write!(formatter, "GLFW error: {message}"),
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Obj { line, message } => {
                write!(formatter, "invalid OBJ at line {line}: {message}")
            }
            Self::OpenGl(message) => write!(formatter, "OpenGL error: {message}"),
            Self::Ppm(message) => write!(formatter, "invalid PPM: {message}"),
            Self::Usage(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for AppError {}
