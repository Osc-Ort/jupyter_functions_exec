use pyo3::prelude::*;

use pyo3::exceptions::PyRuntimeError;
use pyo3::types::{PyDict, PyModule, PyTuple};
use regex::Regex;
use std::ffi::CString;
use std::{collections::HashSet, fs};

#[pyclass]
pub struct JupyterFunctions {
    pub functions: Vec<(String, String)>,
    pub imports: HashSet<String>,
}

#[pymethods]
impl JupyterFunctions {
    #[new]
    #[pyo3(signature = (notebook_path))]
    pub fn new(notebook_path: String) -> Self {
        let mut functions: Vec<(String, String)> = Vec::new();
        let mut imports: HashSet<String> = HashSet::new();
        let raw = fs::read_to_string(notebook_path.clone())
            .expect(format!("Error opening the notebook {}", notebook_path).as_str());
        let archive: Vec<String> = raw.lines().map(String::from).collect();
        let n = archive.len();
        let mut i = 0;
        while i < n {
            if archive[i].contains("\"cell_type\": \"code\"") {
                // busca la línea inicial (índice absoluto)
                let indice = archive.iter().enumerate().skip(i).find_map(|(idx, s)| {
                    if s.contains("\"source\": [") {
                        Some(idx)
                    } else {
                        None
                    }
                });
                if let Some(ini) = indice {
                    // position devuelve una posición relativa desde ini
                    if let Some(pos) = archive.iter().skip(ini).position(|s| {
                        let first_non_whitespace = s.find(|c| c != ' ' && c != '\t');
                        first_non_whitespace.map_or(false, |pos| s[pos..].starts_with(']'))
                    }) {
                        // pos es relativo; tomar pos+1 líneas (incluimos la línea que contiene ']')
                        let slice_lines: Vec<String> =
                            archive.iter().skip(ini).take(pos + 1).cloned().collect();
                        process_code(&mut functions, &mut imports, slice_lines);
                        // avanzar i al índice absoluto del final
                        i = ini + pos;
                    }
                }
            }
            i += 1;
        }
        Self { functions, imports }
    }

    #[pyo3(signature = (name, /, *args, **kwargs))]
    pub fn exec_function<'py>(
        &self,
        py: Python<'py>,
        name: &str,
        args: &Bound<'py, PyTuple>,
        kwargs: Option<&Bound<'py, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        if !self.functions.iter().any(|(n, _)| n == name) {
            return Err(PyRuntimeError::new_err(format!(
                "{} doesn't exist in the notebook.",
                name
            )));
        }

        let main = PyModule::import(py, "__main__")?;
        let globals = main.dict();

        let mut code = imports_as_lines(self);
        for (_, func_code) in &self.functions {
            code += func_code;
        }

        let c_code = CString::new(code)
            .map_err(|_| PyRuntimeError::new_err("Código Python contiene byte nulo (\\0)"))?;
        py.run(&c_code, Some(&globals), None)?;

        // Obtiene la función y la ejecuta con *args y **kwargs
        let func = globals.get_item(name)?.ok_or_else(|| {
            PyRuntimeError::new_err(format!("{} wasn't defined after executing code.", name))
        })?;

        let result = func.call(args, kwargs)?;
        Ok(result.unbind())
    }

    #[pyo3(signature = (name))]
    pub fn return_function<'py>(&self, py: Python<'py>, name: &str) -> PyResult<Py<PyAny>> {
        if !self.functions.iter().any(|(n, _)| n == name) {
            return Err(PyRuntimeError::new_err(format!(
                "{} doesn't exist in the notebook.",
                name
            )));
        }

        let main = PyModule::import(py, "__main__")?;
        let globals = main.dict();

        let mut code = imports_as_lines(self);
        for (_, func_code) in &self.functions {
            code += func_code;
        }
        let c_code = CString::new(code)
            .map_err(|_| PyRuntimeError::new_err("Código Python contiene byte nulo (\\0)"))?;
        py.run(&c_code, Some(&globals), None)?;

        // Obtiene la función y la devuelve sin invocarla
        let func = globals.get_item(name)?.ok_or_else(|| {
            PyRuntimeError::new_err(format!("{} wasn't defined after executing code.", name))
        })?;

        Ok(func.unbind())
    }

    pub fn exists_function(&self, name_of_function: String) -> bool {
        self.functions.iter().any(|(n, _)| n == &name_of_function)
    }

    pub fn functions_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.functions.iter().map(|(n, _)| n.clone()).collect();
        names.sort();
        names.dedup();
        names
    }

    pub fn necessary_imports(&self) -> Vec<String> {
        self.imports.iter().cloned().collect()
    }

    /// Devuelve el código extraído de una función para debug
    pub fn get_function_code(&self, name: String) -> Option<String> {
        self.functions
            .iter()
            .rev()
            .find(|(n, _)| n == &name)
            .map(|(_, c)| c.clone())
    }

    /// Devuelve el código completo que se ejecutaría (imports + función)
    pub fn get_full_code(&self, name: String) -> Option<String> {
        self.functions
            .iter()
            .rev()
            .find(|(n, _)| n == &name)
            .map(|(_, _)| {
                let mut code = imports_as_lines(self);
                for (_, f_code) in &self.functions {
                    code += f_code;
                }
                code
            })
    }
}

