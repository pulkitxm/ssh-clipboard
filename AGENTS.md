# Project architecture and agent guidance

## Discover before editing

- Start with `rg --files` and search for the domain term before opening broad files.
- Follow semantic directories: `src/daemon/`, `src/tui/setup/`, and `src/tui/monitor/` contain focused Rust concerns; `docs/src/components/site/`, `landing/`, `docs/`, and `demo/` contain focused website concerns.
- Name files after the concept they implement. Avoid generic buckets such as `helpers`, `utils`, `common`, or `misc` when a domain name is available.

## File size and responsibility

- Keep every hand-authored website source file and every Rust source file under 1,000 physical lines. This is a hard ceiling, not a target; split files well before they reach it.
- Lockfiles, generated output, vendored sources, and build artifacts are exempt. Do not hand-edit generated files.
- A file should own one semantic concept. When a file mixes state, rendering, transport, persistence, installation, or policy, move each concern into a matching module or component.
- Group related modules in a directory named for their parent concept. Keep the parent file focused on public API, composition, or orchestration, and re-export focused child APIs when needed.
- Prefer clear domain names such as `installation.rs`, `peer_version.rs`, `DocumentationLayout.vue`, or `UpdateGuide.vue` so a future agent can find behavior from filenames alone.

## Website structure

- The website lives in `docs/` and uses pnpm. Preserve `pnpm-lock.yaml` and use `pnpm install --frozen-lockfile` for deterministic installs.
- Keep `App.vue` composition-only. Put site chrome in `components/site/`, landing-page sections in `components/landing/`, documentation navigation and sections in `components/docs/`, and product demonstrations in `components/demo/`.
- Keep section-specific copy and data with the section that renders it. Put only genuinely shared documentation metadata in `components/docs/documentation.js`.
- Preserve SSR and static prerendering; browser-only APIs must run from Vue lifecycle hooks.
- Validate website changes with `cd docs && pnpm run build`.
- Production deploys originate from the connected Cloudflare Workers Build on pushes to `main`; do not deploy production directly with local Wrangler credentials.

## Rust structure

- Keep top-level modules centered on domain APIs and orchestration. Put focused implementation concepts in matching child modules, such as daemon socket clients, setup installation, or monitor version policy.
- Keep platform-specific clipboard behavior under `src/clipboard/`; TUI-specific workflows stay under `src/tui/`; daemon transport and control behavior stay under `src/daemon/`.
- Keep unit tests beside the behavior they cover. Use a named `tests.rs` child module when inline tests make the production module difficult to scan.
- Preserve async cancellation, socket permissions, update verification, and clipboard-format fidelity when moving code across modules.
- Validate Rust changes with `cargo fmt --all -- --check` and `cargo test --all-targets`.

## Line-count check

Before finishing structural work, verify source sizes with:

```sh
find src -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -n
find docs/src docs/worker docs/scripts -type f \( -name '*.vue' -o -name '*.js' -o -name '*.mjs' -o -name '*.css' \) -print0 | xargs -0 wc -l | sort -n
```

## Machine access and development previews

- This Mac (`MacBookServer`) is generally accessed over SSH from another machine on the same Tailscale network.
- Bind local development servers to `100.79.199.1` when supported, otherwise `0.0.0.0`; do not bind only to loopback unless explicitly requested.
- Report previews using `http://macbookserver.tail13bd39.ts.net:<port>` as the primary URL. If MagicDNS is unavailable, use `http://100.79.199.1:<port>`.
