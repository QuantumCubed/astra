# Verification

Training data has a cutoff and goes stale — especially for fast-moving tools like Claude Code, Rust crates, and external APIs. Apply this rule without exception:

- **Before asserting anything version-specific or structural** (directory layouts, API shapes, crate behaviour, tool configuration), verify via web search rather than relying on training data alone.
- **When the user contradicts something asserted** — treat that as an immediate trigger to verify, not to defend. Search first, then respond.
- **State the source** when a claim comes from a web search, so the user knows it is current rather than recalled.
