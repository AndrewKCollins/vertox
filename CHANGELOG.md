# Changelog

All notable changes to VERTOX will be documented here.

## [Unreleased]

### Added

- VERTOX project identity and monochrome logo.
- Public repository documentation, contribution guide, security policy, and community templates.
- CI, dependency-update automation, mdBook deployment, and tagged release builds.
- Global verbosity flags and clearer CLI help.
- Friendly command names: `scan`, `fetch`, and `ast`.

### Changed

- Renamed the executable from `sol-azy` to `vertox`.
- Command failures now propagate to the process exit status, making the CLI safer to use from shell scripts and CI.
- Build output uses `-o/--out-dir` consistently.

### Compatibility

- Legacy aliases `sast`, `fetcher`, and `ast-utils` remain available.

### Attribution

VERTOX is based on sol-azy by FuzzingLabs and contributors. See `NOTICE` and `LICENSE`.