pub fn process_code(
    functions: &mut Vec<(String, String)>,
    imports: &mut HashSet<String>,
    raw_lines: Vec<String>,
) {
    let code_lines: Vec<String> = raw_lines.into_iter().map(|e| clean_line_json(e)).collect();
    // Import form - solo imports sin indentación (nivel raíz)
    let import_regex =
        Regex::new(r"^(import|from)\s+").expect("Error making the regex processing the code.");
    let conj_import: HashSet<String> = code_lines
        .iter()
        .filter(|&e| import_regex.is_match(e))
        .map(|e| e.clone() + "\n")
        .collect();
    imports.extend(conj_import);
    // Functions form
    let func_regex =
        Regex::new(r"^def\s+(\w+)\s*\(").expect("Error making the regex processing the code.");
    let mut i = 0;
    while i < code_lines.len() {
        let line = code_lines[i].clone();
        if let Some(mach) = func_regex.captures(line.as_str()) {
            // captura ahora el nombre correcto en el grupo 1
            let func_name = mach[1].to_string();
            let mut func_body = line.clone() + "\n";
            let mut j = i + 1;
            'inner: while j < code_lines.len() {
                let next_line = code_lines[j].clone();
                // Empty line
                if next_line.is_empty() {
                    func_body.push('\n');
                    j += 1; // mover índice para evitar bucle infinito
                    continue;
                }
                // We look if is tabulated (considera indentación o comentarios)
                let first_char = next_line.chars().next().unwrap();
                if matches!(first_char, '\t' | ' ' | '#') {
                    func_body.push_str(next_line.as_str());
                    func_body.push('\n');
                    j += 1;
                } else {
                    break 'inner;
                }
            }

            functions.push((func_name, func_body));
            if j > i {
                i = j - 1;
            }
        }
        i += 1;
    }
}

// Function to clean all the JSON quoting of the notebook.
pub fn clean_line_json(line: String) -> String {
    let first_non_whitespace = line.find(|c| c != ' ' && c != '\t');
    if let Some(ind) = first_non_whitespace {
        if line[ind..].starts_with('"') {
            let start_quote = ind;
            // buscar índice del último '"' en la línea
            return if let Some(end_quote) = line.rfind('"') {
                // asegurarnos que el end_quote esté después del start_quote
                if start_quote + 1 >= end_quote {
                    return String::new();
                }
                let slice = &line[start_quote + 1..end_quote];
                // ahora procesar escapes
                let line_chars: Vec<char> = slice.chars().collect();
                let mut content = String::with_capacity(line_chars.len());
                let mut i = 0;
                while i < line_chars.len() {
                    if line_chars[i] == '\\' {
                        if i < line_chars.len() - 1 {
                            let next = line_chars[i + 1];
                            match next {
                                '"' => {
                                    content.push('"');
                                    i += 1;
                                }
                                '\\' => {
                                    content.push('\\');
                                    i += 1;
                                }
                                'n' => {
                                    content.push('\n');
                                    i += 1;
                                }
                                't' => {
                                    content.push('\t');
                                    i += 1;
                                }
                                'r' => {
                                    content.push('\r');
                                    i += 1;
                                }
                                'b' => {
                                    content.push('\x08'); // backspace
                                    i += 1;
                                }
                                'f' => {
                                    content.push('\x0C'); // form feed
                                    i += 1;
                                }
                                '/' => {
                                    content.push('/');
                                    i += 1;
                                }
                                '\'' => {
                                    content.push('\'');
                                    i += 1;
                                }
                                'u' => {
                                    // Secuencia Unicode \uXXXX
                                    if i + 5 < line_chars.len() {
                                        let hex: String = line_chars[i + 2..i + 6].iter().collect();
                                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                                            if let Some(c) = char::from_u32(code) {
                                                content.push(c);
                                                i += 5; // saltar \uXXXX
                                            } else {
                                                // Código inválido, mantener literal
                                                content.push('\\');
                                                content.push('u');
                                                i += 1;
                                            }
                                        } else {
                                            // No es hex válido, mantener literal
                                            content.push('\\');
                                            content.push('u');
                                            i += 1;
                                        }
                                    } else {
                                        // No hay suficientes caracteres, mantener literal
                                        content.push('\\');
                                        content.push('u');
                                        i += 1;
                                    }
                                }
                                _ => {
                                    // Escape no reconocido, mantener ambos caracteres
                                    content.push('\\');
                                    content.push(next);
                                    i += 1;
                                }
                            }
                        } else {
                            content.push('\\');
                        }
                    } else {
                        content.push(line_chars[i]);
                    }
                    i += 1;
                }

                if content.chars().last().unwrap_or('\n') == '\n' {
                    content.pop();
                }

                content
            } else {
                String::new()
            };
        }
    }
    String::new()
}

pub fn imports_as_lines(notebook: &JupyterFunctions) -> String {
    notebook.imports.iter().map(|e| e.clone() + "\n").collect()
}

#[pymodule]
fn jupyter_functions_exec(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<JupyterFunctions>()?;
    Ok(())
}

#[cfg(test)]
mod tests;
