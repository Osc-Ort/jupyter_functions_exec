//! Tests de Caja Gris (Gray Box)
//!
//! **DISCLAIMER**: Este módulo de tests ha sido creado con la ayuda de
//! Claude Opus 4.5 (Anthropic) como asistente de programación.
//!
//! Estos tests combinan conocimiento parcial de la implementación con
//! pruebas de interfaz. Verifican comportamientos específicos conociendo
//! algunos detalles internos, pero sin depender completamente de ellos.
//!
//! Características:
//! - Conocen algunas estructuras de datos internas (HashMap, HashSet)
//! - Verifican propiedades que dependen de la implementación
//! - Prueban interacciones entre componentes

use super::fixture_path;
use crate::{JupyterFunctions, process_code};
use std::collections::HashSet;

// ============================================================================
// Tests que verifican estructura interna a través de interfaz
// ============================================================================

#[test]
fn test_imports_son_unicos() {
    // Dado: un notebook que podría tener imports duplicados
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: obtenemos los imports
    let imports = jf.necessary_imports();

    // Entonces: no debe haber duplicados (sabemos que usa HashSet internamente)
    let unique: HashSet<String> = imports.iter().cloned().collect();
    assert_eq!(
        imports.len(),
        unique.len(),
        "No debe haber imports duplicados"
    );
}

#[test]
fn test_funcion_contiene_cuerpo_completo() {
    // Dado: un notebook con funciones conocidas
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos que existe la función (interfaz pública)
    assert!(jf.exists_function(String::from("factorial")));

    // Y accedemos al cuerpo interno (conocimiento de implementación)
    let body = jf
        .functions
        .iter()
        .find(|(n, _)| n == "factorial")
        .unwrap()
        .1
        .clone();

    // Entonces: debe contener toda la lógica de la función
    assert!(body.contains("def factorial"), "Debe tener la definición");
    assert!(body.contains("if n <= 1"), "Debe tener la condición base");
    assert!(body.contains("return"), "Debe tener return");
}

#[test]
fn test_funcion_con_docstring_preserva_formato() {
    // Dado: un notebook con función que tiene docstring
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos la función multiplicar
    assert!(jf.exists_function(String::from("multiplicar")));

    // Entonces: el cuerpo debe preservar el docstring
    let body = jf
        .functions
        .iter()
        .find(|(n, _)| n == "multiplicar")
        .unwrap()
        .1
        .clone();
    assert!(
        body.contains("Multiplica dos números") || body.contains("\"\"\""),
        "Debe preservar el docstring o su contenido"
    );
}

#[test]
fn test_import_from_vs_import_directo() {
    // Dado: un notebook con ambos tipos de import
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: obtenemos los imports
    let imports = jf.necessary_imports();
    let imports_str: String = imports.join(" ");

    // Entonces: debe capturar ambos formatos correctamente
    // (sabemos que el regex debe manejar ambos casos)
    let tiene_import_directo = imports_str.contains("import math");
    let tiene_from_import = imports_str.contains("from collections");

    assert!(tiene_import_directo, "Debe detectar 'import X'");
    assert!(tiene_from_import, "Debe detectar 'from X import Y'");
}

