"""
用法: python release.py 1.5.15

依次执行:
  1. 检查版本号大于所有已有的版本号（pyproject、Cargo、README）
  2. 更新 python_package/pyproject.toml 和 wasm/Cargo.toml 的版本号
  3. 更新 README.md 和 README.en.md 版本表中 Python / WASM 的行
  4. git add --all → git commit -m "release: 发布1.5.15" → git tag 1.5.15

不改动的文件:
  - base/Cargo.toml（手动发布）
  - c/、c++/、java/ 版本（手动发布）
  - WASM (Node.js) 版本号（由 CI 自动追加 -alpha）
"""

import re
import sys
import subprocess
from pathlib import Path


def die(msg: str):
    print(f"Error: {msg}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], check=True, capture=False, timeout=600):
    try:
        return subprocess.run(
            cmd, capture_output=capture, text=True, check=check,
            timeout=timeout, encoding="utf-8",
        )
    except subprocess.CalledProcessError as e:
        if check:
            die(f"Command failed: {' '.join(cmd)}\n{e.stderr}")
        return e
    except FileNotFoundError:
        die(f"Command not found: {cmd[0]}")


def parse_version(v: str) -> tuple[int, int, int]:
    parts = v.split(".")
    return int(parts[0]), int(parts[1]), int(parts[2])


def format_version(t: tuple[int, int, int]) -> str:
    return f"{t[0]}.{t[1]}.{t[2]}"


def main():
    if len(sys.argv) < 2:
        print(__doc__.strip())
        sys.exit(1)

    version = sys.argv[1]

    if not re.match(r"^\d+\.\d+\.\d+$", version):
        die(f"Version must be in format like 1.5.15, got: {version}")

    new_ver = parse_version(version)
    root = Path(__file__).resolve().parent

    # === 1. Collect all current versions ===
    pyproject = root / "python_package" / "pyproject.toml"
    wasm_cargo = root / "wasm" / "Cargo.toml"
    readme = root / "README.md"
    readme_en = root / "README.en.md"

    current_versions: dict[str, tuple[int, int, int]] = {}

    m = re.search(r'^version = "(.+)"', pyproject.read_text("utf-8"), re.MULTILINE)
    if m:
        current_versions["pyproject.toml"] = parse_version(m.group(1))

    m = re.search(r'^version = "(.+)"', wasm_cargo.read_text("utf-8"), re.MULTILINE)
    if m:
        current_versions["wasm/Cargo.toml"] = parse_version(m.group(1))

    # README.md — Python / WASM
    for key, pattern in [
        ("README.md Python", r"(?m)(?<=^\| Python \| )[\d.]+(?= \|)"),
        ("README.md WASM", r"(?m)(?<=^\| WASM \(bundler\) \| )[\d.]+(?= \|)"),
    ]:
        m = re.search(pattern, readme.read_text("utf-8"))
        if m:
            current_versions[key] = parse_version(m.group(0))

    # README.en.md — Python / WASM
    for key, pattern in [
        ("README.en.md Python", r"(?m)(?<=^\| Python \| )[\d.]+(?= \|)"),
        ("README.en.md WASM", r"(?m)(?<=^\| WASM \(bundler\) \| )[\d.]+(?= \|)"),
    ]:
        m = re.search(pattern, readme_en.read_text("utf-8"))
        if m:
            current_versions[key] = parse_version(m.group(0))

    print("Current versions:")
    for name, ver in current_versions.items():
        ver_str = format_version(ver)
        if new_ver > ver:
            ok = "OK"
        elif new_ver < ver:
            ok = "LOWER"
        else:
            ok = "SAME"
        print(f"  {name}: {ver_str}  [{ok}]")

    # === 2. Validate new version > ALL current versions ===
    for name, ver in current_versions.items():
        if new_ver < ver:
            die(f"Version {version} is less than existing {name} ({format_version(ver)})")
        if new_ver == ver:
            die(f"Version {version} already matches {name} ({format_version(ver)}), bump it")

    print(f"\nNew version {version} passed validation.\n")

    # === 3. Update version files ===
    print("--- Updating version files ---")

    content = pyproject.read_text("utf-8")
    content = re.sub(
        r'^version = "[\d.]+(-[a-zA-Z0-9.]+)?"',
        f'version = "{version}"',
        content, count=1, flags=re.MULTILINE,
    )
    pyproject.write_text(content, "utf-8")
    print(f"  Updated {pyproject.relative_to(root)}")

    content = wasm_cargo.read_text("utf-8")
    content = re.sub(
        r'^version = "[\d.]+(-[a-zA-Z0-9.]+)?"',
        f'version = "{version}"',
        content, count=1, flags=re.MULTILINE,
    )
    wasm_cargo.write_text(content, "utf-8")
    print(f"  Updated {wasm_cargo.relative_to(root)}")

    # README.md
    content = readme.read_text("utf-8")
    for pattern in [
        r"(?m)(?<=^\| Python \| )[\d.]+(-[a-zA-Z0-9.]+)?(?= \|)",
        r"(?m)(?<=^\| WASM \(bundler\) \| )[\d.]+(-[a-zA-Z0-9.]+)?(?= \|)",
    ]:
        content = re.sub(pattern, version, content)
    readme.write_text(content, "utf-8")
    print(f"  Updated {readme.relative_to(root)}")

    # README.en.md
    content = readme_en.read_text("utf-8")
    for pattern in [
        r"(?m)(?<=^\| Python \| )[\d.]+(-[a-zA-Z0-9.]+)?(?= \|)",
        r"(?m)(?<=^\| WASM \(bundler\) \| )[\d.]+(-[a-zA-Z0-9.]+)?(?= \|)",
    ]:
        content = re.sub(pattern, version, content)
    readme_en.write_text(content, "utf-8")
    print(f"  Updated {readme_en.relative_to(root)}")

    # === 4. Show changes ===
    print("\n--- Changes summary ---")
    run(["git", "diff", "--stat"])

    # === 5. Commit, tag ===
    print(f"\n--- git add --all && git commit && git tag ---")
    run(["git", "add", "--all"])
    run(["git", "commit", "-m", f"release: 发布{version}"])
    run(["git", "tag", version])

    print(f"\nDone. Release v{version} committed and tagged locally.")
    print("Run 'git push origin main --tags' to push when ready.")


if __name__ == "__main__":
    main()
