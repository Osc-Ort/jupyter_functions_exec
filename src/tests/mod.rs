//! Módulo principal de tests
//!
//! **DISCLAIMER**: Este módulo de tests ha sido creado con la ayuda de
//! Claude Opus 4.5 (Anthropic) como asistente de programación.
//!
//! Este módulo organiza las pruebas en tres categorías:
//! - **black_box**: Pruebas de caja negra (interfaz pública)
//! - **white_box**: Pruebas de caja blanca (lógica interna)
//! - **gray_box**: Pruebas de caja gris (combinación de ambas)
//! - **common**: Utilidades compartidas entre tests

mod black_box;
mod common;
mod gray_box;
mod white_box;

use std::path::PathBuf;

/// Helper para obtener la ruta de los fixtures de test
pub fn fixture_path(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);
    path.to_string_lossy().to_string()
}
