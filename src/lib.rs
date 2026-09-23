use pyo3::prelude::*;

fn count_primes_impl(limit: u64) -> u64 {
    let mut count = 0;

    for number in 2..=limit {
        let mut is_prime = true;
        let mut divisor = 2;

        while divisor * divisor <= number {
            if number % divisor == 0 {
                is_prime = false;
                break;
            }
            divisor += 1;
        }

        if is_prime {
            count += 1;
        }
    }

    count
}

#[pyfunction]
fn count_primes(limit: u64) -> u64 {
    count_primes_impl(limit)
}

#[pyfunction]
fn apply_twice(function: Py<PyAny>, value: i64) -> PyResult<i64> {
    Python::attach(|py| {
        let function = function.bind(py);
        let first: i64 = function.call1((value,))?.extract()?;
        function.call1((first,))?.extract()
    })
}

#[pyfunction]
fn call_upper(text: Bound<'_, PyAny>) -> PyResult<String> {
    let result = text.call_method0("upper")?;
    result.extract()
}

#[pyfunction]
fn count_primes_without_gil(py: Python<'_>, limit: u64) -> u64 {
    py.detach(|| count_primes_impl(limit))
}

#[pymodule]
fn rust_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(count_primes, m)?)?;
    m.add_function(wrap_pyfunction!(apply_twice, m)?)?;
    m.add_function(wrap_pyfunction!(call_upper, m)?)?;
    m.add_function(wrap_pyfunction!(count_primes_without_gil, m)?)?;
    Ok(())
}
