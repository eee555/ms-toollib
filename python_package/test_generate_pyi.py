"""Regression checks for source-driven PyO3 stub generation."""

import ast
import tempfile
import unittest
from pathlib import Path

from generate_pyi import Generator, api_surface, compare, default_value, split_arguments


DIRECTORY = Path(__file__).resolve().parent


class GeneratorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.generator = Generator(DIRECTORY / "src" / "lib.rs")
        cls.output = cls.generator.generate()
        cls.surface = api_surface(cls.output)

    def test_registered_exports(self) -> None:
        self.assertEqual(len(self.generator.export_functions), 31)
        self.assertEqual(len(self.generator.export_classes), 15)
        self.assertNotIn("cal_probability", self.surface)
        self.assertIn("cal_probability_enum", self.surface)
        self.assertIn("cal_probability_csp", self.surface)
        self.assertNotIn("__getattr__", self.surface)

    def test_current_signatures(self) -> None:
        self.assertEqual(self.surface["cal_zini"], "(board: list[list[int]]) -> int")
        self.assertEqual(self.surface["cal_rzini"], "(board: list[list[int]], loop_count: int=100) -> int")
        self.assertEqual(self.surface["BaseVideo.__init__"], "(self, board: list[list[int]], cell_pixel_size: int=16) -> None")
        self.assertEqual(self.surface["Board.hzini"], "attribute: int")

    def test_macro_classes(self) -> None:
        for name in ("AvfVideo", "EvfVideo", "MvfVideo", "RmvVideo"):
            self.assertEqual(self.surface[f"{name}.__init__"], "(self, file_name: str='', raw_data: list[int]=...) -> None")
            self.assertEqual(self.surface[f"{name}.hzini"], "attribute: int")
            self.assertEqual(self.surface[f"{name}.parse"], "(self) -> None")

    def test_setters_and_private_support_types(self) -> None:
        self.assertEqual(self.surface["BaseVideo.current_time"], "attribute: float")
        self.assertEqual(self.surface["BaseVideo.pix_size"], "attribute: int")
        self.assertEqual(self.surface["VideoActionStateRecorder.event"], "attribute: _Event")
        self.assertNotIn("Event", self.surface)
        tree = ast.parse(self.output)
        for node in tree.body:
            if isinstance(node, ast.ClassDef):
                defined: set[str] = set()
                for method in node.body:
                    if isinstance(method, ast.FunctionDef):
                        decorators = [ast.unparse(value) for value in method.decorator_list]
                        for decorator in decorators:
                            if decorator.endswith(".setter"):
                                self.assertIn(decorator.removesuffix(".setter"), defined)
                        defined.add(method.name)

    def test_nested_types_and_helpers(self) -> None:
        self.assertEqual(self.surface["refresh_matrix"], "(game_board: list[list[int]]) -> tuple[list[list[int]], list[tuple[int, int]], list[int]]")
        self.assertEqual(self.surface["SafeBoard.__getitem__"], "(self, key: int) -> SafeBoardRow")
        self.assertEqual(split_arguments('name="a,b", data=vec![1, 2], *, flag=true'), ['name="a,b"', 'data=vec![1, 2]', '*', 'flag=true'])
        self.assertEqual(default_value('"a_b"'), "'a_b'")
        self.assertEqual(default_value("1_000"), "1000")
        self.assertEqual(default_value("vec![]"), "...")

    def test_comparison_normalizes_typing_aliases(self) -> None:
        old = "from typing import List as L, Tuple\ndef example(value: L[int]) -> Tuple[str, int]: ...\n"
        new = "def example(value: list[int]) -> tuple[str, int]: ...\n"
        self.assertEqual(api_surface(old), api_surface(new))
        with tempfile.TemporaryDirectory() as directory:
            previous = Path(directory) / "previous.pyi"
            previous.write_text(
                "def cal_zini(board: list[list[int]], loop_count: int) -> int: ...\n",
                encoding="utf-8",
            )
            differences = compare(self.output, previous)
        self.assertIn("~ cal_zini", differences)
        self.assertIn("+ cal_hzini", differences)

    def test_unknown_types_fail(self) -> None:
        node = self.generator.parse("type Example = Mystery;", "fixture")
        alias = node.named_children[0]
        with self.assertRaisesRegex(ValueError, "Unsupported Rust type"):
            self.generator.python_type(alias.child_by_field_name("type"))

    def test_comments_and_signature_markers(self) -> None:
        source = '''
#[pyfunction]
#[pyo3(name = "renamed", signature = (value, /, *, flag=false))]
fn example(value: i32, /* ignored */ flag: bool, py: Python<'_>) -> Option<String> { None }
// #[pyfunction] fn fake() {}
#[pymodule]
fn fixture(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(example, m)?)?;
    Ok(())
}
'''
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "lib.rs"
            path.write_text(source, encoding="utf-8")
            output = Generator(path).generate()
        self.assertEqual(api_surface(output)["renamed"], "(value: int, /, *, flag: bool=False) -> Optional[str]")
        self.assertNotIn("fake", output)


if __name__ == "__main__":
    unittest.main()