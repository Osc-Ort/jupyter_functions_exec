use pyo3::prelude::*;

use std::{
    fs,
    collections::{
        HashSet,
        HashMap
    }
};
use pyo3::types::{PyDict, PyTuple, PyModule};
use pyo3::exceptions::PyRuntimeError;
use regex::Regex;

#[pyclass]
struct JupyterFunctions {
    functions: HashMap<String,String>,
    imports: HashSet<String>,
}

#[pymethods]
impl JupyterFunctions {
    #[new]
    #[pyo3(signature = (notebook_path))]
    fn new(notebook_path: String) -> Self {
        let mut functions: HashMap<String,String> = HashMap::new();
        let mut imports: HashSet<String> = HashSet::new();
        let raw = fs::read_to_string(notebook_path.clone())
            .expect(format!("Error opening the notebook {}",notebook_path).as_str());
        let archive: Vec<String> = raw.lines().map(String::from).collect();
        let n = archive.len();
        let mut i = 0;
        while i < n {
            if archive[i].contains("\"cell_type\": \"code\""){
                // We search for the occurrence of the code
                let indice = archive.iter().enumerate().skip(i).find_map(|(idx, s)| {
                    if s.contains("\"source\": [") {
                        Some(idx)
                    } else {
                        None
                    }
                });
                if let Some(ini) = indice {
                    let indice_end = archive.iter().skip(ini).position(|s| {
                        let first_non_whitespace = s.find(|c| c != ' ' && c != '\t');
                        first_non_whitespace.map_or(false, |pos| s.chars().nth(pos) == Some(']'))
                    });
                    if let Some(end) = indice_end {
                        process_code(&mut functions, &mut imports, archive.iter().skip(ini).take(end - ini).cloned().collect());
                        i = end;
                    } 
                }
            }
            i += 1;
        }
        Self {functions, imports}
    }

    #[pyo3(signature = (name, /, *args, **kwargs))]
    fn exec_function<'py>(
        &self,
        py: Python<'py>,
        name: &str,
        args: &pyo3::Bound<'py, PyTuple>,
        kwargs: Option<&pyo3::Bound<'py, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        // Verifica existencia
        if !self.functions.contains_key(name) {
            return Err(PyRuntimeError::new_err(format!(
                "{} doesn't exist in the notebook.",
                name
            )));
        }

        // Ejecuta imports y definición de la función en el ámbito global de __main__
        let main = PyModule::import(py, "__main__")?;
        let globals = main.dict();

        let code = imports_as_lines(self) + self.functions.get(name).unwrap();
        let code_cstr = std::ffi::CString::new(code)
            .map_err(|_| PyRuntimeError::new_err("Failed to convert code to C string"))?;
        py.run(code_cstr.as_c_str(), Some(&globals), None)?;

        // Obtiene la función y la ejecuta con *args y **kwargs
        let func = globals.get_item(name)?.ok_or_else(|| {
            PyRuntimeError::new_err(format!(
                "{} wasn't defined after executing code.",
                name
            ))
        })?;

        let result = func.call(args, kwargs)?;
        Ok(result.unbind())
    }

    #[pyo3(signature = (name))]
    fn return_function<'py>(
        &self,
        py: Python<'py>,
        name: &str,
    ) -> PyResult<Py<PyAny>> {
        // Verifica existencia
        if !self.functions.contains_key(name) {
            return Err(PyRuntimeError::new_err(format!(
                "{} doesn't exist in the notebook.",
                name
            )));
        }

        // Ejecuta imports y definición de la función en el ámbito global de __main__
        let main = PyModule::import(py, "__main__")?;
        let globals = main.dict();

        let code = imports_as_lines(self) + self.functions.get(name).unwrap();
        let code_cstr = std::ffi::CString::new(code)
            .map_err(|_| PyRuntimeError::new_err("Failed to convert code to C string"))?;
        py.run(code_cstr.as_c_str(), Some(&globals), None)?;

        // Obtiene la función y la devuelve sin invocarla
        let func = globals.get_item(name)?.ok_or_else(|| {
            PyRuntimeError::new_err(format!(
                "{} wasn't defined after executing code.",
                name
            ))
        })?;

        Ok(func.unbind())
    }

    fn exists_function(&self, name_of_function: String) -> bool {
        self.functions.contains_key(&name_of_function)
    }

    fn functions_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    fn necessary_imports(&self) -> Vec<String> {
        self.imports.iter().cloned().collect()
    }
}

fn process_code(
    functions: &mut HashMap<String,String>,
    imports: &mut HashSet<String>,
    raw_lines: Vec<String>)
{
    let code_lines: Vec<String> = raw_lines.into_iter().map(|e| clean_line_json(e)).collect();
    // Import form
    let import_regex = Regex::new(r"(^\s*(import|from)\s+)")
        .expect("Error making the regex processing the code.");
    let conj_import: HashSet<String> = code_lines
        .iter()
        .filter(|&e| import_regex.is_match(e))
        .map(|e| e.clone() + "\n")
        .collect();
    imports.extend(conj_import);
    // Functions form
    let func_regex = Regex::new(r"(^def\s+(\w+)\s*\()")
        .expect("Error making the regex processing the code.");
    let mut i = 0;
    while i < code_lines.len() {
        let line = code_lines[i].clone();
        if let Some(mach) = func_regex.captures(line.as_str()) {
            let func_name = mach[1].to_string();
            let mut func_body = line + "\n";
            let mut j = i + 1;
            'inner: while j < code_lines.len() {
                let next_line = code_lines[j].clone();
                // Empty line
                if next_line.is_empty() {
                    func_body.push('\n');
                    continue;
                }
                // We look if is tabulated
                let first_char = next_line.chars().next().unwrap();
                if matches!(first_char, '\t' | ' ' | '#') {
                    func_body.push_str(next_line.as_str());
                    func_body.push('\n');
                    j += 1;
                } else {
                    break 'inner;
                }
            }

            functions.insert(func_name,func_body);
            if j > i { i = j - 1; }
        }
        i += 1;
    }
}

// Function to clean al the JSON things of the notebook. Too dense, too boring
fn clean_line_json(line: String) -> String {
    let first_non_whitespace = line.find(|c| c != ' ' && c != '\t');
    if let Some(ind) = first_non_whitespace && line.chars().nth(ind) == Some('"') {
        let start_quote = ind;
        let end_quote = line.chars().rev().position(|c| c == '"').unwrap_or(line.len());
        // We only get the content between ""
        if start_quote >= end_quote { return String::new(); }
        let line: Vec<char> = line[start_quote+1..end_quote].chars().collect();
        let mut content = String::with_capacity(line.len());
        let mut i = 0;
        while i < line.len() {
            if line[i] == '\\' {
                if i < line.len() - 1 {
                    let next = line[i+1];
                    if next == '"' {content.push('"'); i += 1;}
                    else if next == '\\' {content.push('\\'); i += 1;}
                    else if next == 'n' {content.push('\n'); i += 1;}
                    else {content.push('\\');}
                } else {
                    content.push('\\');
                }
            } else {
                content.push(line[i]);
            }
            i += 1;
        }

        if content.chars().last().unwrap_or('\n') == '\n' {
            content.pop();
        }

        content
    } else { String::new() }
}

fn imports_as_lines(notebook: &JupyterFunctions) -> String {
    notebook.imports.iter().map(|e| e.clone() + "\n").collect()
}