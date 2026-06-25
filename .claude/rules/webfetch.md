# Web Search and Fetch Reliability

WebFetch processes pages through a summarizing model that can hallucinate details — especially exact method signatures, parameter names, and argument counts. Web searches can surface outdated or incorrect documentation. Apply these rules without exception:

## For exact API shapes (method signatures, parameter types, return types)

- **Prefer raw source files over READMEs and docs pages.** Fetch `github.com/<owner>/<repo>/blob/main/src/lib.rs` (or equivalent) rather than the README or a docs.rs overview page. Source code cannot be hallucinated.
- **Treat any signature from a WebFetch summary as a hypothesis, not a fact.** When writing code from a WebFetch-derived signature, flag it as unverified until the compiler confirms it.
- **The compiler is always right.** When a compile error contradicts a claimed API shape, trust the error — do not re-fetch the docs to "verify." Fix the code from the error message directly.

## For crate dependency compatibility

- **Check `Cargo.lock` before researching version pins.** The resolved version is in the lock file. Do not guess what version Cargo resolved — read it.
- **When a crate fails to compile, the error is in that crate's source** — read the actual file path in the error (e.g. `~/.cargo/registry/src/.../model.rs:292`) to understand the real API mismatch before deciding on a fix.

## For library selection

- **Do not re-investigate ruled-out libraries without a concrete new reason.** If a library was rejected and the reason is documented (in memory or in conversation), treat that as settled. Re-researching it wastes time unless the user raises a specific new signal (e.g. a new release, a fixed issue).
