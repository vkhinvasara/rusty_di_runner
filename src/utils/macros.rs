#[macro_export]
macro_rules! impl_to_dict {
    ($t:ty) => {
        #[pymethods]
        impl $t {
            fn to_dict(&self, py: Python) -> PyResult<Py<PyAny>> {
                Ok(pythonize(py, self)?.unbind())
            }
        }
    };
}
