# Development

```sh
mise run fmt      # cargo fmt --all
mise run clippy   # cargo clippy --all-targets --locked -- -D warnings
mise run test     # cargo test --locked
```

CI runs all three on every push and pull request. Tasks are also runnable as
plain `cargo` commands if you don't use [mise](https://mise.jdx.dev/).

## Contributing

Issues and pull requests are welcome. For larger changes, please open an
issue first to discuss the approach. Run `mise run fmt clippy test` (or the
plain cargo equivalents) before submitting.

## Website

The [Tuxedo Task landing page](https://caracolaracol.github.io/tuxedo-task/) is a
static Astro site in [`website/`](../website). Its setup, browser tests, and GitHub
Pages deployment are documented in [`website/README.md`](../website/README.md).
The [online application guide](https://caracolaracol.github.io/tuxedo-task/docs/)
is generated from the Markdown guides in this folder. It is regenerated when the website builds.
The website is independent of the Rust application and does not change how tasks are stored.

See [release instructions](RELEASING.md) and the [screenshot gallery](screenshots.md).

[Documentation index](README.md)
