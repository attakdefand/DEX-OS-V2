# DEX-OS-V2 Setup Walkthrough

## MCP Server Setup

### Overview
We have successfully set up a Model Context Protocol (MCP) server for the DEX-OS-V2 project. This server allows AI agents to interact with the project by exposing tools for building, testing, and retrieving project information.

### Components
- **Location**: `d:\DEX-OS-V2\mcp-server`
- **Language**: TypeScript
- **Dependencies**: `@modelcontextprotocol/sdk`, `zod`

### Available Tools
The server exposes the following tools:
1.  **get_project_info**: Returns summary information from `README.md` and `Cargo.toml`.
2.  **run_build**: Executes `cargo build` in the project root.
3.  **run_tests**: Executes `cargo test` in the project root.
4.  **list_components**: Lists the main sub-directories (crates) in the project.

### Verification Results
We verified the server by running it locally and sending a JSON-RPC `tools/list` request.
- **Input**: `{"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}`
- **Output**:
  ```json
  {
    "result": {
      "tools": [
        { "name": "get_project_info", ... },
        { "name": "run_build", ... },
        { "name": "run_tests", ... },
        { "name": "list_components", ... }
      ]
    },
    "jsonrpc": "2.0",
    "id": 1
  }
  ```

### How to Use
To use this server with an MCP client (like Claude Desktop or another agent):
1.  **Build**: `npm run build` (inside `mcp-server`)
2.  **Run**: `node dist/index.js`
3.  **Configure**: Add the server configuration to your MCP client settings.
    ```json
    {
      "mcpServers": {
        "dex-os": {
          "command": "node",
          "args": ["d:/DEX-OS-V2/mcp-server/dist/index.js"]
        }
      }
    }
    ```

## AI Agent Setup

### OpenAI AgentKit (Node.js)
We have set up a Node.js project for creating AI agents using the OpenAI SDK.

- **Location**: `d:\DEX-OS-V2\ai-agents`
- **Setup**:
    1.  Navigate to `d:\DEX-OS-V2\ai-agents`.
    2.  Run `npm install`.
    3.  Create a `.env` file with your `OPENAI_API_KEY`.
    4.  Run `npm start` to test the agent.

### DeepSeek V3 (Local via Ollama)
We have installed Ollama to run the DeepSeek V3 model.

- **Installation**: Ollama was installed via `winget`.
- **Running DeepSeek V3**:
    > [!WARNING]
    > DeepSeek V3 (671B) is extremely large and requires hundreds of GBs of RAM.
    
    To run it, open a **NEW terminal** (to refresh PATH) and run:
    ```bash
    ollama pull deepseek-v3
    ollama run deepseek-v3
    ```
    If your hardware cannot handle it, consider using a smaller model like `deepseek-coder-v2` or `llama3`.
