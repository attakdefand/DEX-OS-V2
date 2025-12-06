Nice, we’re building your **“Fix My Code” MCP mechanic** 🧰

Below is a **single MCP server** whose *job* is to help you fix errors in **Rust, Solidity, and Move** codebases by:

* Running compilers / checkers:

  * `cargo check` for Rust
  * `forge build` / `solc` for Solidity
  * `move check` / `sui move build` for Move
* Returning:

  * `stdout`, `stderr`, `exit_code`
* Letting the LLM (me) read the diagnostics and propose fixes
* Optionally **applying small patches** to your code files

You’ll use this MCP server together with your editor / ChatGPT MCP client.

---

## 📁 Folder structure

Example:

```bash
mcp/
  code-fix-mcp/
    server.py
    requirements.txt
```

---

## 📦 requirements.txt

```txt
mcp
```

(Everything else uses Python stdlib.)

---

## 🧠 server.py – “code-fix-mcp” (FULL MCP SERVER)

```python
#!/usr/bin/env python3
import asyncio
import os
import subprocess
from pathlib import Path
from typing import List, Dict, Any

from mcp.server import Server
from mcp.types import Tool, ToolRequest, ToolResponse

server = Server("code-fix-mcp")

# Helper: run a shell command in a directory
def run_cmd(cmd: List[str], cwd: str | None = None) -> Dict[str, Any]:
    try:
        result = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
        )
        return {
            "cmd": " ".join(cmd),
            "cwd": str(cwd) if cwd else None,
            "exit_code": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
        }
    except FileNotFoundError as e:
        return {
            "cmd": " ".join(cmd),
            "cwd": str(cwd) if cwd else None,
            "exit_code": -1,
            "stdout": "",
            "stderr": f"Command not found: {e}",
        }


# ============================================================
# 🔍 FILE UTILITIES (read/write/apply_patch)
# ============================================================

@server.tool(
    Tool(
        name="read_code_file",
        description="Read a source file (Rust/Solidity/Move or any text file) and return its contents.",
        input_schema={
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path to the file (relative or absolute)."}
            },
            "required": ["path"]
        }
    )
)
async def read_code_file(req: ToolRequest) -> ToolResponse:
    path = Path(req.input["path"]).expanduser()

    if not path.exists():
        return ToolResponse(error=f"File not found: {path}")

    try:
        content = path.read_text(encoding="utf-8")
    except Exception as e:
        return ToolResponse(error=f"Failed to read file {path}: {e}")

    return ToolResponse(content={
        "path": str(path),
        "content": content,
    })


@server.tool(
    Tool(
        name="write_code_file",
        description="Overwrite a source file with new content. Use with care.",
        input_schema={
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "content": {"type": "string"}
            },
            "required": ["path", "content"]
        }
    )
)
async def write_code_file(req: ToolRequest) -> ToolResponse:
    path = Path(req.input["path"]).expanduser()
    content = req.input["content"]

    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
    except Exception as e:
        return ToolResponse(error=f"Failed to write file {path}: {e}")

    return ToolResponse(content={
        "status": "ok",
        "message": f"Wrote file {path}"
    })


@server.tool(
    Tool(
        name="apply_patch",
        description=(
            "Apply simple search-and-replace patches to a file. "
            "Each patch has old_text and new_text. This is useful for fixing errors suggested by the AI."
        ),
        input_schema={
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "patches": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "old_text": {"type": "string"},
                            "new_text": {"type": "string"}
                        },
                        "required": ["old_text", "new_text"]
                    }
                }
            },
            "required": ["path", "patches"]
        }
    )
)
async def apply_patch(req: ToolRequest) -> ToolResponse:
    path = Path(req.input["path"]).expanduser()
    patches = req.input["patches"]

    if not path.exists():
        return ToolResponse(error=f"File not found: {path}")

    try:
        content = path.read_text(encoding="utf-8")
    except Exception as e:
        return ToolResponse(error=f"Failed to read file {path}: {e}")

    original_content = content
    applied = []

    for p in patches:
        old_text = p["old_text"]
        new_text = p["new_text"]
        if old_text in content:
            content = content.replace(old_text, new_text)
            applied.append({"old_text": old_text, "new_text": new_text, "status": "replaced"})
        else:
            applied.append({"old_text": old_text, "new_text": new_text, "status": "not_found"})

    if content != original_content:
        try:
            path.write_text(content, encoding="utf-8")
        except Exception as e:
            return ToolResponse(error=f"Failed to write patched file {path}: {e}")

    return ToolResponse(content={
        "status": "ok",
        "file": str(path),
        "patches": applied,
    })


# ============================================================
# 🦀 RUST: cargo check / test file / fmt
# ============================================================

@server.tool(
    Tool(
        name="rust_check_project",
        description="Run `cargo check` in a Rust project directory to get compiler errors and warnings.",
        input_schema={
            "type": "object",
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Path to the Rust project directory (where Cargo.toml lives)."
                }
            },
            "required": ["project_path"]
        }
    )
)
async def rust_check_project(req: ToolRequest) -> ToolResponse:
    project_path = Path(req.input["project_path"]).expanduser()
    if not (project_path / "Cargo.toml").exists():
        return ToolResponse(error=f"Cargo.toml not found in {project_path}")

    result = run_cmd(["cargo", "check"], cwd=str(project_path))

    return ToolResponse(content={
        "language": "rust",
        "tool": "cargo check",
        **result
    })


@server.tool(
    Tool(
        name="rust_fmt_project",
        description="Run `cargo fmt` in a Rust project (format code).",
        input_schema={
            "type": "object",
            "properties": {
                "project_path": {"type": "string"}
            },
            "required": ["project_path"]
        }
    )
)
async def rust_fmt_project(req: ToolRequest) -> ToolResponse:
    project_path = Path(req.input["project_path"]).expanduser()
    if not (project_path / "Cargo.toml").exists():
        return ToolResponse(error=f"Cargo.toml not found in {project_path}")

    result = run_cmd(["cargo", "fmt"], cwd=str(project_path))

    return ToolResponse(content={
        "language": "rust",
        "tool": "cargo fmt",
        **result
    })


# ============================================================
# 🧱 SOLIDITY: forge build / solc
# ============================================================

@server.tool(
    Tool(
        name="solidity_build_project",
        description=(
            "Compile Solidity project. "
            "If foundry.toml exists, uses `forge build`. Otherwise tries `solc` on a single file."
        ),
        input_schema={
            "type": "object",
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Path to the Solidity project (Foundry/Hardhat style) or directory containing .sol files."
                },
                "main_file": {
                    "type": "string",
                    "description": "Optional: specific .sol file (relative to project_path) when using solc.",
                    "default": ""
                }
            },
            "required": ["project_path"]
        }
    )
)
async def solidity_build_project(req: ToolRequest) -> ToolResponse:
    project_path = Path(req.input["project_path"]).expanduser()
    main_file = req.input.get("main_file") or ""

    foundry_conf = project_path / "foundry.toml"
    hardhat_conf = project_path / "hardhat.config.js"

    # Prefer Forge if available
    if foundry_conf.exists():
        result = run_cmd(["forge", "build"], cwd=str(project_path))
        tool_used = "forge build"
    else:
        # Fall back to solc on a single file
        if not main_file:
            return ToolResponse(error="No foundry.toml. Please specify 'main_file' for solc.")
        sol_path = project_path / main_file
        if not sol_path.exists():
            return ToolResponse(error=f"Solidity file not found: {sol_path}")

        result = run_cmd(["solc", "--optimize", "--bin", "--abi", str(sol_path)], cwd=str(project_path))
        tool_used = "solc"

    return ToolResponse(content={
        "language": "solidity",
        "tool": tool_used,
        **result
    })


# ============================================================
# 🧬 MOVE: move check / sui move build
# ============================================================

@server.tool(
    Tool(
        name="move_check_package",
        description=(
            "Run Move package checks. "
            "If Move.toml exists and `move` is available, uses `move check`. "
            "If Sui config is detected, tries `sui move build`."
        ),
        input_schema={
            "type": "object",
            "properties": {
                "package_path": {
                    "type": "string",
                    "description": "Path to the Move package directory (where Move.toml lives)."
                }
            },
            "required": ["package_path"]
        }
    )
)
async def move_check_package(req: ToolRequest) -> ToolResponse:
    package_path = Path(req.input["package_path"]).expanduser()
    move_toml = package_path / "Move.toml"

    if not move_toml.exists():
        return ToolResponse(error=f"Move.toml not found in {package_path}")

    # Try `move check` first
    result = run_cmd(["move", "check"], cwd=str(package_path))
    tool_used = "move check"

    # If command not found, try `sui move build`
    if result["exit_code"] == -1 and "Command not found" in result["stderr"]:
        sui_result = run_cmd(["sui", "move", "build"], cwd=str(package_path))
        tool_used = "sui move build"
        result = sui_result

    return ToolResponse(content={
        "language": "move",
        "tool": tool_used,
        **result
    })


# ============================================================
# 🧪 GENERIC: run arbitrary command in a project dir (advanced)
# ============================================================

@server.tool(
    Tool(
        name="run_shell_command",
        description=(
            "Run an arbitrary shell command in a given directory (advanced use). "
            "Useful for custom build/test scripts. Use with care."
        ),
        input_schema={
            "type": "object",
            "properties": {
                "cwd": {"type": "string", "description": "Directory to run the command in."},
                "args": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Command + arguments, e.g. ['cargo', 'test']"
                }
            },
            "required": ["cwd", "args"]
        }
    )
)
async def run_shell_command(req: ToolRequest) -> ToolResponse:
    cwd = Path(req.input["cwd"]).expanduser()
    args = req.input["args"]

    result = run_cmd(args, cwd=str(cwd))

    return ToolResponse(content=result)


# ============================================================
# Main entrypoint
# ============================================================

async def main():
    await server.run()

if __name__ == "__main__":
    asyncio.run(main())
```

