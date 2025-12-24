//! Módulo común con helpers y utilidades para los tests.
//!
//! **DISCLAIMER**: Este módulo de tests ha sido creado con la ayuda de
//! Claude Opus 4.5 (Anthropic) como asistente de programación.
//!
//! Este módulo contiene funciones auxiliares que son compartidas
//! entre los diferentes tipos de tests (caja negra, blanca y gris).

use std::path::PathBuf;

/// Obtiene la ruta absoluta a un archivo fixture de prueba.
///
/// # Arguments
/// * `name` - Nombre del archivo fixture (ej: "test_notebook.ipynb")
///
/// # Returns
/// La ruta completa al archivo fixture como String.
///
/// # Example
/// ```
/// let path = fixture_path("test_notebook.ipynb");
/// // Retorna algo como: "/home/user/project/tests/fixtures/test_notebook.ipynb"
/// ```
pub fn fixture_path(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_path_genera_ruta_valida() {
        let path = fixture_path("test_notebook.ipynb");
        assert!(path.contains("tests"));
        assert!(path.contains("fixtures"));
        assert!(path.ends_with("test_notebook.ipynb"));
    }
}
