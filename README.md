# rxml

[![PyPI version](https://img.shields.io/pypi/v/rxml?color=%2334D058&label=pypi%20package)](https://pypi.org/project/rxml)
[![Python versions](https://img.shields.io/pypi/pyversions/rxml.svg?color=%2334D058)](https://pypi.org/project/rxml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A fast, lightweight Python library for reading and writing XML files, powered by Rust.

`rxml` provides up to **2× faster** XML parsing compared to Python's built-in `xml.etree.ElementTree`, with a simple and intuitive API.

---

## Features

- **Fast** — Rust-powered XML parsing, up to 2× faster than the standard library.
- **Simple API** — Read, traverse, and write XML with minimal boilerplate.
- **Type-safe** — Ships with a `.pyi` stub file for full editor autocompletion and type checking.
- **Cross-platform** — Supports CPython and PyPy on Windows, macOS, and Linux.

## Installation

```bash
pip install rxml
```

## Quick Start

### Reading XML

Given an XML file `note.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<note example_attr="example value">
    <to>
        <n>Example Name</n>
    </to>
    <from>
        <n>Example Name</n>
    </from>
    <heading>An Example Heading</heading>
    <body>An Example Body!</body>
</note>
```

Parse it with `rxml`:

```python
from rxml import read_file

root = read_file("note.xml", "note")

for child in root.children:
    print(child.name, child.text)
```

### Writing XML

```python
from rxml import Node, write_file

node = Node(
    name="greeting",
    attrs={"lang": "en"},
    text="Hello, World!",
)
write_file(node, "greeting.xml")
```

### The `Node` Object

Every parsed element is represented as a `Node`:

```python
class Node:
    name: str                # Tag name
    attrs: dict[str, str]    # Element attributes
    children: list[Node]     # Child nodes
    text: str                # Text content
```

Refer to the [`rxml.pyi`](rxml.pyi) stub file for the complete API surface, including `read_string`, `write_string`, and additional utilities.

## Development

`rxml` is built with [PyO3](https://pyo3.rs) and [Maturin](https://www.maturin.rs/).

### Prerequisites

- Python 3.8+
- Rust toolchain (stable)
- [Maturin](https://www.maturin.rs/) (`pip install maturin`)

### Building from Source

```bash
git clone https://github.com/nephi-dev/rxml.git
cd rxml
python -m venv .venv && source .venv/bin/activate
pip install maturin
maturin develop
```

### Running Tests

```bash
cargo test
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/nephi-dev/rxml).

## License

This project is licensed under the [MIT License](LICENSE).

## Support

If you find this project useful, consider supporting the author:

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=flat&logo=buy-me-a-coffee&logoColor=black)](https://buymeacoffee.com/nephilim)
