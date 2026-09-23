# PyO3Demo：Python 与 Rust 的 PyO3 实验教程

本项目演示如何使用 **PyO3** 将 Rust 编译成 Python 扩展模块，并使用 **maturin** 完成开发、构建、安装和发布。

本教程包含：

- Python 调用 Rust 函数
- Rust 函数接收 Python 的整数、字符串和列表
- Rust 调用 Python 函数和对象方法
- `Cargo.toml`、`pyproject.toml` 和模块命名的关系
- 使用 maturin 的开发模式和发布模式
- Windows PowerShell、虚拟环境和 Conda 冲突的处理
- Python 与 Rust 的性能对比

## 1. 最终结构

```text
PyO3Demo/
├── Cargo.toml
├── Cargo.lock
├── pyproject.toml
├── README.md
├── .venv/
├── .vscode/
│   └── run-python.ps1
└── src/
    ├── lib.rs
    └── scripts/
        └── main.py
```

其中：

- `src/lib.rs` 是 Rust/PyO3 扩展模块的入口。
- `src/scripts/main.py` 是 Python 运行和基准测试入口。
- `pyproject.toml` 告诉 Python 构建系统使用 maturin。
- `Cargo.toml` 管理 Rust 包和 PyO3 依赖。

## 2. 环境准备

需要安装：

- Python 3.9 或更高版本
- Rust 工具链，包括 `cargo` 和 `rustc`
- maturin

在 Windows PowerShell 中创建虚拟环境：

```powershell
py -m venv .venv
.\.venv\Scripts\Activate.ps1
python -m pip install --upgrade pip
python -m pip install "maturin>=1.15,<2"
```

检查工具：

```powershell
python --version
rustc --version
cargo --version
python -m maturin --version
```

如果 PowerShell 禁止执行激活脚本，可以只对当前进程放开权限：

```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy RemoteSigned
.\.venv\Scripts\Activate.ps1
```

## 3. Cargo 配置

当前项目的 `Cargo.toml`：

```toml
[package]
name = "pyo3-demo"
version = "0.1.0"
edition = "2024"

[lib]
name = "rust_core"
crate-type = ["cdylib"]

[dependencies.pyo3]
version = "0.29.2"
features = ["extension-module"]
```

关键配置如下：

### 3.1 `crate-type = ["cdylib"]`

普通 Rust 二进制或库不能直接作为 Python 模块导入。`cdylib` 会生成适合外部语言加载的动态库，Windows 下最终会以 `.pyd` 形式被 Python 加载。

### 3.2 `extension-module`

这个 PyO3 feature 用于构建 Python 扩展模块。它会让 PyO3 按 Python 扩展的方式处理链接和初始化。

### 3.3 包名和模块名

本项目有两个名字：

- Rust package 名：`pyo3-demo`
- Python 扩展模块名：`rust_core`

Python 中真正导入的是：

```python
import rust_core
```

## 4. maturin 配置

项目使用 `pyproject.toml`：

```toml
[build-system]
requires = ["maturin>=1.15,<2"]
build-backend = "maturin"

[project]
name = "PyO3Demo"
version = "0.1.0"
requires-python = ">=3.9"

[tool.maturin]
bindings = "pyo3"
module-name = "rust_core"
```

这表示：

- Python 构建后端是 maturin。
- 构建依赖会自动包含 maturin。
- 本项目使用 PyO3 绑定。
- 生成的 Python 模块名是 `rust_core`。

`module-name` 必须和 Rust 中的 `#[pymodule]` 函数名以及 Python 的 `import` 名称保持一致。

## 5. Python 调用 Rust

### 5.1 Rust 导出函数

当前 `src/lib.rs` 的完整核心代码：

```rust
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
```

这里有三个重要部分：

1. `#[pyfunction]` 把 Rust 函数转换成 Python 可以调用的函数。
2. `#[pymodule]` 声明 Python 扩展模块的初始化函数。
3. `wrap_pyfunction!` 把 `count_primes` 注册到模块中。

### 5.2 Python 调用

```python
import rust_core

result = rust_core.count_primes(100)
print(result)
```

Rust 的 `u64` 会自动转换成 Python 的整数。PyO3 会检查 Python 参数是否可以转换成 Rust 要求的类型。

### 5.3 运行当前实验

直接使用 maturin 的开发模式：

```powershell
python -m maturin develop
python .\src\scripts\main.py
```

