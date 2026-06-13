# Obsidian Vault Maintenance

The Obsidian vault at `obsidian/Astra/` is the project's source of truth for decisions, progress, architecture, and todos. Keep it current without being asked.

## Decisions.md

Update immediately when a significant decision is finalised in conversation — do not wait for `/sync-notes`. A decision is significant if it affects architecture, tech stack, project direction, or establishes a pattern others will follow.

Format:
```
## YYYY-MM-DD — <Decision Title>

**Decision:** <what was decided>

**Reasoning:** <why — include tradeoffs considered>

---
```

## Todos.md

- Move an item to Done when it is completed during the session.
- Add to Backlog when a new piece of work is identified.
- Do not add trivial or obvious sub-tasks.

## Progress.md

Add a dated entry at the end of a session if meaningful work happened — new features, significant refactors, architectural changes. Do not add entries for configuration tweaks or note updates alone.

## Architecture.md

Update when the system structure changes — new modules, new routes, changed data flow. Keep it in sync with the actual codebase.

## Subdirectory Structure and Linking

Each subdirectory has a root index file (e.g. `Learning/LEARNING.md`, `Reviews/REVIEWS.md`). The pattern is:

- `ASTRA.md` links to the subdirectory root (`[[LEARNING]]`, `[[REVIEWS]]`) — one line per subdirectory, no detail
- The subdirectory root links back to `[[ASTRA|← Home]]` and indexes its own children
- Individual notes inside a subdirectory link back to their subdirectory root, not to `ASTRA.md`

This keeps `ASTRA.md` stable — it only changes when a new subdirectory is added, not when individual notes are created.

When creating a note inside a subdirectory:
- Backlink to the subdirectory root (e.g. `[[REVIEWS|← Reviews]]`), not to `[[ASTRA|← Home]]`
- Add a one-line entry to the subdirectory root index

## General

- Always update the `updated` frontmatter date on any note you modify.
- Do not add noise. A vault that gets updated on every minor exchange is harder to read than one updated only when something meaningful happens.
