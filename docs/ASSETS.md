# Assets

Assets are files in the project's `Assets` folder. For now they are pictures: `.png`, `.jpg`, `.jpeg`
(meshes, audio and the rest come with the asset pipeline, step 10).

## Fixed uuids: the meta file

The first time the editor sees an asset file it writes a small file next to it, `<file>.meta.yaml`:

```
Assets/Art/hero.png
Assets/Art/hero.png.meta.yaml
```

```yaml
format: asset_meta
version: 1
data:
  uuid: 6f1c0c52-0a7a-4a33-9d2f-7a8f0e3d9c11
  filter: Linear          # or Nearest (hard pixels, for pixel art)
```

- The **uuid never changes**: not when you edit the picture, not when you move or rename it together with its
  meta file. A sprite or a mesh refers to the picture by this uuid, so references survive a rename.
- Keep the meta file next to the picture (and in version control). Without it the picture gets a new uuid.
- If a picture is **copied together with its meta file**, two assets would share a uuid. The second one gets a
  new uuid, the new meta file is written, and the status bar reports it.
- A meta file that can't be read is reported and its picture is left out until it is fixed.
- All asset uuids are registered in the id registry, like every other id.

## In the editor

- **Project panel > Assets** lists the pictures. **Refresh** looks in the folder again (new pictures get their
  meta file). The drop-down next to a picture chooses **Smooth** or **Hard pixels** and is saved in the meta file.
- In the Inspector, a **Texture** field (on `MeshDraw`) and a **Sprite** field (on `SpriteDraw`) is a picker that
  lists the pictures of the project.
- `MeshDraw` also has **Tiling** (how often the picture repeats) and **Offset**.

## How pictures are drawn

The color of the component is multiplied with the picture, so a white color shows the picture as it is.
(0, 0) of the picture is the bottom left, (1, 1) the top right, and outside that range the picture repeats.
**Sprites** use the alpha of the picture (see-through parts show what is behind), **meshes** ignore it for now.
A picture that can't be read, or an asset uuid that is no longer in the project, is simply left out and the
object is drawn with its color only.

Pictures are read when the Scene or Game view needs them, and again when the file changes on disk, moves, or
its filter changes.
