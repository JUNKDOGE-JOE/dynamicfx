# Installing the DynamicFX Shader Skill

This folder is an AI-agent skill that teaches AI coding assistants (Claude Code, Cursor, etc.) how to write and port shaders for the [DynamicFX](https://github.com/JUNKDOGE-JOE/dynamicfx) After Effects plugin.

> **Note:** The one-line installers below assume this folder is published at `skills/dynamicfx-shaders/` on the repository's `main` branch. Until then, copy the folder manually.

## One-line install (paste to your AI assistant)

**English:**

> Download SKILL.md, porting.md and reference.md from https://raw.githubusercontent.com/JUNKDOGE-JOE/dynamicfx/main/skills/dynamicfx-shaders/ and save all three into .claude/skills/dynamicfx-shaders/ in this project, then confirm the skill is installed.

**中文：**

> 请下载 https://raw.githubusercontent.com/JUNKDOGE-JOE/dynamicfx/main/skills/dynamicfx-shaders/ 目录下的 SKILL.md、porting.md、reference.md 三个文件，保存到本项目 .claude/skills/dynamicfx-shaders/ 目录，完成后确认 skill 已安装。

## One-line install (shell)

```bash
mkdir -p .claude/skills/dynamicfx-shaders && for f in SKILL.md porting.md reference.md; do curl -fsSL "https://raw.githubusercontent.com/JUNKDOGE-JOE/dynamicfx/main/skills/dynamicfx-shaders/$f" -o ".claude/skills/dynamicfx-shaders/$f"; done
```

Install to `~/.claude/skills/dynamicfx-shaders/` instead for user-wide availability across all projects.

## After installing

Ask your AI assistant things like:

- "Convert this Shadertoy shader to DynamicFX" (paste the shader)
- "Batch-convert every .glsl file in ./shaders to DynamicFX format"
- "Write a DynamicFX shader that does <effect> with keyframeable controls"
- "My DynamicFX Source expression fails with E32 — fix it"

The assistant will follow the skill's envelope/ABI rules, validation checklist, and porting tables instead of guessing.
