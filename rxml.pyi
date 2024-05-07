from enum import Enum

class SearchType(Enum):
    tag: str
    attr: str
    text: str

class Node:
    name: str
    attrs: dict[str, str]
    children: list[Node]
    text: str

    def __new__(
        cls,
        name: str,
        attrs: dict[str, str] = dict(),
        children: list[Node] = list(),
        text: str | None = None,
    ) -> Node: ...
    def __to_string(self, spacing: int | None) -> str: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def search(self, by: str, value: str) -> list[Node]: ...

def read_file(file_path: str, root_tag: str) -> Node: ...
def read_string(xml_string: str, root_tag: str) -> Node: ...
def write_file(
    node: Node, file_path: str, indent: int = 4, default_xml_def: bool = True
): ...
def write_string(node: Node, indent: int = 4, default_xml_def: bool = True) -> str: ...
