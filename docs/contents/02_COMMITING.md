# Committing

Try to stick to Conventional Commits:

- `type: short summary`
- or `type(scope): short summary`

#### Common types:

- `feat:` → new features
- `fix:` → bug fixes
- `docs:` → documentation changes
- `refactor:` → internal cleanup (no behavior change)
- `chore:` → for small project updates
- `test:` → tests

Keep it short and clear. Don't overthink it.

If it helps, include the issue number:

- `fix(picker): keep the loupe on screen near the edges (#6)`

#### Examples:

- `feat(settings): add a close to tray option`
- `fix(toast): stop the shadow getting cut off`
- `docs: clarify the setup steps`


## Before pushing

- Make sure it works locally with `npm run dev`
- Try a pick, the settings you touched, and the welcome screen if you changed it
- Keep commits focused (don't mix unrelated stuff)
- Push with a clear message

## Your commits are public

Your commit messages end up in the release notes, so write them for the people using XPicker. For example, `fix: show the picker on the monitor the mouse is on` tells a user what changed, while `fix: stuff` doesn't.

#### [03. Releasing ➔](/docs/contents/03_RELEASING.md)
