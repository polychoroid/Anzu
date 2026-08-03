---
description: "Use when the user requests documentation-only work, learning-mode guidance, architecture explanation, or no-code sessions. Enforces doc-only edits and prohibits source-code changes."
name: "Documentation-Only Session Guardrail"
applyTo: ["README.md", "BACKLOG.md", "documents/**/*.md", ".github/**/*instructions.md", ".github/agents/*.agent.md"]
---
# Documentation-Only Session Guardrail

Use this instruction when the session intent is learning, explanation, planning, audit writeups, or documentation maintenance.

Universal response policy
- Apply `.github/instructions/universal-structured-response.instructions.md` to all responses and doc edits in this mode.
- Use terms from `.github/response-dictionary.md` where applicable.

## Required behavior
- Do not create, edit, or refactor source code files.
- Do not modify build scripts, config files, or dependency manifests unless they are documentation files.
- Restrict file edits to documentation paths only.
- Prefer factual explanations with references to repository docs and authoritative external docs.

## Allowed edit scope
- README and markdown docs
- backlog/planning documents
- instruction and agent markdown files under `.github/`

## If asked to write code
Respond with a documentation-first alternative:
1. Explain the design and rationale.
2. Propose implementation steps as a checklist.
3. Offer to update documentation to prepare for implementation.

## Quality bar for doc updates
- Keep language clear, concrete, and teachable.
- Separate facts from assumptions.
- Include acceptance criteria when adding plans/tasks.
- Keep edits scoped to the user request; avoid unrelated rewrites.
- Ask narrowing questions first when scope is broad.
- Default to short mode unless the user requests detail.
- Keep citations compact and defer deep references unless requested.
