use glium::{Display, Program, ProgramCreationError};
use msgbox::IconType;

pub fn compile(display: &Display, vertex: &str, fragment: &str, geometry: Option<&str>) -> Program {
    match Program::from_source(display, vertex, fragment, geometry) {
        Ok(program) => program,
        Err(e) => {
            match e {
                ProgramCreationError::CompilationError(message, shader) => {
                    let message = format!("Error compiling {:?} Shader from source:\n\n{}", shader, message);
                    msgbox::create("Shader compilation error", &message, IconType::Error).unwrap();
                    Err(ProgramCreationError::CompilationError(message, shader)).unwrap()
                }
                other => {
                    Err(other).unwrap()
                }
            }
        }
    }
}