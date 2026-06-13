---
name: log-concept
description: Manually log a concept or insight the user figured out themselves during development. Use this when you have an aha moment you want to record.
---

The user wants to log something they figured out themselves. This is a self-discovered insight, not something Claude explained.

## Step 1 — Determine the concept

If `$ARGUMENTS` is provided, use it as the concept name and ask the user for a brief description of what they understood.

If `$ARGUMENTS` is empty, ask the user: "What concept or insight do you want to log?"

Wait for their response before proceeding.

## Step 2 — Write the entry

Append to `obsidian/Astra/Learning/concepts.md` using this format:

```
## <Concept Name>
**Date:** YYYY-MM-DD
**Context:** <one sentence — what they were building when they figured this out>
**Source:** Self-discovered

<Their explanation in their own words, cleaned up for clarity. Keep their voice — do not over-formalise it.>

---
```

Update the `updated` frontmatter date.

## Step 3 — Confirm

Tell the user the concept has been logged and ask if they want to discuss it further or if anything about their understanding needs refining.
