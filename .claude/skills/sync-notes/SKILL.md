---
name: sync-notes
description: Review and update the Obsidian vault notes to reflect the current state of the project at the end of a session
---

Review and update the Obsidian vault notes at obsidian/Astra/ to reflect the current state of the project.

Follow these steps:

1. Read all five notes: ASTRA.md, Architecture.md, Decisions.md, Todos.md, Progress.md

2. Review what happened in this session by looking at:
   - Any source files that were modified (check src/)
   - Any decisions or tradeoffs that were discussed
   - Any tasks that were completed or newly identified

3. Update each note where there are gaps or stale content:
   - Architecture.md: update if any modules, routes, or data flow changed
   - Decisions.md: add an entry for any significant decision made this session (use today's date and the format already in the file)
   - Todos.md: move completed items to Done, add newly discovered work to Backlog
   - Progress.md: add an entry for today if meaningful work happened (use the format already in the file)
   - ASTRA.md: update only if the project description or component list changed

4. Update the `updated` frontmatter date on any note you modify.

5. Report what you changed and what you left alone, in one short paragraph.

Do not add entries for trivial changes (typo fixes, minor refactors). Only record things that a future developer — or future Claude — would need to know.
