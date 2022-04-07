use std::collections::HashMap;
use std::rc::Rc;
use glium::{Display, Program, ProgramCreationError};

pub fn compile(display: &Display, vertex: &str, fragment: &str, geometry: Option<&str>) -> Program {
    match Program::from_source(display, vertex, fragment, geometry) {
        Ok(program) => program,
        Err(e) => {
            match e {
                ProgramCreationError::CompilationError(message, shader) => {
                    eprintln!("Error compiling {:?} Shader from source:\n\n{}", shader, message);
                    Err(ProgramCreationError::CompilationError(message, shader)).unwrap()
                }
                other => {
                    Err(other).unwrap()
                }
            }
        }
    }
}

#[macro_export]
macro_rules! shader {
    ($display:expr, $name:literal) => {{
        use glium::program::Program;
        Program::from_source($display,
            &include_str!(concat!("../../resources/shaders/", $name, ".vsh")),
            &include_str!(concat!("../../resources/shaders/", $name, ".fsh")),
            None
        ).expect(concat!("Unable to compile `", $name, "` shader"))
    }};
}

pub struct ShaderManager {
    display: Display,
    programs: HashMap<String, Rc<Box<Program>>>
}

impl ShaderManager {
    pub fn new(display: &Display) -> ShaderManager {
        let mut programs = HashMap::new();
        programs.insert("font".into(), Rc::new(Box::new(
            shader!(display, "font")
        )));
        programs.insert("default".into(), Rc::new(Box::new(
            shader!(display, "default")
        )));
        programs.insert("textured".into(), Rc::new(Box::new(
            shader!(display, "textured")
        )));
        programs.insert("water".into(), Rc::new(Box::new(
            shader!(display, "water")
        )));

        ShaderManager {
            display: display.clone(),
            programs
        }
    }

    pub fn font(&self) -> Rc<Box<Program>> {
        self.programs.get("font".into()).cloned().expect("Font shader is missing")
    }

    pub fn default(&self) -> Rc<Box<Program>> {
        self.programs.get("default".into()).cloned().expect("Default shader is missing")
    }

    pub fn light(&self) -> Rc<Box<Program>> {
        self.programs.get("light".into()).cloned().expect("Light shader is missing")
    }

    pub fn textured(&self) -> Rc<Box<Program>> {
        self.programs.get("textured".into()).cloned().expect("Textured shader is missing")
    }

    pub fn water(&self) -> Rc<Box<Program>> {
        self.programs.get("water".into()).cloned().expect("Water shader is missing")
    }
}