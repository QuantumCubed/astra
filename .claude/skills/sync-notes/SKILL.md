---
name: sync-notes
description: Review and update the Obsidian vault notes to reflect the current state of the project at the end of a session
---

Review and update the Obsidian vault notes at obsidian/Astra/ to reflect the current state of the project.

Follow these steps:

1. Read all six notes: ASTRA.md, Architecture.md, Decisions.md, Todos.md, Progress.md, Roadmap.md

2. Review what happened in this session by looking at:
   - Any source files that were modified (check src/)
   - Any decisions or tradeoffs that were discussed
   - Any tasks that were completed or newly identified
   - Any Rust or architecture concepts that were explained in depth

3. Update each note where there are gaps or stale content:
   - Architecture.md: update if any modules, routes, or data flow changed
   - Decisions.md: add an entry for any significant decision made this session (use today's date and the format already in the file)
   - Todos.md: move completed items to Done, add newly discovered work to Backlog
   - Progress.md: add an entry for today if meaningful work happened (use the format already in the file)
   - ASTRA.md: update only if the project description or component list changed

4. Update the `updated` frontmatter date on any note you modify.

5. Log any new concepts to `obsidian/Astra/Learning/concepts.md`:
   - Read the file first and check what is already logged
   - For each Rust language concept, async/Tokio pattern, architectural pattern, or non-obvious standard library behaviour that was explained in meaningful depth this session and is NOT already in the file, append an entry using this format:

   ```
   ## <Concept Name>
   **Date:** YYYY-MM-DD
   **Context:** <one sentence — what we were building when this came up>
   **Source:** Claude

   <Plain-English explanation. 2-5 sentences. No code unless essential.>

   ---
   ```

   Update the `updated` frontmatter date on concepts.md if you add any entries.
   Do NOT log trivial syntax reminders or things already in the file.

6. Update README.md if any of the following changed this session:
   - Module structure or file paths
   - Configuration format or location (`.astra/`, `astra.conf`, etc.)
   - Message protocol
   - Setup steps or prerequisites
   - Roadmap phase status

   Keep README.md in sync with the actual codebase — stale paths and wrong status are worse than no README.

7. Report what you changed and what you left alone, in one short paragraph.

Do not add entries for trivial changes (typo fixes, minor refactors). Only record things that a future developer — or future Claude — would need to know.
