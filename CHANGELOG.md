# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2](https://github.com/0809android/mdja/compare/v0.1.1...v0.1.2) - 2026-05-09

### Added

- Add typed frontmatter metadata, HTML TOC, tree TOC, parse options, CLI output modes, and Python/WASM bindings.

### Fixed

- Preserve malformed frontmatter as content, handle empty frontmatter, and align Markdown TOC indentation with the TOC tree.

## [0.1.1](https://github.com/0809android/mdja/compare/v0.1.0...v0.1.1) - 2026-05-09

### Other

- Fix heading metadata edge cases
- Verify crates before release publishing
- Automate crate releases with release-plz
- Fix heading anchors and stdin handling
- Add CI and trusted publishing workflow
