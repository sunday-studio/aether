# Agent Instructions

## Naming Rules

- New files and folders must use lowercase kebab-case.
- Keep names descriptive and short enough to scan.
- Follow existing generated-file names when editing generated outputs.
- Do not rename existing files or folders just for style unless the task asks for it.

## Commit Message Format

Use this format for commits:

```txt
conventional-commit-type(service/package changed): one liner

- key point if any exists;
- another key point if any exists;
```

- The first line must use a conventional commit type and a scope for the service or package changed.
- Use a concise one-line summary, for example `docs(repo): clarify agent instructions`.
- Add bullet points only when the one-liner does not cover the change.
- When bullets are needed, add one blank line after the subject.
- Bullet points must start with `- ` followed by text, with one space after the dash.
- End each bullet with `;`.
- Do not put blank lines between bullet points.
