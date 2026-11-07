# Releasing

This page is for maintainers. If you're contributing, you don't need any of this.

## How it works

| Branch | For | Commits |
|---|---|---|
| `dev` | Testing | Anything, messy is fine |
| `main` | Users | Only finished work |

Users only get what you publish as a release. Installed copies of XPicker check GitHub Releases from **Settings → Updates**, so pushing to `dev` or `main` doesn't reach them.

Never make a version tag by hand. Publishing a release creates it.

`dev` is public, so don't commit anything private to it.

## Testing on dev

Commit your changes and push them:

```sh
git switch dev
git add -A
git commit -m "test thing"
git push origin dev
```

Then run it with `npm run dev`, see [Setup](/docs/contents/01_SETUP.md).

## Moving finished work to main

When it works, turn it into one clean commit:

```sh
git switch main
git merge --squash dev
git commit -m "feat: add the thing"
git push origin main
```

See [Committing](/docs/contents/02_COMMITING.md) for how to write the message, because it appears in the release notes.

## Testing a pull request

Using the pull request's number:

```sh
git fetch origin pull/123/head:pr-123
git switch pr-123
npm install
npm run dev
```

## Merging a pull request

Use **Squash and merge** on GitHub, so it lands as one clean commit. Then bring it to your PC:

```sh
git switch main
git pull origin main
```

## Publishing a release

Test everything on `dev` first, and make sure it's on `main`.

1. Set the new version, like `1.1.0`, with no `v`:

   ```sh
   git switch main
   npm version 1.1.0 --no-git-tag-version
   ```

   This changes `package.json` and copies the number into `src-tauri/Cargo.toml`. The app reads its version from `package.json`, so that's the only place it lives.

2. Commit and push it:

   ```sh
   git add package.json package-lock.json src-tauri/Cargo.toml
   git commit -m "chore: release 1.1.0"
   git push origin main
   ```

3. On GitHub, open **Releases**. The workflow makes a draft called `v1.1.0` with a downloads table and the commits sorted under headings, with the contributors listed. Then it builds Windows, macOS and Linux into it. This takes around 15 minutes. Users get nothing yet.
4. Edit the notes. Remove anything that doesn't matter to users.
5. Click **Publish release**. This creates the tag, and **Check for updates** in installed copies now finds it.

Publishing also makes `dev` behind `main`. Bring it along:

```sh
git switch dev
git fetch origin
git reset --hard origin/main
git push --force origin dev
```

Then check it: install the previous version, open **Settings → Updates**, click **Check for updates**, and make sure it updates and shows the new number.

## The updater key

Updates are signed, and the app refuses any update that isn't. The key's private half is the `TAURI_SIGNING_PRIVATE_KEY` secret in the repository settings; the public half is in `src-tauri/tauri.conf.json`.

Keep a safe copy of the private key. If it's lost, installed copies can't take updates anymore and users would have to reinstall by hand. Never commit it.

## Version numbers

- **Bug fixes:** `1.0.0` to `1.0.1`
- **New features:** `1.0.1` to `1.1.0`
- **Big changes that break things for users:** `1.1.0` to `2.0.0`

If the workflow fails, a draft isn't made and nothing reaches users. **Actions** shows what failed. Fix it and push the fix to `main`, then open **Actions**, then **Release**, and click **Run workflow** to try again with the same number.
