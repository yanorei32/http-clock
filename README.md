# HTTP Clock

## But, Why?

[HTTP Clockを支える技術 ACT.1](https://blog.yr32.net/post/http-clock-act1/)

## Hosted Version

https://httpclock.yr32.net/

## Self-host Quick Start

```bash
docker run --rm -it -p 3000:3000 ghcr.io/yanorei32/http-clock
```

## Self-build Quick Start

```bash
git clone https://github.com/yanorei32/http-clock
cd http-clock
cargo run # by default, listens in 0.0.0.0:3000
```

If you don't have cargo, you can get it with [rustup](https://www.rust-lang.org/tools/install).
