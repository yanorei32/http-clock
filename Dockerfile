FROM rust:1.86.0-bookworm as build-env
LABEL maintainer="yanorei32"

SHELL ["/bin/bash", "-o", "pipefail", "-c"]

WORKDIR /usr/src
RUN cargo new http-clock
COPY LICENSE Cargo.toml Cargo.lock /usr/src/http-clock/
WORKDIR /usr/src/http-clock
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN	cargo install cargo-license && cargo license \
	--authors \
	--do-not-bundle \
	--avoid-dev-deps \
	--avoid-build-deps \
	--filter-platform "$(rustc -vV | sed -n 's|host: ||p')" \
	> CREDITS

RUN cargo build --release
COPY src/ /usr/src/http-clock/src/
COPY assets/ /usr/src/http-clock/assets/
RUN touch  assets/* src/* && cargo build --release

FROM debian:bookworm-slim@sha256:4b50eb66f977b4062683ff434ef18ac191da862dbe966961bc11990cf5791a8d

WORKDIR /

COPY --chown=root:root --from=build-env \
	/usr/src/http-clock/CREDITS \
	/usr/src/http-clock/LICENSE \
	/usr/share/licenses/http-clock/

COPY --chown=root:root --from=build-env \
	/usr/src/http-clock/target/release/http-clock \
	/usr/bin/http-clock

CMD ["/usr/bin/http-clock"]
