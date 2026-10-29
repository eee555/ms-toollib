"""Generate stubs from this package's PyO3 sources without compiling Rust.

Requires Python 3.10+, tree-sitter>=0.25,<0.27 and tree-sitter-rust>=0.24,<0.25.
Run: python python_package/generate_pyi.py --compare python_package/ms_toollib.pyi
Only single-ident macro_rules templates are expanded; this is not rustc/cfg evaluation.
"""

from __future__ import annotations

import argparse
import ast
import difflib
import re
import sys
from pathlib import Path

from tree_sitter import Language, Node, Parser
import tree_sitter_rust


def text(node: Node) -> str:
    return (node.text or b"").decode("utf-8")


def field(node: Node, name: str) -> Node:
    result = node.child_by_field_name(name)
    if result is None:
        raise ValueError(f"Missing {name} in {text(node)[:120]}")
    return result


def walk(node: Node):
    yield node
    for child in node.named_children:
        yield from walk(child)


def items(node: Node):
    attributes: list[str] = []
    docs: list[str] = []
    for child in node.named_children:
        if child.type == "attribute_item":
            attributes.append(text(child)[2:-1].strip())
        elif child.type == "line_comment":
            if text(child).startswith("///"):
                docs.append(text(child)[3:].removeprefix(" "))
        elif child.type != "block_comment":
            yield child, attributes, "\n".join(docs)
            attributes, docs = [], []


def has(attributes: list[str], name: str) -> bool:
    return any(re.match(rf"{name}(?:\s*\(|$)", value) for value in attributes)


def option(attributes: list[str], name: str) -> str | None:
    for attribute in attributes:
        match = re.search(rf'\b{name}\s*=\s*("(?:\\.|[^"\\])*")', attribute)
        if match:
            return ast.literal_eval(match[1])
    return None


def split_arguments(value: str) -> list[str]:
    parts: list[str] = []
    stack: list[str] = []
    quote = ""
    escaped = False
    start = 0
    for index, character in enumerate(value):
        if quote:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == quote:
                quote = ""
        elif character == '"':
            quote = character
        elif character in "([{":
            stack.append(character)
        elif character in ")]}":
            if not stack:
                raise ValueError(f"Unbalanced signature: {value}")
            stack.pop()
        elif character == "," and not stack:
            parts.append(value[start:index].strip())
            start = index + 1
    if quote or stack:
        raise ValueError(f"Unbalanced signature: {value}")
    if value[start:].strip():
        parts.append(value[start:].strip())
    return parts


def default_value(value: str) -> str:
    value = {"true": "True", "false": "False", "None": "None"}.get(value, value)
    try:
        return repr(ast.literal_eval(value.replace("_", "") if re.fullmatch(r"[\d_]+", value) else value))
    except (SyntaxError, ValueError):
        return "..."


