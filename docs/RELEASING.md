# Publishing a release

The launcher installs whatever the GitHub **Releases** of the repository offer.

## Steps

1. Set the version in the root `Cargo.toml` (`[workspace.package] version`). It is 1.0.0 now and
   only changes when you decide to publish a new release. Add the changelog entry in `changelog/`.
2. Tag and push:
   ```
   git tag v1.0.0
   git push origin v1.0.0
   ```
   For a beta use a tag like `v1.1.0-beta.1`, it is marked as a pre-release automatically.
3. `.github/workflows/release.yml` builds the Windows editor and attaches the files below.

## What the launcher expects in a release

| File                                         | Required | Notes                                        |
| -------------------------------------------- | -------- | -------------------------------------------- |
| `flux-aphelion-<version>-windows-x64.zip`    | yes      | the name must contain `windows` and end in `.zip` |
| `flux-aphelion-<version>-windows-x64.zip.sha256` | no   | `<hash>  <file name>`, checked when present  |

The zip must contain `flux-editor.exe`, either at the top or inside one single folder.
Tags that are not versions (like `nightly`) and draft releases are ignored.
