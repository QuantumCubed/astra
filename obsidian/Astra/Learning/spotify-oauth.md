---
created: 2026-06-27
updated: 2026-06-27
---

[[LEARNING|← Learning]]

# Spotify OAuth Flow

**Date:** 2026-06-27
**Context:** Setting up Spotify API access for the Astra voice assistant
**Source:** Self-discovered

You start the flow by visiting Spotify's authorization URL in a browser. Spotify redirects back to a local callback URL with a one-time `auth_code` in the query string. That code expires in 10 minutes and can only be used once — you exchange it immediately via a POST to Spotify's token endpoint to get back a `refresh_token` and an `access_token`.

The `access_token` is short-lived (1 hour). The `refresh_token` is long-lived (roughly 6 months as of 2026) and is stored in `.astra/astra.conf`. Whenever the access token expires or a 401 is returned by the API, you POST the refresh token to Spotify's token endpoint to mint a new access token — no user interaction needed. Spotify may also issue a new refresh token in that response, in which case the old one is invalidated and the new one must be saved.

The `auth_code` → token exchange is a one-time manual step. Everything after that is automated.

---
