# Jupyter Functions Exec

Simple Python package build in Rust to use functions inside a Jupyter Notebook.

It gives the JupyterFunctions class.

- Constructor: Simple contructor with the path of the file.
- exec_function: Execute a function with its arguments.
- return_function: Return the funtion as an object.
- exists_function: Returns a boolean of if a function exists.
- functions_names: Returns a list of all the names of functions.
- necessary_imports: Returns all the necesary imports.

For each function (for now) we need to import all the imports of the notebook.

To install
```bash
  pip install jupyter-functions-exec
```
