# Releasing

This page is for maintainers. If you're contributing, you don't need any of this.

## How it works

| Branch | For |
|---|---|
| `dev` | Your own work in progress, messy is fine |
| `main` | Finished work |

Pushing to `dev` or `main` never reaches users. Installed copies of XPicker only get what you **publish** as a release.

A release only starts when the version in `package.json` changes on `main`. Anything else you push to `main` is safe.

Never make a version tag by hand. Publishing a release creates it.

`dev` is public, so don't commit anything private to it.

## Testing

Test on your own PC, no need to push anything for it:

```sh
npm run dev      # run it
npm run build    # make the installer, see Setup
```

## Working on dev

```sh
git switch dev
git add -A
git commit -m "test thing"
git push origin dev
```

When it works, move it to `main` as one clean commit:

```sh
git switch main
git merge --squash dev
git commit -m "feat: add the thing"
git push origin main
```

See [Committing](/docs/contents/02_COMMITING.md) for how to write the message, because it appears in the release notes.

After that, `dev` is behind `main`. Bring it along:

```sh
git switch dev
git fetch origin
git reset --hard origin/main
git push --force origin dev
```

## Pull requests

To try one on your PC, using its number:

```sh
git fetch origin pull/123/head:pr-123
git switch pr-123
npm install
npm run dev
```

To merge it, use **Squash and merge** on GitHub, so it lands as one clean commit. Then:

```sh
git switch main
git pull origin main
```

## Publishing a release

1. Set the new version, like `1.1.0`, with no `v`:

   ```sh
   git switch main
   npm version 1.1.0 --no-git-tag-version
   ```

   This changes `package.json` and `package-lock.json`. The app, the installer and the updates all read the version from `package.json`. The version in `src-tauri/Cargo.toml` stays as it is, nothing uses it.

2. Commit and push it:

   ```sh
   git add package.json package-lock.json
   git commit -m "chore: release 1.1.0"
   git push origin main
   ```

3. On GitHub, open **Releases**. A draft called `v1.1.0` shows up with a downloads table, the commits sorted under headings, and the contributors. Windows, macOS and Linux get built into it, which takes around 15 minutes. Users get nothing yet.
4. Edit the notes. Remove anything that doesn't matter to users.
5. Click **Publish release**. This creates the tag, and installed copies now find the update.

Then check it: install the previous version, open **Settings → Updates**, click **Check for updates**, and make sure it updates and shows the new number.

## If the workflow fails

No draft is made and nothing reaches users. **Actions** shows what failed. Push the fix to `main`, then open **Actions**, then **Release**, and click **Run workflow** to try again with the same number.

## The updater key

Updates are signed, and the app refuses any update that isn't. The private key is the `TAURI_SIGNING_PRIVATE_KEY` secret in the repository settings; the public key is in `src-tauri/tauri.conf.json`.

Keep a safe copy of the private key. If it's lost, installed copies can't take updates anymore and users would have to reinstall by hand. Never commit it.

## Version numbers

- **Bug fixes:** `1.0.0` to `1.0.1`
- **New features:** `1.0.1` to `1.1.0`
- **Big changes that break things for users:** `1.1.0` to `2.0.0`
