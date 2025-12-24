//! Tests de Caja Blanca (White Box)
//!
//! **DISCLAIMER**: Este módulo de tests ha sido creado con la ayuda de
//! Claude Opus 4.5 (Anthropic) como asistente de programación.
//!
//! Estos tests prueban la lógica interna de la librería, incluyendo:
//! - Funciones privadas/internas
//! - Caminos de ejecución específicos
//! - Condiciones y bucles
//! - Manejo de casos límite en la implementación
//!
//! Requieren conocimiento detallado de la implementación.

use super::fixture_path;
use crate::{JupyterFunctions, clean_line_json, imports_as_lines, process_code};
use std::collections::{HashMap, HashSet};

// ============================================================================
// Tests de clean_line_json
// ============================================================================

#[test]
fn test_clean_line_json_linea_simple() {
    // Dado: una línea JSON con comillas simples
    let linea = String::from("    \"def suma(a, b):\"");

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe extraer el contenido sin comillas JSON
    assert_eq!(resultado, "def suma(a, b):");
}

#[test]
fn test_clean_line_json_con_escape_comillas() {
    // Dado: una línea con comillas escapadas
    let linea = String::from("    \"texto con \\\"comillas\\\"\"");

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe convertir las comillas escapadas
    assert_eq!(resultado, "texto con \"comillas\"");
}

#[test]
fn test_clean_line_json_con_newline_escape() {
    // Dado: una línea con \n escapado
    let linea = String::from("    \"primera linea\\n\"");

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe convertir \n a newline real (y removerlo si está al final)
    assert_eq!(resultado, "primera linea");
}

#[test]
fn test_clean_line_json_con_backslash_escape() {
    // Dado: una línea con \\ escapado
    let linea = String::from("    \"ruta\\\\archivo\"");

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe convertir \\ a \
    assert_eq!(resultado, "ruta\\archivo");
}

#[test]
fn test_clean_line_json_linea_vacia() {
    // Dado: una línea vacía
    let linea = String::new();

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe retornar vacío
    assert!(resultado.is_empty());
}

#[test]
fn test_clean_line_json_sin_comillas() {
    // Dado: una línea sin comillas JSON (solo espacios)
    let linea = String::from("    contenido sin comillas");

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe retornar vacío (no es formato JSON válido)
    assert!(resultado.is_empty());
}

#[test]
fn test_clean_line_json_solo_comillas() {
    // Dado: una línea con solo comillas (contenido vacío)
    let linea = String::from("    \"\"");

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe retornar vacío
    assert!(resultado.is_empty());
}

#[test]
fn test_clean_line_json_backslash_al_final() {
    // Dado: una línea con backslash al final sin siguiente carácter
    let linea = String::from("    \"texto\\\"");

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe preservar el backslash
    assert_eq!(resultado, "texto\\");
}

#[test]
fn test_clean_line_json_multiples_escapes() {
    // Dado: una línea con múltiples tipos de escapes
    let linea = String::from("    \"linea\\ncon\\\"comillas\\\"y\\\\backslash\"");

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe procesar todos los escapes correctamente
    assert!(resultado.contains('\n'));
    assert!(resultado.contains('"'));
    assert!(resultado.contains('\\'));
}

#[test]
fn test_clean_line_json_solo_espacios() {
    // Dado: una línea con solo espacios
    let linea = String::from("        ");

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe retornar vacío
    assert!(resultado.is_empty());
}

#[test]
fn test_clean_line_json_tabs_y_espacios() {
    // Dado: una línea con tabs y espacios mezclados
    let linea = String::from("\t  \t\"contenido\"");

    // Cuando: limpiamos la línea
    let resultado = clean_line_json(linea);

    // Entonces: debe extraer el contenido correctamente
    assert_eq!(resultado, "contenido");
}

// ============================================================================
// Tests de process_code
// ============================================================================

#[test]
fn test_process_code_extrae_funcion_simple() {
    // Dado: líneas de código con una función simple
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"def mi_funcion():\""),
        String::from("    \"    return 42\""),
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe extraer la función
    assert!(functions.contains_key("mi_funcion"));
    let body = functions.get("mi_funcion").unwrap();
    assert!(body.contains("def mi_funcion():"));
    assert!(body.contains("return 42"));
}

#[test]
fn test_process_code_extrae_imports() {
    // Dado: líneas con imports
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"import os\""),
        String::from("    \"from sys import path\""),
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe extraer los imports
    assert_eq!(imports.len(), 2);
    let imports_vec: Vec<&String> = imports.iter().collect();
    let imports_str: String = imports_vec.iter().map(|s| s.as_str()).collect();
    assert!(imports_str.contains("import os"));
    assert!(imports_str.contains("from sys import path"));
}

#[test]
fn test_process_code_funcion_con_lineas_vacias() {
    // Dado: una función con líneas vacías intermedias
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"def funcion_con_espacios():\""),
        String::from("    \"    x = 1\""),
        String::from("    \"\""), // línea vacía
        String::from("    \"    return x\""),
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe incluir la función completa
    assert!(functions.contains_key("funcion_con_espacios"));
}

#[test]
fn test_process_code_multiples_funciones() {
    // Dado: código con múltiples funciones
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"def primera():\""),
        String::from("    \"    pass\""),
        String::from("    \"\""),
        String::from("    \"def segunda():\""),
        String::from("    \"    pass\""),
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe extraer ambas funciones
    assert!(functions.contains_key("primera"));
    assert!(functions.contains_key("segunda"));
}

#[test]
fn test_process_code_funcion_con_comentarios() {
    // Dado: una función con comentarios
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"def con_comentario():\""),
        String::from("    \"    # Este es un comentario\""),
        String::from("    \"    return True\""),
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe incluir el comentario en el cuerpo
    let body = functions.get("con_comentario").unwrap();
    assert!(body.contains("# Este es un comentario"));
}

