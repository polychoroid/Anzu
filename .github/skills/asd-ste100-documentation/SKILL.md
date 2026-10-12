# ASD-STE100 Documentation

Use this skill when you write, edit, or review documentation for Anzu.

## Purpose

Write clear Anzu documentation that follows ASD-STE100 Issue 9 where the standard applies. Use Anzu's approved vocabulary for project concepts. Do not reproduce the standard's general dictionary.

## Sources

- Use `.github/response-dictionary.md` for approved project and domain terms.
- Use `documents/architecture.md` for the current architecture glossary and system boundaries.
- Use `documents/` and the implementation as evidence for product behavior. Do not infer behavior from terminology alone.
- Use ASD-STE100 Issue 9, Part 1, for the full writing rules and the current approved word list. This skill is a practical checklist, not a replacement for the standard.

## Workflow

1. Identify the reader, the task, and the information the reader needs.
2. Confirm technical facts against the relevant source document or implementation.
3. Use the approved Anzu term for each project concept. Keep one term for one concept.
4. Write the content with the checks below.
5. Review the complete change for meaning, terminology, rule exceptions, and consistency with nearby documents.

## Writing Checks

### Words and Terms

- Use the ASD-STE100 Issue 9 word list for common English words. Check the required part of speech, meaning, and permitted form.
- Do not add the standard's common-word entries to `.github/response-dictionary.md`.
- Use project-specific technical nouns only when they name real Anzu concepts. Use the approved term from `.github/response-dictionary.md`.
- Do not use a technical noun as a verb. Use a clear approved verb to describe the action.
- Do not introduce a synonym for an existing Anzu concept. If a new term is needed, add it to Pending Terms for review.
- Use American English spelling unless an official product, code, or external name requires another form.

### Noun Groups

- Keep a multi-word noun to three words when possible.
- Keep the full approved technical name when shortening it would change or obscure its meaning. Explain a long name when the reader needs help with it.
- Do not create dense chains of nouns. Rewrite them as a short phrase or a sentence when this makes the relationship clear.

### Sentences and Procedures

- Write short, direct sentences. Do not remove articles or use contractions to make a sentence shorter.
- Use a vertical list when a sentence contains several related items or actions.
- In a procedure, use no more than 20 words per sentence.
- Give one instruction in each procedural sentence, unless the actions must occur together.
- Start an instruction with an imperative verb. Put a required condition before the instruction.
- Use notes for information, not for instructions.
- For descriptive text, use no more than 25 words per sentence. Prefer active voice. Use passive voice only when it is clearer, such as when the actor is unknown or unimportant.

### Code and Exact Names

- Preserve code identifiers, API names, file paths, command names, protocol labels, and literal user-interface text exactly.
- Do not alter code examples or machine-readable values to satisfy common-word rules.
- Define an acronym on first use when the intended reader may not know it. Keep established names such as `wgpu`, `WGSL`, and `ROM` when they identify actual project interfaces or concepts.
- Explain an unfamiliar technical term at first use when the document depends on it.

### Safety, Punctuation, and Structure

- Keep warnings, cautions, and notes distinct. State the hazard and required action in the correct safety format; consult Issue 9 for safety-instruction rules.
- Use punctuation and capitalization consistently. Preserve exact capitalization in proper names and code identifiers.
- Organize information in the order the reader needs it. Use headings and lists to expose procedures, conditions, and alternatives.
- Do not use a word count alone as proof of compliance. Check that each sentence is complete and has the intended meaning.

## Review Output

- Separate confirmed rule violations from optional style improvements.
- For each issue, quote only the short text needed to locate it, identify the relevant rule area, and suggest a correction.
- Preserve technical meaning. If a term or rule application is uncertain, state the uncertainty and ask for domain review instead of silently changing it.
- Do not claim full ASD-STE100 compliance unless the complete applicable content has been checked against Issue 9.