class Generator:
    def __init__(self, source: Path):
        self.parser = Parser(Language(tree_sitter_rust.language()))
        self.units: list[Node] = []
        self.loaded: set[Path] = set()
        self.names: dict[str, str] = {}
        self.classes: dict[str, tuple[Node, list[str]]] = {}
        self.methods: dict[str, list[tuple[Node, list[str], str]]] = {}
        self.functions: dict[str, tuple[Node, list[str], str]] = {}
        self.export_classes: set[str] = set()
        self.export_functions: set[str] = set()
        self.load(source)

    def parse(self, source: str, label: str) -> Node:
        root = self.parser.parse(source.encode("utf-8")).root_node
        if root.has_error:
            errors = [node for node in walk(root) if node.type == "ERROR" or node.is_missing]
            raise ValueError(f"Rust parse error in {label}: {[(node.start_point, text(node)[:80]) for node in errors[:3]]}")
        return root

    def load(self, path: Path) -> None:
        path = path.resolve()
        if path in self.loaded:
            return
        self.loaded.add(path)
        root = self.parse(path.read_text(encoding="utf-8-sig"), str(path))
        self.units.append(root)
        for node, attributes, _ in items(root):
            if has(attributes, "cfg") or has(attributes, "cfg_attr"):
                raise ValueError(f"Conditional item requires manual cfg selection: {path}: {text(node)[:80]}")
            if node.type == "mod_item":
                if node.child_by_field_name("body") is not None:
                    raise ValueError(f"Inline module not supported: {text(node)[:80]}")
                name = text(field(node, "name"))
                custom = option(attributes, "path")
                candidates = [path.parent / custom] if custom else [path.parent / f"{name}.rs", path.parent / name / "mod.rs"]
                module = next((candidate for candidate in candidates if candidate.is_file()), None)
                if module is None:
                    raise ValueError(f"Cannot locate module {name} from {path}")
                self.load(module)
        definitions = {text(field(node, "name")): node for node in root.named_children if node.type == "macro_definition"}
        for node in root.named_children:
            invocation = node.named_children[0] if node.type == "expression_statement" and node.named_children else node
            if invocation.type != "macro_invocation":
                continue
            name = text(field(invocation, "macro"))
            if name not in definitions:
                raise ValueError(f"Unsupported top-level macro: {name}")
            definition = definitions[name]
            rules = [child for child in definition.named_children if child.type == "macro_rule"]
            if len(rules) != 1:
                raise ValueError(f"Only single-rule macros supported: {name}")
            rule = rules[0]
            pattern = text(field(rule, "left"))
            match = re.fullmatch(r"\(\s*\$(\w+)\s*:\s*ident\s*\)", pattern)
            tokens = next(child for child in invocation.named_children if child.type == "token_tree")
            argument = text(tokens)[1:-1].strip()
            if match is None or not re.fullmatch(r"\w+", argument):
                raise ValueError(f"Unsupported macro invocation: {text(invocation)}")
            body = text(field(rule, "right"))[1:-1]
            expanded = re.sub(rf"\${match[1]}\b", argument, body)
            self.units.append(self.parse(expanded, f"{name}!({argument})"))

    def collect(self) -> None:
        for root in self.units:
            for node in walk(root):
                if node.type == "macro_invocation" and text(field(node, "macro")) == "wrap_pyfunction":
                    tokens = next(child for child in node.named_children if child.type == "token_tree")
                    self.export_functions.add(text(tokens)[1:-1].split(",")[0].strip())
                if node.type == "call_expression":
                    match = re.search(r"\.add_class\s*::\s*<\s*(\w+)\s*>", text(field(node, "function")))
                    if match:
                        self.export_classes.add(match[1])
            for node, attributes, docs in items(root):
                if node.type == "struct_item" and has(attributes, "pyclass"):
                    name = text(field(node, "name"))
                    self.classes[name] = (node, attributes)
                    public = option(attributes, "name") or name
                    self.names[name] = public if name in self.export_classes else "_" + public
                elif node.type == "function_item" and has(attributes, "pyfunction"):
                    self.functions[text(field(node, "name"))] = (node, attributes, docs)
                elif node.type == "impl_item" and has(attributes, "pymethods"):
                    name = text(field(node, "type"))
                    methods = [(method, attrs, doc) for method, attrs, doc in items(field(node, "body")) if method.type == "function_item"]
                    self.methods.setdefault(name, []).extend(methods)
        for name in self.classes:
            if name in self.export_classes:
                self.names[name] = option(self.classes[name][1], "name") or name
        missing = (self.export_classes - self.classes.keys()) | (self.export_functions - self.functions.keys())
        if missing:
            raise ValueError(f"Registered bindings not parsed: {sorted(missing)}")

    def python_type(self, node: Node | None, owner: str = "") -> str:
        if node is None:
            return "None"
        value = text(node)
        if node.type == "reference_type":
            return self.python_type(field(node, "type"), owner)
        if node.type == "tuple_type":
            types = [self.python_type(child, owner) for child in node.named_children]
            return "tuple[" + ", ".join(types) + "]" if types else "None"
        if node.type == "unit_type":
            return "None"
        if node.type == "generic_type":
            base = text(field(node, "type")).split("::")[-1]
            arguments = [child for child in field(node, "type_arguments").named_children if child.type != "lifetime"]
            if base in {"PyResult", "Result", "Py", "Bound", "PyRef", "PyRefMut", "Box"}:
                return self.python_type(arguments[0], owner)
            if base == "Option":
                return f"Optional[{self.python_type(arguments[0], owner)}]"
            containers = {"Vec": "list", "HashMap": "dict", "BTreeMap": "dict", "HashSet": "set", "BTreeSet": "set"}
            if base in containers:
                return containers[base] + "[" + ", ".join(self.python_type(child, owner) for child in arguments) + "]"
        if node.type in {"array_type", "slice_type"}:
            return f"list[{self.python_type(field(node, 'element'), owner)}]"
        if value == "Self":
            return self.names[owner]
        if value in self.names:
            return self.names[value]
        if re.fullmatch(r"[iu](8|16|32|64|128|size)", value):
            return "int"
        primitives = {"f32": "float", "f64": "float", "bool": "bool", "String": "str", "str": "str", "char": "str", "PyAny": "Any", "PyObject": "Any", "PyBytes": "bytes"}
        if value in primitives:
            return primitives[value]
        raise ValueError(f"Unsupported Rust type: {value}")

    def signature(self, node: Node, attributes: list[str], owner: str) -> tuple[str, str]:
        arguments: dict[str, str] = {}
        receiver = False
        for parameter in field(node, "parameters").named_children:
            if parameter.type in {"line_comment", "block_comment", "attribute_item"}:
                continue
            if parameter.type == "self_parameter":
                receiver = True
                continue
            type_node = field(parameter, "type")
            if re.match(r"(?:pyo3::)?Python\s*<", text(type_node)):
                continue
            name = text(field(parameter, "pattern")).removeprefix("mut ").removeprefix("r#")
            if not name.isidentifier():
                raise ValueError(f"Unsupported parameter: {text(parameter)}")
            arguments[name] = self.python_type(type_node, owner)
        explicit = next((re.search(r"\bsignature\s*=\s*\((.*)\)\s*\)$", attribute, re.S) for attribute in attributes if re.search(r"\bsignature\s*=", attribute)), None)
        if any(re.search(r"\bsignature\s*=", attribute) for attribute in attributes) and explicit is None:
            raise ValueError(f"Unsupported signature attribute: {attributes}")
        parts: list[str] = []
        if owner and not has(attributes, "staticmethod"):
            parts.append("cls" if has(attributes, "classmethod") else "self")
        elif receiver:
            parts.append("self")
        for argument in split_arguments(explicit[1]) if explicit else arguments:
            if argument in {"/", "*"}:
                parts.append(argument)
                continue
            name, separator, default = argument.partition("=")
            name = name.strip()
            prefix = name[:len(name) - len(name.lstrip("*"))]
            name = name.lstrip("*")
            if name not in arguments:
                raise ValueError(f"Signature references unknown parameter {name}")
            annotation = arguments[name]
            if prefix == "*" and annotation.startswith("tuple["):
                annotation = "Any"
            elif prefix == "**" and annotation.startswith("dict["):
                annotation = "Any"
            parts.append(f"{prefix}{name}: {annotation}" + (f" = {default_value(default.strip())}" if separator else ""))
        result = self.python_type(node.child_by_field_name("return_type"), owner)
        if has(attributes, "new"):
            result = "None"
        return ", ".join(parts), result

    def render_method(self, node: Node, attributes: list[str], docs: str, owner: str = "") -> list[str]:
        name = option(attributes, "name") or text(field(node, "name")).removeprefix("r#")
        if has(attributes, "new"):
            name = "__init__"
        arguments, result = self.signature(node, attributes, owner)
        decorators = [f"@{decorator}" for decorator in ("staticmethod", "classmethod") if has(attributes, decorator)]
        if has(attributes, "getter") or has(attributes, "setter"):
            kind = "getter" if has(attributes, "getter") else "setter"
            attribute = next(value for value in attributes if re.match(rf"{kind}(?:\(|$)", value))
            explicit = re.fullmatch(rf'{kind}\(\s*(\w+|"[^"]+")\s*\)', attribute)
            name = explicit[1].strip('"') if explicit else name.removeprefix("get_").removeprefix("set_")
            decorators = ["@property"] if kind == "getter" else [f"@{name}.setter"]
        line = f"def {name}({arguments}) -> {result}:"
        if docs:
            return decorators + [line, "    " + repr(docs), "    ..."]
        return decorators + [line + " ..."]

    def generate(self) -> str:
        self.collect()
        exports = sorted([self.names[name] for name in self.export_classes] + [option(self.functions[name][1], "name") or name for name in self.export_functions])
        lines = ["# Generated by generate_pyi.py. Do not edit manually.", "from typing import Any, Optional", "", "__all__ = " + repr(exports), ""]
        for name in sorted(self.export_functions):
            lines.extend(self.render_method(*self.functions[name]))
            lines.append("")
        for name in sorted(self.classes, key=lambda value: self.names[value]):
            lines.extend(["", f"class {self.names[name]}:"])
            methods = self.methods.get(name, [])
            getters = [entry for entry in methods if has(entry[1], "getter")]
            others = [entry for entry in methods if not has(entry[1], "getter") and not has(entry[1], "setter")]
            setters = [entry for entry in methods if has(entry[1], "setter")]
            getter_names = {self.render_method(*entry, name)[1].split("(")[0].removeprefix("def ") for entry in getters}
            for method, attributes, docs in others + getters + setters:
                rendered = self.render_method(method, attributes, docs, name)
                if has(attributes, "setter"):
                    property_name = rendered[0][1:].removesuffix(".setter")
                    if property_name not in getter_names:
                        parameters = [parameter for parameter in field(method, "parameters").named_children if parameter.type == "parameter" and not re.match(r"Python\s*<", text(field(parameter, "type")))]
                        if len(parameters) != 1:
                            raise ValueError(f"Unexpected setter signature: {text(method)[:120]}")
                        annotation = self.python_type(field(parameters[0], "type"), name)
                        lines.extend([f"    {property_name}: {annotation}", ""])
                        continue
                lines.extend("    " + line for line in rendered)
                lines.append("")
            if not methods:
                lines.append("    ...")
        result = "\n".join(lines).rstrip() + "\n"
        ast.parse(result)
        return result


