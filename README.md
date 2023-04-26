# rxml

## What is rxml?

`rxml` is a simple python library to read xml files up to 2 times faster than python's `xml(ElementTree)` library.

## Installation

To install `rxml` you can use `pip`:

```bash
pip install rxml
```

Simply as that!

## Example usage

To a given xml with `test.xml` as name:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<note example_attr="example value">
    <to>
        <name>Example Name</name>
    </to>
    <from>
        <name>Example Name</name>
    </from>
    <heading>An Example Heading</heading>
    <body>An Example Body!</body>
</note>
```

We write the following python code:

```python
from rxml import read_file

root_file = read_file("test.xml", "note")
```

where `"test.xml"` is the `file_name` and `"note"` is the `root_tag`.

After that we can simply iter through the children with:

```python
for node in root_file.children:
    # do something whith the node here
```

## Node attributes

This is how the `Node` looks like:

```python
class Node:
    name: str
    attrs: dict[str, str]
    children: list[Node]
    text: str
```
