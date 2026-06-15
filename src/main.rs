mod app;
mod error;
mod input;
mod math;
mod mesh;
mod mtl;
mod obj;
mod ppm;
mod renderer;
mod shader;

use std::path::PathBuf;

use error::{AppError, Result};

fn main() {
    if let Err(error) = run() {
        eprintln!("scop: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut arguments = std::env::args_os();
    let executable = arguments.next().unwrap_or_default();
    let model_path = arguments.next().map(PathBuf::from).ok_or_else(|| {
        AppError::Usage(format!(
            "usage: {} <model.obj>",
            PathBuf::from(executable).display()
        ))
    })?;

    if arguments.next().is_some() {
        return Err(AppError::Usage(
            "usage: scop <model.obj> (exactly one model is required)".into(),
        ));
    }

    let mesh = obj::load(&model_path)?;
    app::run(mesh)
}