`maturin develop` 会编译 Rust 扩展，并以 editable 方式安装到当前 Python 环境。开发时不需要自己寻找 wheel，也不需要手动执行 `pip install`。

需要发布构建或测试优化版本时：

```powershell
python -m maturin develop --release
python .\src\scripts\main.py
```

## 6. Rust 接收 Python 类型

下面是扩展模块时可以添加的类型转换示例。它们目前没有注册到本项目的
`rust_core` 模块中；如果要实际使用，需要像第 8 节那样在 `#[pymodule]`
中注册函数。

PyO3 支持许多常见的自动转换：

```rust
use pyo3::prelude::*;

#[pyfunction]
fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[pyfunction]
fn sum_numbers(values: Vec<i64>) -> i64 {
    values.into_iter().sum()
}

#[pyfunction]
fn describe(name: String, values: Vec<i64>) -> String {
    format!("{name}: {} numbers", values.len())
}
```

对应的 Python 调用：

```python
import rust_core

print(rust_core.greet("Rust"))
print(rust_core.sum_numbers([1, 2, 3, 4]))
print(rust_core.describe("sample", [10, 20]))
```

常见对应关系：

| Rust 类型 | Python 类型 |
| --- | --- |
| `bool` | `bool` |
| `i64`, `u64` | `int` |
| `f64` | `float` |
| `String` | `str` |
| `Vec<T>` | `list` |
| `Option<T>` | `T` 或 `None` |
| `HashMap<K, V>` | `dict` |

当函数需要返回 Python 异常时，可以返回 `PyResult<T>`：

```rust
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
fn positive_square(value: i64) -> PyResult<i64> {
    if value < 0 {
        return Err(PyValueError::new_err("value must be non-negative"));
    }

    Ok(value * value)
}
```

Python 中会得到普通的 Python 异常：

```python
import rust_core

rust_core.positive_square(-1)
# ValueError: value must be non-negative
```

## 7. Rust 调用 Python

PyO3 不只是让 Python 调 Rust，也允许 Rust 保存或调用 Python 对象。

下面的函数接收一个 Python callable，然后让 Rust 调用它：

```rust
use pyo3::prelude::*;

#[pyfunction]
fn apply_twice(function: Py<PyAny>, value: i64) -> PyResult<i64> {
    Python::attach(|py| {
        let function = function.bind(py);
        let first: i64 = function.call1((value,))?.extract()?;
        let second: i64 = function.call1((first,))?.extract()?;
        Ok(second)
    })
}
```

Python 调用：

```python
import rust_core

result = rust_core.apply_twice(lambda value: value + 3, 10)
print(result)  # 16
```

执行过程是：

1. Python 把 lambda 作为对象传给 Rust。
2. Rust 使用 `Python::attach` 获取 Python 解释器上下文。
3. `call1` 调用 Python 函数。
4. `extract` 把 Python 返回值转换为 Rust 的 `i64`。

### 7.1 调用 Python 对象方法

```rust
use pyo3::prelude::*;

#[pyfunction]
fn call_upper(text: Bound<'_, PyAny>) -> PyResult<String> {
    let result = text.call_method0("upper")?;
    result.extract()
}
```

Python：

```python
import rust_core

print(rust_core.call_upper("hello"))
# HELLO
```

### 7.2 关于 GIL

Python 对象的访问必须在 Python 解释器上下文中完成。PyO3 通过 `Python::attach` 帮助 Rust 获取 Python 上下文。

如果 Rust 正在执行很长时间、且这段代码不访问 Python 对象，可以考虑释放 GIL，让其他 Python 线程继续执行。当前 `count_primes` 只使用 Rust 数据，可以改成：

```rust
use pyo3::prelude::*;

#[pyfunction]
fn count_primes_without_gil(py: Python<'_>, limit: u64) -> u64 {
    py.detach(|| count_primes_impl(limit))
}
```

本项目已经把算法拆成普通 Rust 函数，实际实现如下：

```rust
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
```

只有在确实需要并发或较长计算时才释放 GIL。若 Rust 代码需要频繁调用 Python，就不能在调用期间释放 GIL。

## 8. 模块注册多个函数

完整的模块注册示例：

```rust
use pyo3::prelude::*;

#[pyfunction]
fn add(left: i64, right: i64) -> i64 {
    left + right
}

#[pyfunction]
fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[pymodule]
fn rust_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(greet, m)?)?;
    Ok(())
}
```

