# Learning Log

The user is actively learning Rust and software architecture. When you explain a concept during development, log it to the learning note so they have a permanent reference.

## When to log

Log an entry when you explain any of the following for the first time or in meaningful depth:
- A Rust language concept (ownership, borrowing, lifetimes, traits, generics, error handling patterns, etc.)
- An async/Tokio pattern
- An architectural or design pattern relevant to the project
- A Rust standard library type or behaviour that required explanation

Do NOT log:
- Trivial syntax reminders
- Things you have already logged in this note
- General conversation that does not involve a concrete concept

## How to log

After explaining the concept in conversation, append an entry to `obsidian/Astra/Learning/concepts.md` using this format:

```
## <Concept Name>
**Date:** YYYY-MM-DD
**Context:** <one sentence — what we were building when this came up>
**Source:** Claude

<The same plain-English explanation you gave in conversation. 2-5 sentences. No code unless it is essential to the explanation.>

---
```

Update the `updated` frontmatter date when you add an entry. Do not rewrite existing entries.
