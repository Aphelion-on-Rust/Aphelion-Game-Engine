# Launcher

A separate app from the editor. Run it with `cargo run -p flux-launcher`.
It depends only on `flux-core` (not on the engine or the editor).

## What it does

1. **Versions tab**: reads the releases of a GitHub repository, shows them with size and date.
   Press Install and it downloads the file for your platform, checks the SHA-256, unpacks it and
   registers the version. Launch starts that version's editor. Set default, Uninstall (asks first).
   A banner says when a newer version than any installed one exists.
2. **Projects tab**: a list of projects, each with its own engine version (a drop-down of every
   known version; changing it also rewrites the project's own `project.yaml`).
   **Create a new project**: name, folder, engine version, graphics pipeline (SGP or HGP) and
   Aphelion's Control. Those two choices are final. **Add an existing project** reads the
   folder's `project.yaml`. The layout is described in `docs/PROJECT_FORMAT.md`. **Open** starts the editor with `--project <folder>`. If the version is not
   installed, the button says **Install & Open**: it installs, then opens the editor by itself.
3. **Settings tab**: GitHub owner and repository, install folder, show pre-releases,
   look for updates on start.

The release list is cached (`releases.yaml`), so the launcher still shows versions offline.

## Where things are

| Path                                          | What                                      |
| --------------------------------------------- | ----------------------------------------- |
| `%APPDATA%\FluxAphelion\launcher\*.yaml`      | settings, releases, installs, projects    |
| `<install folder>\versions\<version>\`        | one unpacked engine version               |
| `<install folder>\downloads\`                 | the zip while downloading (deleted after) |

Default install folder: `%LOCALAPPDATA%\FluxAphelion`.

## Safety

- A download goes to `<name>.part` and is renamed when complete.
- The zip is checked against the `.sha256` file when the release has one.
- Zip entries that try to leave the folder (`../x`) are refused.
- A version is unpacked into `<version>.installing` and moved into place only when it contains
  the editor, so a broken download never looks like an installed version.
- Uninstall only deletes folders inside `<install folder>\versions`.

## How it is built

```
launcher/src
  settings releases installs projects jobs   MIVC folders: pure logic, no network, no disk
  services                                   network, zip, disk, starting programs (the only impure code)
  ui                                         window; views push commands, controllers answer with Effects
```

A command such as Install goes: view pushes `LauncherCommand::Install` -> `apply_all` checks it
and starts a job -> returns `Effect::Install` -> the app runs `services::install` on a thread ->
the thread sends `InstallEvent`s back -> the controllers update the models. Everything except the
last network step is covered by tests.

## Before it works for you

- Publish a release the way `docs/RELEASING.md` says.
- The default repository is `atropinex/flux-aphelion`. That is a guess, change it in Settings
  (or in `settings.yaml`) if the repository is called something else.
- JSON appears in exactly one place: `releases/parse.rs`, because GitHub's answer is JSON.
  Everything the launcher saves is YAML.
