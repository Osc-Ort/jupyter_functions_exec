# Jupyter Functions Exec

[![PyPI version](https://badge.fury.io/py/jupyter-functions-exec.svg)](https://badge.fury.io/py/jupyter-functions-exec)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

A high-performance Python package built in Rust that allows you to extract and execute functions directly from Jupyter Notebooks (`.ipynb` files).

## Installation

```bash
pip install jupyter-functions-exec
```

## API Reference

### `JupyterFunctions(notebook_path: str)`

Creates a new instance by parsing a Jupyter notebook.

---

### `exec_function(name: str, /, *args, **kwargs) -> Any`

Executes a function from the notebook with the provided arguments and returns the result.

---

### `return_function(name: str) -> Callable`

Returns a function object that can be called later.

---

### `exists_function(name: str) -> bool`

Checks if a function exists in the notebook.

---

### `functions_names() -> List[str]`

Returns a list of all function names found in the notebook.

---

### `necessary_imports() -> List[str]`

Returns all import statements found in the notebook.

## License

MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Oscar - oscarortegarios@gmail.com