//! Tests de Caja Negra (Black Box)
//!
//! **DISCLAIMER**: Este módulo de tests ha sido creado con la ayuda de
//! Claude Opus 4.5 (Anthropic) como asistente de programación.
//!
//! Estos tests prueban la interfaz pública de la librería sin conocer
//! la implementación interna. Se enfocan únicamente en:
//! - Entradas válidas e inválidas
//! - Salidas esperadas
//! - Comportamiento observable desde fuera
//!
//! No acceden a campos internos ni funciones privadas.

use super::fixture_path;
use crate::JupyterFunctions;

// ============================================================================
// Tests de creación de JupyterFunctions
// ============================================================================

#[test]
fn test_crear_jupyter_functions_notebook_valido() {
    // Dado: un path a un notebook válido
    let path = fixture_path("test_notebook.ipynb");

    // Cuando: creamos una instancia de JupyterFunctions
    let jf = JupyterFunctions::new(path);

    // Entonces: debe crearse sin errores y tener funciones
    assert!(!jf.functions.is_empty(), "Debe tener funciones extraídas");
}

#[test]
#[should_panic(expected = "Error opening the notebook")]
fn test_crear_jupyter_functions_archivo_inexistente() {
    // Dado: un path a un archivo que no existe
    let path = String::from("/ruta/inexistente/notebook.ipynb");

    // Cuando: intentamos crear una instancia
    // Entonces: debe lanzar un panic
    let _jf = JupyterFunctions::new(path);
}

#[test]
fn test_crear_jupyter_functions_notebook_vacio() {
    // Dado: un notebook vacío
    let path = fixture_path("empty_notebook.ipynb");

    // Cuando: creamos una instancia
    let jf = JupyterFunctions::new(path);

    // Entonces: no debe tener funciones ni imports
    assert!(jf.functions.is_empty(), "No debe tener funciones");
    assert!(jf.imports.is_empty(), "No debe tener imports");
}

// ============================================================================
// Tests de exists_function
// ============================================================================

#[test]
fn test_exists_function_funcion_existente() {
    // Dado: un notebook con funciones
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: buscamos una función que existe
    let existe = jf.exists_function(String::from("suma"));

    // Entonces: debe retornar true
    assert!(existe, "La función 'suma' debe existir");
}

#[test]
fn test_exists_function_funcion_inexistente() {
    // Dado: un notebook con funciones
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: buscamos una función que no existe
    let existe = jf.exists_function(String::from("funcion_inventada"));

    // Entonces: debe retornar false
    assert!(!existe, "La función 'funcion_inventada' no debe existir");
}

#[test]
fn test_exists_function_nombre_vacio() {
    // Dado: un notebook con funciones
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: buscamos con nombre vacío
    let existe = jf.exists_function(String::new());

    // Entonces: debe retornar false
    assert!(!existe, "Una función con nombre vacío no debe existir");
}

#[test]
fn test_exists_function_nombre_con_espacios() {
    // Dado: un notebook con funciones
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: buscamos con un nombre que tiene espacios
    let existe = jf.exists_function(String::from("suma con espacios"));

    // Entonces: debe retornar false (los nombres de funciones no tienen espacios)
    assert!(
        !existe,
        "No debe existir una función con espacios en el nombre"
    );
}

// ============================================================================
// Tests de functions_names
// ============================================================================

#[test]
fn test_functions_names_retorna_nombres() {
    // Dado: un notebook con funciones conocidas
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: obtenemos los nombres de funciones
    let nombres = jf.functions_names();

    // Entonces: debe contener las funciones esperadas
    assert!(
        nombres.contains(&String::from("suma")),
        "Debe contener 'suma'"
    );
    assert!(
        nombres.contains(&String::from("multiplicar")),
        "Debe contener 'multiplicar'"
    );
    assert!(
        nombres.contains(&String::from("factorial")),
        "Debe contener 'factorial'"
    );
}

