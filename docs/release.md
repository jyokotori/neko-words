# Release: Neko Words

This project releases through GitHub tags. The release workflow builds prebuilt
packages, creates the GitHub Release, and updates the Homebrew tap automatically.

## Small Version Release

1. Update the workspace version in `Cargo.toml`.
2. Commit the release changes.

```bash
git add -A
git commit -m "Prepare v0.1.2 release"
```

3. Create an annotated tag matching the version.

```bash
git tag -a v0.1.2 -m "v0.1.2"
```

4. Push `main` and the tag.

```bash
git push origin main
git push origin v0.1.2
```

5. Wait for the `Release` GitHub Actions workflow to finish.

The workflow should:

- build `aarch64-apple-darwin`;
- build `x86_64-unknown-linux-gnu`;
- create the GitHub Release assets;
- update `jyokotori/homebrew-tap`.

The tap update requires the repository secret `HOMEBREW_TAP_TOKEN`.

## Verify Homebrew

After the workflow succeeds:

```bash
brew update
brew info jyokotori/tap/neko-words
brew upgrade neko-words
```

If the package is not installed yet:

```bash
brew install jyokotori/tap/neko-words
```

Do not manually update the Homebrew tap for a normal release. If the workflow
fails after the GitHub Release is created, inspect the workflow logs first and
only patch the tap manually when the automatic update cannot be rerun cleanly.
