// C/C++ Interaction for Crabby

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_double, c_int, c_void};
use libloading::{Library, Symbol};
use periphery::sys::gpio::Value;
use std::collections::HashMap;
use crate::utils::CrabbyError;
use crate::compile::Compiler;

#[derive(Debug, Clone)]
pub enum FFIType {
    Int,
    Float,
    String,
    Void,
    Pointer(Box<FFIType>),
}

pub struct FFIFunction<'a> {
    lib: Library,
    func: Symbol<'a, unsafe extern "C" fn()>,
    arg_types: Vec<FFIType>,
    return_type: FFIType,
}

#[derive(Debug, Clone)]
pub enum FFIValue {
    Int(c_int),
    Float(c_double),
    String(CString),
    Void,
    Pointer(*mut c_void),
}

pub struct FFIManager<'a> {
    loaded_libs: HashMap<String, Library>,
    functions: HashMap<String, FFIFunction<'a>>,
}

impl FFIManager<'_> {
    pub fn new() -> Self {
        Self {
            loaded_libs: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn load_library(&mut self, path: &str) -> Result<(), CrabbyError> {
        unsafe {
            let lib = Library::new(path).map_err(|e| {
                CrabbyError::CompileError(format!("Failed to load library {}: {}", path, e))
            })?;
            self.loaded_libs.insert(path.to_string(), lib);
            Ok(())
        }
    }

    pub fn register_function(
        &mut self,
        lib_path: &str,
        func_name: &str,
        arg_types: Vec<FFIType>,
        return_type: FFIType
    ) -> Result<(), CrabbyError> {
        let lib = self.loaded_libs.get(lib_path).ok_or_else(|| {
            CrabbyError::CompileError(format!("Library {} not loaded", lib_path))
        })?;

        unsafe {
            let func: Symbol<unsafe extern "C" fn()> = lib.get(func_name.as_bytes())
                .map_err(|e| CrabbyError::CompileError(
                    format!("Failed to load function {}: {}", func_name, e)
                ))?;

            self.functions.insert(
                func_name.to_string(),
                FFIFunction {
                    lib: Library,
                    func,
                    arg_types,
                    return_type,
                }
            );
        }

        Ok(())
    }

    pub fn call_function(&self, name: &str, args: Vec<FFIValue>) -> Result<FFIValue, CrabbyError> {
        let func = self.functions.get(name).ok_or_else(|| {
            CrabbyError::CompileError(format!("Function {} not registered", name))
        })?;

        if args.len() != func.arg_types.len() {
            return Err(CrabbyError::CompileError(format!(
                "Function {} expects {} arguments, got {}",
                name,
                func.arg_types.len(),
                args.len()
            )));
        }

        let c_args: Vec<_> = args.iter().zip(&func.arg_types)
            .map(|(arg, ty)| self.convert_to_c_value(arg, ty))
            .collect::<Result<_, _>>()?;

        unsafe {
            let result = match func.return_type {
                FFIType::Int => {
                    let f: Symbol<unsafe extern "C" fn() -> c_int> =
                        std::mem::transmute(func.func.clone());
                    FFIValue::Int(f())
                },
                FFIType::Float => {
                    let f: Symbol<unsafe extern "C" fn() -> c_double> =
                        std::mem::transmute(func.func.clone());
                    FFIValue::Float(f())
                },
                FFIType::String => {
                    let f: Symbol<unsafe extern "C" fn() -> *const c_char> =
                        std::mem::transmute(func.func.clone());
                    let ptr = f();
                    let cstr = CStr::from_ptr(ptr);
                    FFIValue::String(CString::from(cstr))
                },
                FFIType::Void => {
                    let f: Symbol<unsafe extern "C" fn()> = func.func.clone();
                    f();
                    FFIValue::Void
                },
                FFIType::Pointer(_) => {
                    let f: Symbol<unsafe extern "C" fn() -> *mut c_void> =
                        std::mem::transmute(func.func.clone());
                    FFIValue::Pointer(f())
                }
            };

            Ok(result)
        }
    }

    fn convert_to_c_value(&self, value: &FFIValue, ty: &FFIType) -> Result<FFIValue, CrabbyError> {
        match (value, ty) {
            (FFIValue::Int(i), FFIType::Int) => Ok(FFIValue::Int(*i)),
            (FFIValue::Float(f), FFIType::Float) => Ok(FFIValue::Float(*f)),
            (FFIValue::String(s), FFIType::String) => Ok(FFIValue::String(s.clone())),
            (FFIValue::Pointer(p), FFIType::Pointer(_)) => Ok(FFIValue::Pointer(*p)),
            (FFIValue::Void, FFIType::Void) => Ok(FFIValue::Void),
            _ => Err(CrabbyError::CompileError(
                format!("Type mismatch in FFI conversion")
            ))
        }
    }
}

pub fn register_ffi_builtins(compiler: &mut crate::compile::Compiler) {
    compiler.add_builtin("load_library", |args| {
        if args.len() != 1 {
            return Err(CrabbyError::CompileError("load_library expects a library path".into()));
        }
        let lib_path = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(CrabbyError::CompileError("First argument must be library path".into())),
        };
        compiler.ffi_manager.load_library(&lib_path)?;
        Ok(Value::Void)
    });

    compiler.add_builtin("extern_function", |args| {
        if args.len() != 3 {
            return Err(CrabbyError::CompileError("extern_function expects library path, function name, and type signature".into()));
        }

        let lib_path = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(CrabbyError::CompileError("First argument must be library path".into())),
        };

        let func_name = match &args[1] {
            Value::String(s) => s.clone(),
            _ => return Err(CrabbyError::CompileError("Second argument must be function name".into())),
        };

        let type_sig = match &args[2] {
            Value::Array(types) => {
                let mut arg_types = Vec::new();
                let mut return_type = FFIType::Void;

                for (i, t) in types.iter().enumerate() {
                    if let Value::String(type_str) = t {
                        if i == types.len() - 1 {
                            return_type = parse_ffi_type(type_str)?;
                        } else {
                            arg_types.push(parse_ffi_type(type_str)?);
                        }
                    } else {
                        return Err(CrabbyError::CompileError("Type signature must be strings".into()));
                    }
                }
                (arg_types, return_type)
            }
            _ => return Err(CrabbyError::CompileError("Third argument must be array of type signatures".into())),
        };

        compiler.ffi_manager.register_function(&lib_path, &func_name, type_sig.0, type_sig.1)?;
        Ok(Value::Void)
    });
}