def api_surface(source: str) -> dict[str, str]:
    tree = ast.parse(source)
    aliases: dict[str, str] = {}
    for node in tree.body:
        if isinstance(node, ast.ImportFrom) and node.module == "typing":
            aliases.update({alias.asname or alias.name: {"List": "list", "Tuple": "tuple", "Dict": "dict", "Set": "set"}.get(alias.name, alias.name) for alias in node.names})

    class Normalize(ast.NodeTransformer):
        def visit_Name(self, node: ast.Name) -> ast.Name:
            return ast.copy_location(ast.Name(id=aliases.get(node.id, node.id), ctx=node.ctx), node)

    def normalized(node: ast.AST | None) -> str:
        return ast.unparse(Normalize().visit(node)) if node is not None else "Any"

    surface: dict[str, str] = {}

    def visit(nodes: list[ast.stmt], prefix: str = "") -> None:
        for node in nodes:
            if isinstance(node, ast.ClassDef) and not node.name.startswith("_"):
                surface[node.name] = "class"
                visit(node.body, node.name + ".")
            elif isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name):
                surface[prefix + node.target.id] = "attribute: " + normalized(node.annotation)
            elif isinstance(node, ast.FunctionDef) and node.name != "__getattr__":
                decorators = [ast.unparse(decorator) for decorator in node.decorator_list]
                if any(decorator.endswith(".setter") for decorator in decorators):
                    continue
                if "property" in decorators:
                    surface[prefix + node.name] = "attribute: " + normalized(node.returns)
                else:
                    arguments = normalized(node.args)
                    surface[prefix + node.name] = f"({arguments}) -> {normalized(node.returns)}" + (" @" + ",".join(decorators) if decorators else "")

    visit(tree.body)
    return surface


