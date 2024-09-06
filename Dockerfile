FROM rust:1.81.0-bookworm as build-env
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

FROM debian:bookworm-slim@sha256:16112ae93b810eb1ec6d1db6e01835d2444c8ca99aa678e03dd104ea3ec68408

WORKDIR /

COPY --chown=root:root --from=build-env \
	/usr/src/http-clock/CREDITS \
	/usr/src/http-clock/LICENSE \
	/usr/share/licenses/http-clock/

COPY --chown=root:root --from=build-env \
	/usr/src/http-clock/target/release/http-clock \
	/usr/bin/http-clock

CMD ["/usr/bin/http-clock"]
