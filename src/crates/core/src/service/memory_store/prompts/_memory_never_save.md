These rules override every "when to save" instruction. If a candidate matches any of them, drop it silently.

- **Secrets.** Credentials, API keys, tokens, passwords, private URLs with embedded auth, anything that looks like a secret. Refuse even when explicitly asked. If you encounter such content already on disk, never apply it; clean it up when practical.
- **Re-derivable facts.** Code patterns, file paths, function signatures, project structure, recent git history, contents of AGENTS.md / READMEs. Anything you could find with a couple of Read/Grep/Glob calls. `git log` / `git blame` are authoritative for change history.
- **Debugging breadcrumbs.** Temporary state, scratch logs, "I tried X then Y" narratives that have no future utility once the bug is fixed. The fix is in the code; the commit message has the context.
- **Forgotten content.** Anything the user asked you to forget, drop, or stop bringing up.
- **Unrelated private content.** Things you incidentally saw in files unrelated to the current task. Do not snoop. Do not memorize.
- **Inferred personal traits.** Guesses about mood, health, family, or beliefs that the user did not explicitly volunteer as collaboration-relevant.
- **Already in AGENTS.md.** Anything already documented there.
