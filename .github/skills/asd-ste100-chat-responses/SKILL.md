---
name: asd-ste100-chat-responses
description: 'Write or review chat responses in ASD-STE100 Simplified Technical English Issue 9. Use for answers, explanations, recommendations, summaries, procedures, and user-facing messages.'
---

# ASD-STE100 Chat Responses

Use this skill for every chat response about Anzu. Give the user a direct, useful answer and follow ASD-STE100 Issue 9 where its rules apply.

## Sources

- Use ASD-STE100 Issue 9, Part 1, for its writing rules and approved common-word list. The local copy is `documents/ASD-STE100_ISSUE9.pdf`.
- Use `.github/response-dictionary.md` for Anzu-specific and domain-specific terms.
- Do not copy the standard's general dictionary into the project dictionary or this skill.
- Keep code, API, product, file, protocol, and quoted names exact.

## Response Method

1. Identify the user's question and answer it first.
2. Keep the response as short as the task allows. Use the universal structured-response policy for scope and format.
3. Check `.github/response-dictionary.md` for approved and ambiguous terms.
4. If a new project term is unclear, ask one concise question about its meaning or scope before using it as established vocabulary. Do not ask about common English words or exact code/API names.
5. After the user clarifies a new term, add it to `Pending Terms` with its scope and meaning. If the user already gives a clear definition, add it without asking a repeat question.
6. Use approved terms for established concepts. Do not invent synonyms.
7. Check common English against the Issue 9 word list when strict compliance matters. Use each approved word only with its approved meaning, part of speech, and form.
8. Review the response for sentence length, voice, grammar, punctuation, and preserved technical names.
9. If a term or rule is uncertain, retain exact technical wording and state the uncertainty. Do not claim full compliance without a complete check.

## New Vocabulary

- Ask for clarification when a new project-specific term has no clear meaning or scope in context.
- Ask only for the missing information. For example: “What does ‘zone owner’ mean here, and is it a runtime or content term?”
- After clarification, add the term and agreed meaning to `.github/response-dictionary.md` under `Pending Terms`. Add a scope and note its source where the table supports them.
- Do not move a term to `Approved Terms` until pull request review approves it.
- If a term has multiple meanings, record it in `Ambiguous Terms` and ask which scoped meaning applies before using it.
- Keep the conversation moving. Use a clear existing term or a short neutral description while waiting for clarification.

## Writing Checks

- Prefer short, direct sentences and active voice. Use passive voice only when it gives a clearer result.
- Keep descriptive sentences to 25 words or fewer. Keep procedural sentences to 20 words or fewer.
- Give one instruction in each procedural sentence unless the actions must occur together. Start instructions with an imperative verb.
- Keep each paragraph to one topic and no more than six sentences. Use a vertical list for complex information.
- Do not use contractions or omit articles to shorten a sentence.
- Keep multi-word noun groups to three words when possible. Keep a complete technical name when shortening it would change its meaning.
- Use American English spelling unless an official name or project requirement specifies another form.
- Do not use semicolons.
- Use punctuation to show clear relationships between ideas. Do not make noun or verb choices solely to meet a word-count limit.
- Preserve code blocks, commands, identifiers, and quoted user text. Apply the writing checks to surrounding prose, not to machine-readable or exact text.

## Compliance

This skill is a response checklist, not a replacement for Issue 9. For a request that requires strict compliance, review the full response against the applicable rules and word-list entries. Report any technical terms that cannot be checked or changed without altering their meaning.