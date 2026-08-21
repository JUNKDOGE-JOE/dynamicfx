"""Minimal client for the ae-mcp panel's loopback /exec channel (warm AE session)."""
import json, os, sys, urllib.request, urllib.error
_PORT = None
def _token():
    with open(os.path.expanduser("~/.ae-mcp/auth-token"), encoding="utf-8") as f:
        return f.read().strip()
def _probe(port):
    try:
        with urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=0.4) as r:
            return json.loads(r.read().decode("utf-8"))
    except Exception:
        return None
def port():
    global _PORT
    if _PORT: return _PORT
    env = os.environ.get("AEMCP_PORT")
    cands = ([int(env)] if env else []) + [11488, 11480] + list(range(11470, 11500))
    for p in cands:
        if _probe(p) is not None:
            _PORT = p; return p
    raise SystemExit("ae-mcp panel /health not reachable on 11470-11499")
def health():
    return _probe(port())
def exec_js(code, timeout_ms=60000, undo=None):
    body = {"code": code, "timeoutMs": timeout_ms}
    if undo: body["undoGroup"] = undo
    req = urllib.request.Request(f"http://127.0.0.1:{port()}/exec", data=json.dumps(body).encode("utf-8"),
        headers={"Content-Type": "application/json", "x-ae-mcp-token": _token(), "x-ae-mcp-client": "claude-code"})
    try:
        with urllib.request.urlopen(req, timeout=timeout_ms/1000 + 5) as r:
            return json.loads(r.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        return {"ok": False, "error": f"HTTP {e.code}: {e.read().decode('utf-8', 'replace')[:400]}"}
if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else "health"
    if cmd == "health":
        h = health(); print(json.dumps({k: v for k, v in h.items() if k != "token"}, ensure_ascii=False)[:600], "port", port())
    elif cmd == "exec":
        src = sys.argv[2]
        code = open(src, encoding="utf-8").read() if os.path.exists(src) else src
        print(json.dumps(exec_js(code), ensure_ascii=False)[:2000])
