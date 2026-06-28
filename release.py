"""
用法: python release.py 1.5.13

依次执行:
  1. 检查当前在 main 分支、gh CLI 已登录
  2. 更新 python_package/pyproject.toml 和 wasm/Cargo.toml 的版本号
  3. 更新 README.md 和 README.en.md 版本表中 Python / WASM 的行
  4. git add 这四个文件 → git commit -m "release: 发布1.5.13" → git tag 1.5.13
  5. git push origin main + git push origin 1.5.13
  6. 用 gh CLI 查找触发的 CI 和 npm_CI workflow run
  7. 依次 gh run watch 实时输出日志，等待两个 workflow 完成
  8. 全部成功则退出码 0，任一失败则退出码 1

前置依赖:
  - gh CLI (https://cli.github.com/) 已安装并 gh auth login
  - 本地 git 已配置推送凭证（SSH / GCM / PAT）
  - GitHub 仓库已配置 secrets: PYPI_API_TOKEN、NODE_AUTH_TOKEN

不改动的文件:
  - python_package/Cargo.toml（无关）
  - base/Cargo.toml（手动发布）
  - c/、c++/、java/ 版本（手动发布）
  - WASM (Node.js) 版本号（由 CI 自动追加 -alpha）
"""

import re
import sys
import subprocess
import time
import json
from pathlib import Path
from datetime import datetime, timedelta


def die(msg: str):
    print(f"Error: {msg}", file=sys.stderr)
    sys.exit(1)


def run(cmd: list[str], check=True, capture=False, timeout=600):
    """Run a command, return CompletedProcess."""
    try:
        return subprocess.run(
            cmd,
            capture_output=capture,
            text=True,
            check=check,
            timeout=timeout,
            encoding="utf-8",
        )
    except subprocess.CalledProcessError as e:
        if check:
            die(f"Command failed: {' '.join(cmd)}\n{e.stderr}")
        return e
    except FileNotFoundError:
        die(f"Command not found: {cmd[0]}")


def get_gh_run_list(workflow: str) -> list[dict]:
    result = run(
        ["gh", "run", "list", "--workflow", workflow, "--limit", "1",
         "--json", "databaseId,status,conclusion"],
        capture=True,
        check=False,
    )
    if result.returncode != 0 or not result.stdout.strip():
        return []
    return json.loads(result.stdout)


