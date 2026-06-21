---
name: rust-review
description: Review Rust code for security issues, conventions, and correctness. Invoke with a file path, directory, or no argument for the entire project. Writes a dated report to the Obsidian vault.
---

You are performing a Rust code review. The user is actively learning Rust, so every finding must include a clear explanation of WHY it is a problem — not just what it is. Do not fix anything. Report only.

## Step 1 — Determine scope

- If `$ARGUMENTS` is a file path: review that file only.
- If `$ARGUMENTS` is a directory: review all `.rs` files in that directory.
- If `$ARGUMENTS` is empty: review all `.rs` files under `src/`.

Read every file in scope before proceeding.

## Step 2 — Review each file against these categories

### Security
- `unwrap()` or `expect()` calls outside of tests — these panic on failure in production; flag each one and explain what should be used instead
- `unsafe` blocks — flag and explain why they require extra justification
- `std::process::Command` calls that incorporate any non-literal arguments — potential command injection
- Hardcoded sensitive values (tokens, passwords, URLs that look like credentials)

### Rust Conventions
- Naming: functions and variables must be snake_case, types PascalCase, constants SCREAMING_SNAKE_CASE
- Visibility: flag `pub` on anything that does not need to be public
- Idiomatic error handling: prefer `?` over `unwrap()`/`expect()` in functions that return `Result`
- Unnecessary `.clone()` calls where a reference would suffice
- Unused imports or dead code

### Async Correctness
- Blocking calls inside async functions (e.g. `std::thread::sleep`, synchronous file I/O, `std::process::Command` in an async context without `spawn_blocking`)
- Functions marked `async` that contain no `.await` expressions

### Structure
- Functions that are doing more than one clear thing (flag, do not refactor)
- Missing error propagation — errors being silently discarded
- Inconsistent patterns compared to how similar things are handled elsewhere in the codebase

## Step 3 — Write the report to Obsidian

Create a new note at `obsidian/Astra/Reviews/YYYY-MM-DD-<scope>.md` where `<scope>` is the file name, directory name, or "project". Use today's date.

After creating the note, add a one-line entry to `obsidian/Astra/Reviews/REVIEWS.md` under the index (replacing `_No reviews yet._` if it is the first entry):
```
- [[YYYY-MM-DD-<scope>]] — <scope>, <N> findings
```
Update the `updated` frontmatter date on `REVIEWS.md`.

Use this exact format:

```
---
created: YYYY-MM-DD
updated: YYYY-MM-DD
scope: <file, directory, or project>
---

# Code Review — <scope> — YYYY-MM-DD

[[REVIEWS|← Reviews]]

## Summary

<2-3 sentences: overall impression, number of findings, most critical concern>

## Findings

### Critical
<Issues that could cause panics, security vulnerabilities, or data loss. If none: "None.">

### Warnings
<Incorrect Rust conventions, async mistakes, or patterns that will cause problems as the codebase grows. If none: "None.">

### Suggestions
<Style, clarity, and idiomatic improvements that are worth making but not urgent. If none: "None.">

## Discussion
<Leave blank — filled in during follow-up conversation>
```

Each finding must follow this format:
- **File**: `src/path/to/file.rs` line N
- **Issue**: what the problem is
- **Why**: explanation suitable for someone learning Rust
- **Direction**: a hint toward the fix without writing the code

## Step 4 — Report back in conversation

After writing the note, give the user a short summary: scope reviewed, total findings by severity, and the path to the Obsidian note. Then ask if they want to discuss any specific finding.
