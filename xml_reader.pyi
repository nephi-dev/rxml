class Node:
    name: str
    attrs: dict[str, str]
    children: list[Node]
    text: str

def read_file(file_path: str, root_tag: str) -> Node: ...
