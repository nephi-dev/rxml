from enum import Enum

DictTypes = str | dict[str, str] | list[DictTypes] | None

class SearchType(Enum):
    Tag: int
    Attr: int
    Text: int

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
    def search(
        self, by: SearchType, value: str, depth: int | None = None
    ) -> list[Node]: ...
    @classmethod
    def from_dict(cls, dict_: dict[str, DictTypes]) -> Node: ...
    def to_dict(self) -> dict[str, DictTypes]: ...

def read_file(file_path: str, root_tag: str) -> Node: ...
def read_string(xml_string: str, root_tag: str) -> Node: ...
def write_file(
    node: Node, file_path: str, indent: int = 4, default_xml_def: bool = True
): ...
def write_string(node: Node, indent: int = 4, default_xml_def: bool = True) -> str: ...
