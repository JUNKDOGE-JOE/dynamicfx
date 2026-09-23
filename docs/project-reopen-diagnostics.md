# 工程重开提示“图层没有源”

Issue #12 尚未复现，不能据本机重开通过判定已修复。用户已决定将此项留待
后续处理，并先发布其余完成项。请保留原工程，只在副本上重试。

在受影响的 Windows 主机运行 PowerShell：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\collect-reopen-diagnostics.ps1
```

脚本在当前目录生成 `dynamicfx-reopen-diagnostics.json`，记录已安装的 AE
版本，以及专用 DynamicFx 目录中两个组件的 SHA-256。不会启动/关闭 AE，
不会读取工程、shader、日志或账号，也不会自动上传。非标准插件安装目录
需要另行说明。

再次遇到问题时，保留完整报错文字和错误码，并记录是否仅关闭工程后重开
就出现，还是必须退出并重启 AE。若能提供最小复现工程，请先移除私人素材。
这些信息用于建立修复前后对照；不要求直接修改原工程或禁用全部插件。