def compare(generated: str, existing: Path) -> str:
    before = api_surface(existing.read_text(encoding="utf-8-sig"))
    after = api_surface(generated)
    lines: list[str] = []
    for name in sorted(after.keys() - before.keys()):
        lines.append(f"+ {name}: {after[name]}")
    for name in sorted(before.keys() - after.keys()):
        lines.append(f"- {name}: {before[name]}")
    for name in sorted(before.keys() & after.keys()):
        if before[name] != after[name]:
            lines.extend([f"~ {name}", f"  old: {before[name]}", f"  new: {after[name]}"])
    return "\n".join(lines) or "No public API differences."


def main() -> int:
    directory = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=directory / "src" / "lib.rs")
    parser.add_argument("--output", type=Path, default=directory / "ms_toollib.generated.pyi")
    parser.add_argument("--compare", type=Path, help="Compare public API with an existing pyi")
    parser.add_argument("--check", action="store_true", help="Check output is current without writing")
    parser.add_argument("--diff", action="store_true", help="Also print a unified text diff when comparing")
    arguments = parser.parse_args()
    try:
        generator = Generator(arguments.source)
        generated = generator.generate()
        if arguments.compare:
            print(compare(generated, arguments.compare))
            if arguments.diff:
                print("".join(difflib.unified_diff(arguments.compare.read_text(encoding="utf-8-sig").splitlines(True), generated.splitlines(True), fromfile=str(arguments.compare), tofile=str(arguments.output))))
        if arguments.check:
            if not arguments.output.is_file() or arguments.output.read_text(encoding="utf-8-sig") != generated:
                print(f"Outdated or missing: {arguments.output}", file=sys.stderr)
                return 1
        else:
            arguments.output.write_text(generated, encoding="utf-8")
        print(f"{len(generator.export_functions)} functions, {len(generator.export_classes)} exported classes; {arguments.output}")
        return 0
    except (OSError, ValueError, SyntaxError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())