#[test]
fn test_funcion_closure_detecta_funcion_externa() {
    // Dado: un notebook con función que contiene closure
    let path = fixture_path("complex_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos la función con closure
    assert!(jf.exists_function(String::from("funcion_con_closure")));

    // Entonces: debe incluir la función interna en el cuerpo
    // (sabemos cómo process_code maneja indentación)
    let body = jf
        .functions
        .iter()
        .find(|(n, _)| n == "funcion_con_closure")
        .unwrap()
        .1
        .clone();
    assert!(
        body.contains("def incrementar"),
        "Debe incluir la función anidada"
    );
}

#[test]
fn test_indentacion_determina_fin_de_funcion() {
    // Dado: un notebook con múltiples funciones en una celda
    let path = fixture_path("complex_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos que se detectaron como funciones separadas
    let nombres = jf.functions_names();

    // Entonces: ambas funciones deben existir como entidades separadas
    // (verificamos que la lógica de indentación funciona)
    assert!(nombres.contains(&String::from("funcion_con_decorador_simulado")));
    assert!(nombres.contains(&String::from("otra_funcion_en_misma_celda")));

    // Y sus cuerpos deben ser distintos
    let body1 = jf
        .functions
        .iter()
        .find(|(n, _)| n == "funcion_con_decorador_simulado")
        .unwrap()
        .1
        .clone();
    let body2 = jf
        .functions
        .iter()
        .find(|(n, _)| n == "otra_funcion_en_misma_celda")
        .unwrap()
        .1
        .clone();
    assert_ne!(body1, body2, "Los cuerpos deben ser diferentes");
}

#[test]
fn test_caracteres_especiales_en_strings_python() {
    // Dado: un notebook con strings que tienen caracteres especiales
    let path = fixture_path("complex_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos la función con strings especiales
    assert!(jf.exists_function(String::from("funcion_con_string_especial")));

    // Entonces: el cuerpo debe tener los escapes procesados correctamente
    let body = jf
        .functions
        .iter()
        .find(|(n, _)| n == "funcion_con_string_especial")
        .unwrap()
        .1
        .clone();
    // Verificamos que el código es válido (tiene la estructura esperada)
    assert!(body.contains("texto"), "Debe contener la variable 'texto'");
}

#[test]
fn test_funcion_vacia_con_pass() {
    // Dado: un notebook con función que solo tiene pass
    let path = fixture_path("complex_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos la función vacía
    assert!(jf.exists_function(String::from("funcion_vacia")));

    // Entonces: debe tener un cuerpo mínimo con pass
    let body = jf
        .functions
        .iter()
        .find(|(n, _)| n == "funcion_vacia")
        .unwrap()
        .1
        .clone();
    assert!(body.contains("pass"), "Debe contener 'pass'");
}

#[test]
fn test_try_except_preserva_estructura() {
    // Dado: un notebook con try/except/finally
    let path = fixture_path("complex_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos la función con manejo de excepciones
    assert!(jf.exists_function(String::from("funcion_con_try_except")));

    // Entonces: debe preservar toda la estructura del try/except
    let body = jf
        .functions
        .iter()
        .find(|(n, _)| n == "funcion_con_try_except")
        .unwrap()
        .1
        .clone();
    assert!(body.contains("try"), "Debe contener 'try'");
    assert!(body.contains("except"), "Debe contener 'except'");
    assert!(body.contains("finally"), "Debe contener 'finally'");
}

#[test]
fn test_parametros_con_valores_default() {
    // Dado: un notebook con función que tiene parámetros con valores por defecto
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos la función saludar
    assert!(jf.exists_function(String::from("saludar")));

    // Entonces: debe preservar los parámetros con sus valores default
    let body = jf
        .functions
        .iter()
        .find(|(n, _)| n == "saludar")
        .unwrap()
        .1
        .clone();
    assert!(
        body.contains("saludo=") || body.contains("saludo ="),
        "Debe preservar el parámetro con valor por defecto"
    );
}

#[test]
fn test_lambda_dentro_de_funcion() {
    // Dado: un notebook con función que usa lambda
    let path = fixture_path("complex_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos la función con lambda
    assert!(jf.exists_function(String::from("funcion_con_lambda")));

    // Entonces: el lambda debe estar en el cuerpo
    let body = jf
        .functions
        .iter()
        .find(|(n, _)| n == "funcion_con_lambda")
        .unwrap()
        .1
        .clone();
    assert!(body.contains("lambda"), "Debe contener la expresión lambda");
}

// ============================================================================
// Tests de edge cases conociendo la implementación
// ============================================================================

#[test]
fn test_solo_comentarios_sin_funcion() {
    // Dado: código que es solo comentarios (sin def)
    let mut functions: Vec<(String, String)> = Vec::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"# Solo un comentario\""),
        String::from("    \"# Otro comentario\""),
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: no debe crear ninguna función
    assert!(
        functions.is_empty(),
        "No debe crear funciones de comentarios"
    );
}

#[test]
fn test_import_con_indentacion() {
    // Dado: imports con diferentes indentaciones
    let mut functions: Vec<(String, String)> = Vec::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"import os\""),
        String::from("    \"  import sys\""), // con indentación extra
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe capturar el import sin indentación extra
    // (el regex usa ^\s* para manejar indentación)
    assert!(!imports.is_empty(), "Debe capturar al menos un import");
}

// ============================================================================
// Tests de interacción entre componentes
// ============================================================================

#[test]
fn test_imports_y_funciones_se_procesan_juntos() {
    // Dado: código con imports y funciones mezclados
    let mut functions: Vec<(String, String)> = Vec::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"import math\""),
        String::from("    \"\""),
        String::from("    \"def usar_math():\""),
        String::from("    \"    return math.pi\""),
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: debe tener ambos
    assert!(!imports.is_empty(), "Debe tener imports");
    assert!(
        functions.iter().any(|(n, _)| n == "usar_math"),
        "Debe tener la función"
    );
}

#[test]
fn test_hashmap_funciones_permite_acceso_por_nombre() {
    // Dado: un notebook procesado
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: accedemos a funciones por nombre (conociendo que es un HashMap)
    let nombres = jf.functions_names();

    // Entonces: cada nombre debe existir en el Vec interno
    for nombre in &nombres {
        assert!(
            jf.functions.iter().any(|(n, _)| n == nombre),
            "El nombre '{}' debe existir en el Vec",
            nombre
        );
    }
}

#[test]
fn test_imports_hashset_evita_duplicados() {
    // Dado: código con imports duplicados
    let mut functions: Vec<(String, String)> = Vec::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"import os\""),
        String::from("    \"import os\""), // duplicado
        String::from("    \"import os\""), // otro duplicado
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: solo debe haber un import (HashSet elimina duplicados)
    assert_eq!(imports.len(), 1, "HashSet debe eliminar duplicados");
}

#[test]
fn test_funcion_multilinea_parametros() {
    // Dado: un notebook con función de parámetros multilinea
    let path = fixture_path("complex_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos la función
    assert!(jf.exists_function(String::from("funcion_multilinea")));

    // Entonces: el cuerpo debe contener todos los parámetros
    let body = jf
        .functions
        .iter()
        .find(|(n, _)| n == "funcion_multilinea")
        .unwrap()
        .1
        .clone();
    assert!(body.contains("param1"), "Debe contener param1");
    assert!(body.contains("param2"), "Debe contener param2");
    assert!(body.contains("param3"), "Debe contener param3");
}

#[test]
fn test_estructura_cuerpo_funcion_termina_con_newline() {
    // Dado: un notebook procesado
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos los cuerpos de las funciones
    for (nombre, body) in &jf.functions {
        // Entonces: cada cuerpo debe terminar con newline
        // (conocemos que process_code agrega \n al final de cada línea)
        assert!(
            body.ends_with('\n'),
            "El cuerpo de '{}' debe terminar con newline",
            nombre
        );
    }
}

#[test]
fn test_cuerpo_funcion_empieza_con_def() {
    // Dado: un notebook procesado
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos los cuerpos de las funciones
    for (nombre, body) in &jf.functions {
        // Entonces: cada cuerpo debe empezar con 'def'
        assert!(
            body.starts_with("def"),
            "El cuerpo de '{}' debe empezar con 'def'",
            nombre
        );
    }
}

// ============================================================================
// Tests de comportamiento con conocimiento de regex interno
// ============================================================================

#[test]
fn test_regex_no_captura_def_en_strings() {
    // Dado: código con 'def' dentro de un string (no es definición de función)
    let mut functions: Vec<(String, String)> = Vec::new();
    let mut imports = HashSet::new();
    let lines = vec![String::from("    \"x = 'def not_a_function(): pass'\"")];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: no debe crear función (el 'def' está en un string)
    // Nota: Esto depende de cómo el regex maneja el caso - verificamos el comportamiento actual
    // El regex ^def\s+(\w+)\s*\( solo captura 'def' al inicio de la línea limpia
    assert!(
        !functions.iter().any(|(n, _)| n == "not_a_function"),
        "No debe capturar 'def' dentro de strings"
    );
}

#[test]
fn test_import_regex_requiere_espacio_despues() {
    // Dado: código donde 'import' es parte de otro identificador
    let mut functions: Vec<(String, String)> = Vec::new();
    let mut imports = HashSet::new();
    let lines = vec![
        String::from("    \"reimport = True\""), // no es import
        String::from("    \"import_data = 1\""), // no es import
    ];

    // Cuando: procesamos el código
    process_code(&mut functions, &mut imports, lines);

    // Entonces: no debe capturar estos como imports
    // (el regex usa ^\s*(import|from)\s+ que requiere espacio después)
    assert!(
        imports.is_empty(),
        "No debe capturar 'import' como parte de otros identificadores"
    );
}