如果函数数量继续增加，可以把 Rust 实现拆成多个模块，再在 `lib.rs` 的 `#[pymodule]` 中统一注册。

## 9. 当前基准测试

`src/scripts/main.py` 会先验证 Rust 调用 Python，再运行 Python/Rust 基准：

```python
if rust_core.apply_twice(lambda value: value + 3, 10) != 16:
    raise RuntimeError("Rust callback invocation returned an unexpected result")
if rust_core.call_upper("hello") != "HELLO":
    raise RuntimeError("Rust Python method invocation returned an unexpected result")

python_result, python_time = benchmark("Python", count_primes, limit)
rust_result, rust_time = benchmark("Rust  ", rust_core.count_primes, limit)
rust_detached_result = rust_core.count_primes_without_gil(limit)

if python_result != rust_result or rust_result != rust_detached_result:
    raise RuntimeError("Python and Rust returned different results")

print(f"Rust speedup: {python_time / rust_time:.2f}x")
```

这里先比较结果，再比较时间。这样可以避免因为 Rust 版本算错而得到没有意义的性能结论。

运行：

```powershell
python -m maturin develop --release
python .\src\scripts\main.py
```

基准结果会受到 CPU、Python 版本、编译模式和系统负载影响。开发模式使用 debug 编译，性能通常低于 `--release`。

## 10. Windows 下使用项目脚本

本项目提供 `.vscode/run-python.ps1`，用于：

1. 定位项目的 `.venv` Python。
2. 调用 `python -m maturin develop`。
3. 运行传入的 Python 文件。

使用方式：

```powershell
powershell -ExecutionPolicy Bypass -File .\.vscode\run-python.ps1 .\src\scripts\main.py
```

脚本会临时清除 `CONDA_PREFIX`。这是因为某些终端会同时设置：

- `VIRTUAL_ENV`：项目 `.venv`
- `CONDA_PREFIX`：Miniconda 或 Anaconda 环境

maturin 检测到两个环境同时存在时会拒绝构建。脚本只在子进程执行期间清除 `CONDA_PREFIX`，结束后恢复原值，不会永久修改当前终端环境。

更简单的做法是只激活一个环境：

```powershell
conda deactivate
.\.venv\Scripts\Activate.ps1
python -m maturin develop
```

## 11. maturin 命令说明

### 开发模式

```powershell
python -m maturin develop
```

编译并以 editable 方式安装到当前环境，适合频繁修改代码。

### Release 开发模式

```powershell
python -m maturin develop --release
```

适合运行性能测试。

### 构建 wheel

```powershell
python -m maturin build --release
```

wheel 通常会生成在 `target/wheels/` 目录中。

### 安装 wheel

```powershell
python -m pip install .\target\wheels\*.whl --force-reinstall
```

### 发布前检查

```powershell
cargo check
cargo fmt --check
python -m maturin build --release
```

## 12. 常见问题

### `No module named rust_core`

通常是扩展还没有安装到当前 Python 环境。执行：

```powershell
python -m maturin develop
```

并确认运行脚本使用的是同一个 Python：

```powershell
python -c "import sys; print(sys.executable)"
```

### `Both VIRTUAL_ENV and CONDA_PREFIX are set`

说明 venv 和 Conda 环境变量同时存在。可以：

```powershell
conda deactivate
```

或者直接使用项目提供的 `run-python.ps1`，它会临时处理该冲突。

### 修改 Rust 后 Python 仍使用旧代码

开发模式下重新执行：

```powershell
python -m maturin develop
```

如果仍然异常，可以清理后重新构建：

```powershell
cargo clean
python -m maturin develop
```

### `maturin` 命令找不到

优先使用当前 Python 调用：

```powershell
python -m maturin --version
```

如果失败，安装到当前虚拟环境：

```powershell
python -m pip install "maturin>=1.15,<2"
```

## 13. 推荐的开发循环

日常修改 Rust 代码后：

```powershell
python -m maturin develop
python .\src\scripts\main.py
```

测试性能时：

```powershell
python -m maturin develop --release
python .\src\scripts\main.py
```

准备发布时：

```powershell
cargo fmt
cargo check
python -m maturin build --release
```

核心思路是：**PyO3 负责 Rust 与 Python 的类型和调用转换，Cargo 负责 Rust 编译，maturin 负责把 Rust 扩展接入 Python 的构建和安装流程。**