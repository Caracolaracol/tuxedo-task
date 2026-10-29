# Tuxedo Task website

Static Astro landing page for **https://caracolaracol.github.io/tuxedo-task/**.
The website lives in `website/`, independently of the Rust application. The
`feat/github-pages-astro` branch can deploy it without merging into `main`.

## Local development

Requires Node **22.12+** and npm. Astro is pinned to **7.3.8**, the patched release
compatible with the project's current Node 22.21.1 installation. The lockfile
makes CI installs reproducible. An `esbuild` override fixes a transitive Vite
security advisory; review it when upgrading Vite/Astro.

From this directory:

```sh
npm ci
npm run dev
```

Open **http://127.0.0.1:4321/tuxedo-task/**, not the host root. `site` and `base` in
`astro.config.mjs` match the GitHub project-site URL. Change both if moving to a
custom domain or another repository name.

## Validation

```sh
npm run check
npm audit --audit-level=low
npm run build
npx playwright install chromium
npm test
```

The 14 browser tests cover desktop/mobile layouts, the `/tuxedo-task/` subpath,
assets, metadata, fork attribution, accessible preview tabs, native FAQ controls,
clipboard success/failure, and content with JavaScript disabled. Layout checks
include 320, 375, 768, 1024, and 1440px widths. They test the **production build**.

For a manual production preview:

```sh
npm run build
npm run preview
```

Astro may auto-background dev/preview when run by an AI agent. Playwright uses
`--ignore-lock` so its own preview remains a foreground child it can clean up.
An explicitly backgrounded manual preview can be stopped with
`./node_modules/.bin/astro preview stop`.

## Assets and content

- `src/pages/index.astro`: content, metadata, install command and clipboard logic.
- `src/components/TerminalPreview.astro`: illustrative sample, not an embedded app.
- `src/styles/global.css`: responsive, paper/plum/cerise visual identity.
- `scripts/prepare-assets.mjs`: copies the app's original mask and the real
  screenshot from the repository into `public/` before development/builds.
  Generated copies are ignored; edit the originals in `src/serve/assets/icon.svg`
  and `docs/screenshots/subtasks.svg` at the repository root.
- `public/social-card.svg` and `.png`: original social preview. After editing the
  SVG, run `npm run social-image` (requires the Playwright browser), then commit
  both files.

No analytics, external fonts, cookies, hosted backend, or UI framework runtime.
Installation and docs work without JavaScript. Small native scripts enhance
preview tabs and copy-to-clipboard; reduced-motion and keyboard users are supported.
The site preserves attribution to upstream Tuxedo and uses no third-party
character artwork.

## GitHub Pages

The workflow is **`.github/workflows/pages.yml`**:

1. Pushes to `feat/github-pages-astro` or `main` affecting site/shared assets trigger
   installation, dependency audit, type checks, static build and browser tests.
2. Pull requests run the same validation but **cannot deploy**.
3. Successful branch builds upload `website/dist` and deploy to GitHub Pages with
   a short-lived OIDC token, not a personal access token.

Repository **Settings → Pages → Build and deployment → Source** must be set to
**GitHub Actions**. The `github-pages` environment must allow the deployment
branch. It is intentionally configured to allow `feat/github-pages-astro` (and
`main` for a future merge), not arbitrary branches.

After merging the website into `main`, remove the feature-branch trigger and its
environment rule, or delete the feature branch, so only `main` publishes updates.
The application release workflow and Rust source are unchanged by the website.
