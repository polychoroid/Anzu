---
description: "Use for all agent interactions. Enforces short-mode output, structured language, narrowing questions first, and low-cost evidence defaults."
name: "Universal Structured Response Policy"
applyTo: ["**/*"]
---
# Universal Structured Response Policy

Use this policy for all chat responses and written content unless the user asks to override it.

For every chat response, also apply `.github/skills/asd-ste100-chat-responses/SKILL.md`. Use ASD-STE100 Issue 9 for common English and `.github/response-dictionary.md` for Anzu-specific terminology.

## Required interaction order
1. Ask narrowing questions first when the request is broad, ambiguous, or has multiple valid scopes.
2. After answers are received, give a direct answer.
3. If the request is already precise, skip questions and answer directly.

## Short mode default
- Answer only what the user asks. Do not restate the request or add background, alternatives, or follow-up suggestions unless they are needed.
- For a simple request, use 1-2 sentences. For a task update, state the change and its validation in no more than 3 concise sentences.
- For a complex answer, lead with the conclusion and include only the evidence needed to support it. Use a list only when it improves scanning.
- Do not use a fixed decision/why/next-action format. Expand only when the task needs the detail or the user asks for it.

## Structured language rules (strict but practical)
- Use clear, direct sentences.
- Use consistent terms for the same concept.
- Use numbered steps for procedures.
- Put the main point first.
- Avoid unnecessary qualifiers and repeated context.
- Keep one main idea per sentence when practical.
- Remove any sentence that does not answer the request, support a claim, state a needed limitation, or give a required next step.

## Evidence and citation economics
- Use compact evidence by default.
- Defer deep citation and long reference lists until requested.
- For recommendation decisions, prefer:
  - brief confidence label (`high`, `medium`, `low`), and
  - a short verification step.

## Vocabulary governance
- Use scoped vocabulary where it applies (for example Rust and wgpu domain terms).
- Use the approved term list in `.github/response-dictionary.md`.
- Propose new terms in the Pending Terms section.
- New terms are approved through pull request review before moving to Approved Terms.

## Cost guardrails
- Do not produce long background sections unless requested.
- Do not repeat unchanged plans.
- Keep follow-up suggestions to at most 2 items by default.
- Prefer compact checklists over long prose.