#[test]
fn test_functions_names_notebook_vacio() {
    // Dado: un notebook vacío
    let path = fixture_path("empty_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: obtenemos los nombres
    let nombres = jf.functions_names();

    // Entonces: debe estar vacío
    assert!(nombres.is_empty(), "No debe haber nombres de funciones");
}

#[test]
fn test_functions_names_cantidad_correcta() {
    // Dado: un notebook con un número conocido de funciones
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: obtenemos los nombres
    let nombres = jf.functions_names();

    // Entonces: debe tener al menos las funciones que sabemos que existen
    assert!(
        nombres.len() >= 4,
        "Debe tener al menos 4 funciones (suma, multiplicar, factorial, saludar)"
    );
}

// ============================================================================
// Tests de necessary_imports
// ============================================================================

#[test]
fn test_necessary_imports_retorna_imports() {
    // Dado: un notebook con imports
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: obtenemos los imports
    let imports = jf.necessary_imports();

    // Entonces: debe contener los imports esperados
    let imports_str: String = imports.join("");
    assert!(
        imports_str.contains("import math"),
        "Debe contener 'import math'"
    );
    assert!(
        imports_str.contains("from collections import Counter"),
        "Debe contener 'from collections import Counter'"
    );
}

#[test]
fn test_necessary_imports_solo_imports() {
    // Dado: un notebook con solo imports (sin funciones)
    let path = fixture_path("only_imports_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos imports y funciones
    let imports = jf.necessary_imports();
    let funciones = jf.functions_names();

    // Entonces: debe tener imports pero no funciones
    assert!(!imports.is_empty(), "Debe tener imports");
    assert!(funciones.is_empty(), "No debe tener funciones");
}

#[test]
fn test_necessary_imports_notebook_vacio() {
    // Dado: un notebook vacío
    let path = fixture_path("empty_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: obtenemos los imports
    let imports = jf.necessary_imports();

    // Entonces: debe estar vacío
    assert!(imports.is_empty(), "No debe haber imports");
}

// ============================================================================
// Tests de integración (comportamiento general)
// ============================================================================

#[test]
fn test_multiples_funciones_en_misma_celda() {
    // Dado: un notebook con múltiples funciones en una celda
    let path = fixture_path("complex_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos las funciones
    let nombres = jf.functions_names();

    // Entonces: debe detectar ambas funciones
    assert!(nombres.contains(&String::from("funcion_con_decorador_simulado")));
    assert!(nombres.contains(&String::from("otra_funcion_en_misma_celda")));
}

#[test]
fn test_funcion_despues_de_markdown() {
    // Dado: un notebook con celda markdown entre celdas de código
    let path = fixture_path("complex_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: buscamos la función después del markdown
    let existe = jf.exists_function(String::from("funcion_despues_markdown"));

    // Entonces: debe encontrarla
    assert!(
        existe,
        "Debe encontrar funciones después de celdas markdown"
    );
}

#[test]
fn test_consistencia_entre_exists_y_names() {
    // Dado: un notebook con funciones
    let path = fixture_path("test_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: obtenemos los nombres y verificamos existencia
    let nombres = jf.functions_names();

    // Entonces: cada nombre reportado debe existir según exists_function
    for nombre in nombres {
        assert!(
            jf.exists_function(nombre.clone()),
            "La función '{}' debe existir según exists_function",
            nombre
        );
    }
}

#[test]
fn test_notebook_complejo_tiene_todas_las_funciones() {
    // Dado: un notebook complejo con varias funciones
    let path = fixture_path("complex_notebook.ipynb");
    let jf = JupyterFunctions::new(path);

    // Cuando: verificamos las funciones esperadas
    let funciones_esperadas = vec![
        "funcion_con_closure",
        "funcion_multilinea",
        "funcion_con_decorador_simulado",
        "otra_funcion_en_misma_celda",
        "funcion_con_string_especial",
        "funcion_vacia",
        "funcion_con_lambda",
        "funcion_con_try_except",
        "funcion_despues_markdown",
    ];

    // Entonces: todas deben existir
    for nombre in funciones_esperadas {
        assert!(
            jf.exists_function(String::from(nombre)),
            "Debe existir la función '{}'",
            nombre
        );
    }
}