def main():
    if len(sys.argv) < 2:
        print(__doc__.strip())
        sys.exit(1)

    version = sys.argv[1]
    timeout_seconds = int(sys.argv[2]) if len(sys.argv) > 2 else 1800

    if not re.match(r"^\d+\.\d+\.\d+$", version):
        die(f"Version must be in format like 1.5.13, got: {version}")

    root = Path(__file__).resolve().parent
    os.chdir(str(root))

    # Check branch
    result = run(["git", "rev-parse", "--abbrev-ref", "HEAD"], capture=True)
    if result.stdout.strip() != "main":
        die(f"Must be on main branch, currently on {result.stdout.strip()}")

    # Check gh
    try:
        run(["gh", "--version"], capture=True)
    except:
        die("gh CLI is required. Install from https://cli.github.com/")

    result = run(["gh", "auth", "status"], capture=True, check=False)
    if result.returncode != 0:
        die("gh is not authenticated. Run 'gh auth login' first.")

    result = run(["git", "remote", "get-url", "origin"], capture=True)
    remote_url = result.stdout.strip()
    if not remote_url:
        die("No remote 'origin' configured.")

    # Check if version already matches
    pyproject = root / "python_package" / "pyproject.toml"
    wasm_cargo = root / "wasm" / "Cargo.toml"

    current_py = None
    m = re.search(r'^version = "(.+)"', pyproject.read_text("utf-8"), re.MULTILINE)
    if m:
        current_py = m.group(1)

    current_wasm = None
    m = re.search(r'^version = "(.+)"', wasm_cargo.read_text("utf-8"), re.MULTILINE)
    if m:
        current_wasm = m.group(1)

    if current_py == version and current_wasm == version:
        print(f"Version {version} already set in all files, nothing to do.")
        return

    print(f"""
============================================
  Releasing v{version}
  Remote: {remote_url}
  Timeout: {timeout_seconds}s
============================================
""")

    # === 1. Update version files ===
    print("=== 1. Updating version files ===")

    content = pyproject.read_text("utf-8")
    content = re.sub(
        r'^version = "[\d.]+(-[a-zA-Z0-9.]+)?"',
        f'version = "{version}"',
        content,
        count=1,
        flags=re.MULTILINE,
    )
    pyproject.write_text(content, "utf-8")
    print(f"  Updated {pyproject.relative_to(root)}")

    content = wasm_cargo.read_text("utf-8")
    content = re.sub(
        r'^version = "[\d.]+(-[a-zA-Z0-9.]+)?"',
        f'version = "{version}"',
        content,
        count=1,
        flags=re.MULTILINE,
    )
    wasm_cargo.write_text(content, "utf-8")
    print(f"  Updated {wasm_cargo.relative_to(root)}")

    # README.md
    readme = root / "README.md"
    content = readme.read_text("utf-8")
    content = re.sub(
        r"(?m)(?<=^\| Python \| )[\d.]+(-[a-zA-Z0-9.]+)?(?= \|)", version, content
    )
    content = re.sub(
        r"(?m)(?<=^\| WASM \(bundler\) \| )[\d.]+(-[a-zA-Z0-9.]+)?(?= \|)",
        version,
        content,
    )
    readme.write_text(content, "utf-8")
    print(f"  Updated {readme.relative_to(root)}")

    # README.en.md
    readme_en = root / "README.en.md"
    content = readme_en.read_text("utf-8")
    content = re.sub(
        r"(?m)(?<=^\| Python \| )[\d.]+(-[a-zA-Z0-9.]+)?(?= \|)", version, content
    )
    content = re.sub(
        r"(?m)(?<=^\| WASM \(bundler\) \| )[\d.]+(-[a-zA-Z0-9.]+)?(?= \|)",
        version,
        content,
    )
    readme_en.write_text(content, "utf-8")
    print(f"  Updated {readme_en.relative_to(root)}")

    # === 2. Show changes ===
    print("\n=== 2. Changes summary ===")
    run(["git", "diff", "--stat"])

    # === 3. Commit, tag, push ===
    print("\n=== 3. Commit, tag, push ===")
    run(["git", "add",
          "python_package/pyproject.toml",
          "wasm/Cargo.toml",
          "README.md",
          "README.en.md"])
    run(["git", "commit", "-m", f"release: 发布{version}"])
    run(["git", "tag", version])

    print("Pushing to origin...")
    run(["git", "push", "origin", "main"])
    run(["git", "push", "origin", version])

    # === 4. Wait for CI ===
    print("\n=== 4. Waiting for CI to complete ===")
    print("""
  Repo secrets required on GitHub:
    PYPI_API_TOKEN  - PyPI API token (publish to pypi.org)
    NODE_AUTH_TOKEN - npm access token (publish to npmjs.com)
""")

    deadline = datetime.utcnow() + timedelta(seconds=timeout_seconds)
    workflows = ["CI", "npm_CI"]
    run_ids = {}

    for wf in workflows:
        print(f"  Waiting for {wf} run to start...")
        run_id = None
        while datetime.utcnow() < deadline:
            runs = get_gh_run_list(wf)
            if runs:
                run_id = runs[0]["databaseId"]
                break
            time.sleep(5)
        if not run_id:
            die(f"Timeout waiting for {wf} run to start")
        run_ids[wf] = run_id
        print(f"  {wf} run started: {run_id}")

    failed = False
    for wf in workflows:
        run_id = run_ids[wf]
        print(f"\n  Watching {wf} (#{run_id})...")
        run(["gh", "run", "watch", str(run_id)], check=False, timeout=timeout_seconds)

        result = run(
            ["gh", "run", "view", str(run_id), "--json", "conclusion", "--jq", ".conclusion"],
            capture=True,
            check=False,
        )
        conclusion = result.stdout.strip()
        if conclusion != "success":
            print(f"  {wf} result: {conclusion}")
            failed = True
        else:
            print(f"  {wf} result: success")

    print()
    if failed:
        print("============================================")
        print(f"  Release v{version} FAILED")
        print("============================================")
        sys.exit(1)
    else:
        print("============================================")
        print(f"  Release v{version} SUCCESS")
        print("============================================")


if __name__ == "__main__":
    import os
    main()
