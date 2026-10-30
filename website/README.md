# Tuxedo Task website

Static Astro website for **https://caracolaracol.github.io/tuxedo-task/**, with an
application guide at **https://caracolaracol.github.io/tuxedo-task/docs/**.
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

`npm test` runs 4 Node tests for documentation generation, then 26 browser tests.
They cover the landing and `/tuxedo-task/docs/`, source-derived reference content,
internal anchor integrity, desktop/mobile layouts, assets, metadata, fork
attribution, real-renderer screenshots, decorative roses and their license,
native FAQ/reference controls, clipboard success/failure, and content/navigation
with JavaScript disabled. Layout checks include 320, 375, 768, 1024, and 1440px
widths. Browser tests use the **production build**.

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
- `src/components/SiteHeader.astro` and `SiteFooter.astro`: shared navigation and
  credits, including local documentation links.
- `src/components/RoseDivider.astro`: decorative, locally hosted Twemoji roses.
- `src/styles/global.css`: responsive, paper/plum/cerise visual identity.
- `scripts/prepare-assets.mjs`: copies the app's original mask and real UI renders
  from the repository into `public/` before development/builds. The hero uses
  `docs/screenshots/subtasks.svg`; the workflow section uses `docs/screenshots/list.svg`.
  These come from the application's actual Ratatui renderer with sample tasks,
  not an HTML/CSS mockup. Generated copies are ignored; edit the originals.
- `public/social-card.svg` and `.png`: original social preview. After editing the
  SVG, run `npm run social-image` (requires the Playwright browser), then commit
  both files.

No analytics, external fonts, cookies, hosted backend, or UI framework runtime.
Installation, screenshots, and docs work without JavaScript. A small native
script enhances copy-to-clipboard; reduced-motion and keyboard users are supported.
The site preserves attribution to upstream Tuxedo and uses no third-party
character artwork. Rose illustrations were sourced online from Twemoji, are
hosted locally without modifications, and remain under CC BY 4.0 rather than MIT.
See [`THIRD_PARTY.md`](THIRD_PARTY.md) for pinned source URLs and license details;
the footer also credits the creators.

## Application documentation

`src/pages/docs.astro` renders a user guide with a sticky desktop index, a native
collapsible mobile index, reference tables, and highlighted code examples. It
uses Astro's native Markdown support, with no client-side documentation framework.

The technical reference is **generated from the repository's `README.md`**:

- `scripts/manual.mjs` extracts the selected application sections and rewrites
  links for the documentation route. It fails if an expected section disappears.
- `scripts/prepare-docs.mjs` combines those references with
  `src/content/start.md` and `src/content/reference-notes.md`.
- The result is `src/generated/manual.md`, which is ignored by Git. Do not edit it.
- The `predev`, `precheck`, and `prebuild` hooks regenerate the guide, so fresh
  checkouts and CI can import it before Astro compiles the page.
- The Pages workflow also watches root `README.md`, so reference edits redeploy
  the documentation. Run `npm run test:docs` to check the generator separately.

Change application reference material in the root README; change onboarding and
troubleshooting in the two source Markdown files. Keep claims about fork-only
features scoped to the original upstream base, rather than unverified current
upstream releases.

## GitHub Pages

The workflow is **`.github/workflows/pages.yml`**:

1. Pushes to `feat/github-pages-astro` or `main` affecting the README or site/shared assets trigger
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