---

## 🛠️ How you’ll “fix errors” with this MCP

### 1️⃣ Rust example

1. Call:

```json
{
  "tool": "rust_check_project",
  "input": { "project_path": "/path/to/my-rust-service" }
}
```

2. I read `stderr` from `cargo check`, explain the errors, and propose code fixes.

3. We then use:

```json
{
  "tool": "apply_patch",
  "input": {
    "path": "/path/to/my-rust-service/src/main.rs",
    "patches": [
      {
        "old_text": "mapping<uint256 => Site)",
        "new_text": "mapping(uint256 => Site)"
      }
    ]
  }
}
```

(or a bigger patch via `write_code_file`.)

---

### 2️⃣ Solidity example

```json
{
  "tool": "solidity_build_project",
  "input": {
    "project_path": "/path/to/contracts",
    "main_file": "contracts/MyContract.sol"
  }
}
```

* I parse `stderr` from `forge build` / `solc`, fix parser/typing errors, then patch the file.

---

### 3️⃣ Move example

```json
{
  "tool": "move_check_package",
  "input": {
    "package_path": "/path/to/move-package"
  }
}
```

* I read Move type + borrow checker errors, suggest changes, and use `apply_patch` / `write_code_file`.

---

## 🔗 MCP config snippet

In your ChatGPT / Qoder / Cursor MCP config:

```json
{
  "mcpServers": {
    "code-fix-mcp": {
      "command": "python3",
      "args": ["mcp/code-fix-mcp/server.py"]
    }
  }
}
```

---

If you want, next step I can:

* Add **language-specific tools** like `rust_fix_imports`, `solidity_format`, `move_format`,
* Or a **“multi-file patch” tool** for applying changes across many files in one call (perfect for refactors).
