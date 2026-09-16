---
name: agent-tool-catalog
description: >-
  Choose external agent skills, MCP servers, and self-host utilities from
  ebluffy/agent-tool-catalog instead of inventing tooling from memory. Use when
  picking UI/design skills, MCP, token compression, search, Windows computer-use,
  OCR, or security tooling for this workspace.
---

# Agent tool catalog

Source of truth (fetch live, do not rely on memory):

https://github.com/ebluffy/agent-tool-catalog

Raw README:

https://raw.githubusercontent.com/ebluffy/agent-tool-catalog/main/README.md

## When to use

Before installing or recommending a new skill, MCP, UI anti-slop pack, Windows desktop MCP, OCR stack, search/archive utility, or security scanner — **read that README** and prefer rows listed there over inventing alternatives.

## Owyx defaults (subset)

For this monorepo (`apps/app*`, `owyxsite/`, `brand/`), prefer:

| Need | Catalog pick |
|------|----------------|
| UI / anti-AI-slop | [hallmark](https://github.com/Nutlope/hallmark) + [ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill) — still respect `brand/DESIGN.md` and Owyx tokens first |
| Engineering process skills | [mattpocock/skills](https://github.com/mattpocock/skills), [addyosmani/agent-skills](https://github.com/addyosmani/agent-skills), [spec-kit](https://github.com/github/spec-kit) |
| Token / log compression | [headroom](https://github.com/headroomlabs-ai/headroom), [rtk](https://github.com/rtk-ai/rtk) |
| Code graph MCP | [code-review-graph](https://github.com/tirth8205/code-review-graph) or [codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp) |
| Windows desktop MCP | [Windows-MCP](https://github.com/CursorTouch/Windows-MCP) (default) |
| Skill install safety | scan with [SkillSpector](https://github.com/NVIDIA/SkillSpector) before adding third-party skills |

Do **not** pull Unity, trading, manga, or unrelated companion stacks into Owyx work unless the task explicitly needs them.

## Repo skills stay primary

Existing `.agents/skills/` (api-module, cross-platform-pages, figma-mcp, i18n-pass, review-changelog, tanstack-query) and root/`owyxsite` `AGENTS.md` win for Modrinth-fork and Owyx product conventions. The catalog only fills gaps for **external** tooling.

## Do not

- Commit secrets, private project names, or live keys into the catalog or this skill.
- Vendor entire third-party skill trees into this fork unless the user asks — link + install command is enough.
- Run pentest tools (`shannon`, `pentagi`) except against **our own** staging with explicit approval.
