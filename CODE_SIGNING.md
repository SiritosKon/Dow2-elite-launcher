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

No analytics, no tracking, no accounts.
