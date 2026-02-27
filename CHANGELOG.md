# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed (Unreleased)

- No unreleased changes yet.

## [2.4.0] - 2026-02-27

### Added (2.4.0)

- Added community health templates: bug report and feature request issue forms.
- Added a pull request template with required validation checklist.
- Added a contributor guide (`CONTRIBUTING.md`) with setup, workflow, and submission steps.
- Added pre-commit configuration with Rust and Python quality hooks.
- Added package typing marker distribution support via `py.typed` inclusion.

### Changed (2.4.0)

- Overhauled project metadata in `pyproject.toml` (description, authorship, classifiers, URLs, optional dev dependencies, dynamic versioning, and wheel include rules).
- Reworked GitHub Actions CI/CD for a clearer multi-stage flow (test gating, wheel matrix builds, sdist, and trusted-publishing release step).
- Modernized `.gitignore` to align Rust/Python build artifacts and common local tooling outputs.
- Rewrote README into a complete open-source landing page with features, quick start, development, contribution, license, and support sections.

### Removed (2.4.0)

- Removed legacy and redundant CI workflow logic (nightly bootstrap/update steps, mixed upload strategy, and manual token-based publish path).
- Removed outdated metadata and wording from project definition files and documentation.

## [2.3.6] - 2025-10-08

### Changed (2.3.6)

- Updated dependencies and internal linting fixes.

## [2.3.5] - 2025-06-11

### Fixed (2.3.5)

- Corrected the `indent` argument handling in `write_string`.

## [2.3.4] - 2025-06-09

### Changed (2.3.4)

- Updated Maturin build CI configuration.

## [2.3.3] - 2025-06-09

### Changed (2.3.3)

- Version bump release.

## [2.3.2] - 2025-03-05

### Changed (2.3.2)

- Performance branch merged with parser/read-path speed improvements.

## [2.3.1] - 2025-03-05

### Changed (2.3.1)

- Additional speed-focused refactors merged.

## [2.3.0] - 2025-03-03

### Added (2.3.0)

- Added `Node` dictionary conversion helpers (`from_dict`/`to_dict`) and related typing updates.

### Changed (2.3.0)

- Updated CI/actions and modernized dependency/tooling configuration.

## [2.2.0] - 2024-12-18

### Changed (2.2.0)

- Updated project packages and Maturin configuration.

## [2.1.2] - 2024-12-18

### Fixed (2.1.2)

- Fixed `.pyi` search function typing and `SearchType` definition.

## [2.1.1] - 2024-11-19

### Added (2.1.1)

- Improved node reader behavior for empty-tag XML elements.

## [2.1.0] - 2024-10-30

### Changed (2.1.0)

- Updated core libraries to newer versions.

## [2.0.0] - 2024-07-12

### Fixed (2.0.0)

- Stabilized tests by creating/removing files only during test execution.

## [1.1.0] - 2024-04-08

### Changed (1.1.0)

- Updated PyO3 to a newer version.

## [1.0.1] - 2023-12-15

### Changed (1.0.1)

- Version bump release.

## [1.0.0] - 2023-11-24

### Fixed (1.0.0)

- Rolled back minimum supported Python version.

## [0.4.0] - 2023-05-01

### Fixed (0.4.0)

- Allowed constructor text values to be null/empty safely.

## [0.2.0] - 2023-04-26

### Added (0.2.0)

- Expanded installation/help documentation and project description.

## [0.1.0] - 2023-04-26

### Changed (0.1.0)

- Early refactor and initial rename to `rxml`.

[Unreleased]: https://github.com/nephi-dev/rxml/compare/2.4.0...HEAD
[2.4.0]: https://github.com/nephi-dev/rxml/compare/2.3.6...2.4.0
[2.3.6]: https://github.com/nephi-dev/rxml/compare/2.3.5...2.3.6
[2.3.5]: https://github.com/nephi-dev/rxml/compare/2.3.4...2.3.5
[2.3.4]: https://github.com/nephi-dev/rxml/compare/2.3.3...2.3.4
[2.3.3]: https://github.com/nephi-dev/rxml/compare/2.3.2...2.3.3
[2.3.2]: https://github.com/nephi-dev/rxml/compare/2.3.1...2.3.2
[2.3.1]: https://github.com/nephi-dev/rxml/compare/2.3.0...2.3.1
[2.3.0]: https://github.com/nephi-dev/rxml/compare/2.2.0...2.3.0
[2.2.0]: https://github.com/nephi-dev/rxml/compare/2.1.2...2.2.0
[2.1.2]: https://github.com/nephi-dev/rxml/compare/2.1.1...2.1.2
[2.1.1]: https://github.com/nephi-dev/rxml/compare/2.1.0...2.1.1
[2.1.0]: https://github.com/nephi-dev/rxml/compare/2.0.0...2.1.0
[2.0.0]: https://github.com/nephi-dev/rxml/compare/1.1.0...2.0.0
[1.1.0]: https://github.com/nephi-dev/rxml/compare/1.0.1...1.1.0
[1.0.1]: https://github.com/nephi-dev/rxml/compare/1.0.0...1.0.1
[1.0.0]: https://github.com/nephi-dev/rxml/compare/0.4.0...1.0.0
[0.4.0]: https://github.com/nephi-dev/rxml/compare/0.2.0...0.4.0
[0.2.0]: https://github.com/nephi-dev/rxml/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/nephi-dev/rxml/releases/tag/0.1.0