#[test]
fn test_process_code_sin_funciones_ni_imports() {
    // Dado: código sin funciones ni imports
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"x = 1\""),
        String::from("    \"y = 2\""),
        String::from("    \"print(x + y)\""),
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: no debe haber funciones ni imports
    assert!(functions.is_empty());
    assert!(imports.is_empty());
}

#[test]
fn test_process_code_funcion_con_parametros() {
    // Dado: una función con parámetros
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"def funcion_params(a, b, c=10):\""),
        String::from("    \"    return a + b + c\""),
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe extraer la función con sus parámetros
    assert!(functions.contains_key("funcion_params"));
    let body = functions.get("funcion_params").unwrap();
    assert!(body.contains("a, b, c=10"));
}

#[test]
fn test_process_code_import_from_con_multiples_items() {
    // Dado: un import con múltiples items
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![String::from(
        "    \"from typing import List, Dict, Optional\"",
    )];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe capturar el import completo
    assert_eq!(imports.len(), 1);
    let import_line = imports.iter().next().unwrap();
    assert!(import_line.contains("from typing import"));
}

#[test]
fn test_process_code_import_as() {
    // Dado: un import con alias
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![String::from("    \"import numpy as np\"")];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe capturar el import con alias
    assert_eq!(imports.len(), 1);
    let import_line = imports.iter().next().unwrap();
    assert!(import_line.contains("import numpy as np"));
}

// ============================================================================
// Tests de imports_as_lines
// ============================================================================

#[test]
fn test_imports_as_lines_genera_string_correcto() {
    // Dado: un JupyterFunctions con imports
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: generamos las líneas de imports
    let resultado = imports_as_lines(&jf);

    // Entonces: debe ser un string con newlines
    assert!(resultado.contains('\n'), "Debe contener saltos de línea");
}

#[test]
fn test_imports_as_lines_sin_imports() {
    // Dado: un notebook sin imports
    let jf = JupyterFunctions {
        functions: HashMap::new(),
        imports: HashSet::new(),
    };

    // Cuando: generamos las líneas de imports
    let resultado = imports_as_lines(&jf);

    // Entonces: debe estar vacío
    assert!(resultado.is_empty());
}

#[test]
fn test_imports_as_lines_cada_import_termina_en_newline() {
    // Dado: un JupyterFunctions con imports
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: generamos las líneas de imports
    let resultado = imports_as_lines(&jf);

    // Entonces: debe terminar con newline (cada import agrega uno)
    if !resultado.is_empty() {
        assert!(
            resultado.ends_with('\n'),
            "Debe terminar con salto de línea"
        );
    }
}

// ============================================================================
// Tests de cobertura de caminos en new()
// ============================================================================

#[test]
fn test_new_procesa_celdas_code() {
    // Dado: un notebook con celdas de código
    let path = fixture_path("test_notebook.ipynb");

    // Cuando: creamos JupyterFunctions
    let jf = JupyterFunctions::new(path);

    // Entonces: debe haber procesado las celdas de código
    assert!(!jf.functions.is_empty() || !jf.imports.is_empty());
}

#[test]
fn test_new_ignora_celdas_markdown() {
    // Dado: un notebook con celdas markdown
    let path = fixture_path("complex_notebook.ipynb");

    // Cuando: creamos JupyterFunctions
    let jf = JupyterFunctions::new(path);

    // Entonces: no debe incluir contenido de markdown como función
    // (verificamos que no hay funciones con nombres típicos de encabezados markdown)
    let nombres = jf.functions_names();
    for nombre in &nombres {
        // Los encabezados markdown empiezan con # pero eso no es un nombre válido
        assert!(
            !nombre.starts_with('#'),
            "No debe interpretar encabezados markdown como funciones"
        );
    }
    // Y debe tener funciones válidas de Python
    assert!(
        nombres.contains(&String::from("funcion_despues_markdown")),
        "Debe tener funciones Python definidas después de celdas markdown"
    );
}

#[test]
fn test_new_maneja_notebook_sin_source() {
    // Dado: un notebook vacío (sin celdas con source)
    let path = fixture_path("empty_notebook.ipynb");

    // Cuando: creamos JupyterFunctions
    let jf = JupyterFunctions::new(path);

    // Entonces: no debe fallar y debe tener estructuras vacías
    assert!(jf.functions.is_empty());
    assert!(jf.imports.is_empty());
}

// ============================================================================
// Tests de regex de imports
// ============================================================================

#[test]
fn test_regex_import_simple() {
    // Dado: diferentes formatos de import
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"import math\""),
        String::from("    \"import os\""),
    ];

    // Cuando: procesamos
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe capturar ambos
    assert_eq!(imports.len(), 2);
}

#[test]
fn test_regex_from_import() {
    // Dado: imports con from
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"from os import path\""),
        String::from("    \"from sys import argv\""),
    ];

    // Cuando: procesamos
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe capturar ambos
    assert_eq!(imports.len(), 2);
}

// ============================================================================
// Tests de regex de funciones
// ============================================================================

#[test]
fn test_regex_funcion_nombre_con_underscores() {
    // Dado: una función con underscores en el nombre
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"def mi_funcion_larga():\""),
        String::from("    \"    pass\""),
    ];

    // Cuando: procesamos
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe capturar el nombre completo
    assert!(functions.contains_key("mi_funcion_larga"));
}

#[test]
fn test_regex_funcion_nombre_con_numeros() {
    // Dado: una función con números en el nombre
    let mut functions = HashMap::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"def funcion123():\""),
        String::from("    \"    pass\""),
    ];

    // Cuando: procesamos
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe capturar el nombre
    assert!(functions.contains_key("funcion123"));
}
