"""Local regression client for the AE-MCP Streamable HTTP endpoint."""
import json
import urllib.request


class Client:
    def __init__(self, port=11488):
        self.url = f"http://127.0.0.1:{port}/mcp"
        self.session = None
        self.next_id = 0
        self.request("initialize", {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "Codex DynamicFX regression", "version": "1"},
        })
        self.request("notifications/initialized", {}, notification=True)

    def request(self, method, params, notification=False, timeout=60):
        self.next_id += 1
        body = {"jsonrpc": "2.0", "method": method, "params": params}
        if not notification:
            body["id"] = self.next_id
        headers = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream"}
        if self.session:
            headers["Mcp-Session-Id"] = self.session
        request = urllib.request.Request(self.url, json.dumps(body).encode(), headers)
        with urllib.request.urlopen(request, timeout=timeout) as response:
            self.session = response.headers.get("Mcp-Session-Id", self.session)
            raw = response.read().decode("utf-8")
            content_type = response.headers.get("Content-Type", "")
        if not raw.strip():
            return None
        if "text/event-stream" in content_type:
            messages = [json.loads(line[5:].strip()) for line in raw.splitlines() if line.startswith("data:")]
            result = next(message for message in reversed(messages) if message.get("id") == body.get("id"))
        else:
            result = json.loads(raw)
        if "error" in result:
            raise RuntimeError(json.dumps(result["error"]))
        return result.get("result")

    def call(self, name, arguments, timeout=60):
        return self.request("tools/call", {"name": name, "arguments": arguments}, timeout=timeout)

    @staticmethod
    def data(result):
        if "structuredContent" in result:
            return result["structuredContent"]
        texts = [block["text"] for block in result.get("content", []) if block.get("type") == "text"]
        return json.loads("\n".join(texts))


if __name__ == "__main__":
    import sys
    from pathlib import Path
    client = Client()
    name, arguments_path, output_path = sys.argv[1:4]
    arguments = json.loads(Path(arguments_path).read_text(encoding="utf-8"))
    result = client.call(name, arguments)
    Path(output_path).write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(Client.data(result), ensure_ascii=False)[:12000])
