# Changelog

Every version gets an entry here before it reaches `main`. CI then tags `vX.Y.Z` on its own; for a minor or major
version (`X.Y.0`) it also publishes a GitHub Release whose notes are the entries since the previous `X.Y.0`.
The version itself lives only in `Cargo.toml` (`[workspace.package] version`).

## 1.2.8 — 2026-10-26
- An Elite install the launcher did not make is recognised and not downloaded again.
- Progress is shown while big files are checked.
- Feedback chat with the developers (text and pictures) instead of the news panel.

## 1.2.7 — 2026-10-25
- The lobby host filter is gone: no auto-kick settings; old filter rules are cleaned out of `LobbyStats.ini`.
- Add-ons update on their own when their version changes.

## 1.2.6 — 2026-10-25
- Anonymous daily usage count: the hash changes every day, can be turned off in the settings.

## 1.2.5 — 2026-10-24
- Add-ons step aside for a hand-installed `dow2-patches` copy and ask before deleting it.

## 1.2.4 — 2026-10-24
- Finds the game in every Steam library; install, update and repair of Elite with chunked downloads, Ed25519-signed
  manifests and several mirrors; stable / beta / archive channels; add-ons; news, announcements and links from
  `launcher.json`; "Send logs".
