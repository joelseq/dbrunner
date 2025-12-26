# Release Process

This document describes how to create a new release of DBRunner.

## Overview

DBRunner uses automated GitHub Actions to build desktop applications for macOS (Intel & Apple Silicon), Windows, and Linux. When you push a version tag, the workflow automatically builds all platform variants and creates a draft GitHub release.

## Prerequisites

- Write access to the repository
- Git installed locally
- Node.js and pnpm installed

## Step-by-Step Release Process

### 1. Bump the Version

Use the version bumping script to update the version across all configuration files:

```bash
pnpm version:bump <new-version>
```

**Example:**
```bash
pnpm version:bump 0.2.0
```

This script will:
- Update `package.json`
- Update `src-tauri/Cargo.toml`
- Update `src-tauri/tauri.conf.json`

Version format must be `X.Y.Z` (semantic versioning without the `v` prefix).

### 2. Review the Changes

Check that the version was updated correctly:

```bash
git diff
```

You should see version changes in:
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

### 3. Commit the Version Bump

Commit the version changes:

```bash
git add -A
git commit -m "Bump version to v0.2.0"
```

### 4. Create and Push the Tag

Create a git tag with the `v` prefix:

```bash
git tag v0.2.0
```

Push both the commit and the tag:

```bash
git push
git push --tags
```

**Important:** The tag **must** start with `v` and match the pattern `v*.*.*` (e.g., `v0.2.0`, `v1.0.0`, `v2.1.3`).

### 5. Monitor the Build

Once you push the tag, the GitHub Action will automatically start:

1. Go to your repository on GitHub
2. Click on the "Actions" tab
3. Find the "Release" workflow running for your tag
4. Monitor the progress of all platform builds

The workflow runs in parallel across:
- **macOS (Apple Silicon)**: Builds for M1/M2/M3 Macs
- **macOS (Intel)**: Builds for Intel Macs
- **Linux**: Builds AppImage, deb, and rpm packages
- **Windows**: Builds MSI installer

### 6. Publish the Release

After all builds complete successfully:

1. Go to the "Releases" page on GitHub
2. Find your draft release (it will be named "DB Runner v0.2.0")
3. Review the release notes and assets
4. Edit the release body to add:
   - What's new in this version
   - Bug fixes
   - Breaking changes (if any)
   - Known issues
5. Click "Publish release"

## Build Artifacts

The following artifacts are automatically built and attached to the release:

### macOS
- `DB Runner_<version>_aarch64.dmg` - Apple Silicon installer
- `DB Runner_<version>_x86_64.dmg` - Intel installer
- `.app.tar.gz` variants for both architectures

### Windows
- `DB Runner_<version>_x64-setup.exe` - Installer
- `DB Runner_<version>_x64_en-US.msi` - MSI package

### Linux
- `db-runner_<version>_amd64.AppImage` - Universal AppImage
- `db-runner_<version>_amd64.deb` - Debian/Ubuntu package
- `db-runner_<version>_amd64.rpm` - Fedora/RHEL package

## Troubleshooting

### Build Fails

If a build fails:
1. Check the GitHub Actions logs for error details
2. Fix the issue in your code
3. Delete the tag locally and remotely:
   ```bash
   git tag -d v0.2.0
   git push --delete origin v0.2.0
   ```
4. Make your fixes and commit them
5. Start the release process again from step 1

### Wrong Version Number

If you pushed the wrong version:
1. Delete the tag (see above)
2. Delete the draft release on GitHub
3. Run the version bump script again with the correct version
4. Commit and tag again

## Quick Reference

```bash
# Full release flow
pnpm version:bump 0.2.0
git diff                    # Review changes
git add -A
git commit -m "Bump version to v0.2.0"
git tag v0.2.0
git push && git push --tags

# Then monitor GitHub Actions and publish the draft release
```

## Version Numbering Guidelines

DBRunner follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0): Breaking changes, major features
- **MINOR** (0.X.0): New features, backward compatible
- **PATCH** (0.0.X): Bug fixes, backward compatible

Examples:
- `0.1.0` → `0.2.0`: Added Redis support (new feature)
- `0.2.0` → `0.2.1`: Fixed volume mount bug (bug fix)
- `0.9.0` → `1.0.0`: First stable release (major milestone)

## Notes

- The release is created as a **draft** by default, giving you a chance to review before publishing
- All builds must succeed for the release to be complete
- The workflow uses caching to speed up subsequent builds
- macOS builds require code signing for distribution outside the workflow (not configured yet)
