# documentation-review

A skill for writing and reviewing Anzu documentation for quality, consistency, completeness, and ASD-STE100 Issue 9 alignment.

Policy bindings
- Apply `.github/instructions/universal-structured-response.instructions.md`.
- Use scoped terms from `.github/response-dictionary.md`.
- Use `.github/skills/asd-ste100-documentation/SKILL.md` when writing or editing documentation.
- Ask narrowing questions first when scope is broad.
- Default to short mode and compact evidence.

Focus
- Verify that docs cover user workflows, APIs, architecture, developer onboarding, deployment, and operations.
- Identify missing sections, unclear wording, and drift between implementation and docs.
- Check that common English follows the ASD-STE100 Issue 9 writing rules. Do not copy or recreate the standard's general dictionary.
- Check that Anzu-specific technical nouns match the approved project vocabulary and use one term for one concept.
- Check that procedures use clear imperative steps, one instruction per sentence, and no more than 20 words per procedural sentence.
- Check that descriptive sentences are clear and no more than 25 words. Use active voice unless the passive voice is clearer or the actor is unknown.
- Check that code identifiers, API names, file names, and protocol labels remain exact. Do not rewrite these to fit general-word rules.

Output
- Report issues with a short reason and a proposed correction. Separate confirmed violations from style suggestions.
- Do not claim full STE100 compliance unless a complete, appropriate review has been done.
