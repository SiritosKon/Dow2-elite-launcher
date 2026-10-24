# Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org).

Only the Elite Launcher itself is signed: the Windows installer (`EliteLauncher-<version>-setup.exe`) and the program
inside it (`elite-launcher.exe`, also shipped in the portable zip). They are built by GitHub Actions from the tagged
source in this repository; nothing built on a developer's machine is signed. Elite Mod game files and the optional
add-ons the launcher can install are not part of this project and are not signed with this certificate.

## Team roles

| Role | Members |
|---|---|
| Committers and reviewers | [pivolan](https://github.com/pivolan) |
| Approvers (approve each signing request) | [pivolan](https://github.com/pivolan) |

All team members use multi-factor authentication for GitHub and SignPath.

## Privacy policy

This program does not collect or send personal data on its own. It connects only to:

- the update mirrors listed in its configuration, to download the signed configuration, news and game files;
- the address shown on the *Send logs* button, and only when the user presses it: the log files of the game and of the
  installed add-ons are uploaded so a developer can investigate a crash. The user can type a name to attach to the
  upload; otherwise it is anonymous.

- the same address once a day for an anonymous usage count, unless the user unticks it in Settings. The request
  carries a hash that changes every day (SHA-256 of a random local secret and the day number, cut to 128 bits), the
  launcher version, the operating system and the installed Elite version. The secret never leaves the computer, and
  because the hash changes daily the server cannot link two days to the same install. No names, Steam, hardware or
  account data. The server keeps only these fields per day and does not log IP addresses for this request.

No tracking, no accounts.
