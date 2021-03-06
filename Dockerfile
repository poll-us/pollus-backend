FROM alpine:edge AS builder
# using edge because we need rust >=1.45, and 3.12 is at 1.44

# install rust
RUN apk update && apk upgrade && apk add cargo && cargo -V

# install refinery
RUN cargo install refinery_cli --no-default-features --features postgresql

# install postgres
RUN apk add postgresql

# setup postgres
RUN install -d -m 0750 -o postgres -g postgres /var/lib/postgres/data \
	&& install -d -m 0755 -o postgres -g postgres /run/postgresql
WORKDIR /var/lib/postgres/data
USER postgres
RUN initdb --pgdata "$PWD"
USER root

# copy cargo files
RUN mkdir -p /src/src
WORKDIR /src
COPY Cargo.lock Cargo.toml ./

# pre-compile dependencies
RUN echo 'fn main() { panic!("CHECK YOUR DOCKERFILE"); }' >src/main.rs \
	&& cargo build --release \
	&& rm -r src/

# copy source files
COPY src/ src/
COPY migrations/ migrations/
RUN touch src/main.rs

# compile source code with database schema
ENV DATABASE_URL=postgres://postgres:postgres@localhost/pollus
RUN su postgres -c 'pg_ctl start --silent -w --pgdata=/var/lib/postgres/data -o "--data-directory=/var/lib/postgres/data"' \
	&& createdb -U postgres -E UTF-8 pollus \
	&& /root/.cargo/bin/refinery migrate -e DATABASE_URL files \
	&& cargo build --release \
	&& bins="$(find target/release -maxdepth 1 -type f -executable)" \
	&& mkdir bins/ \
	&& cp $bins bins/ \
	&& strip bins/*

################
FROM alpine:edge

LABEL org.opencontainers.image.source https://github.com/poll-us/pollus-backend
# this list should cover all dependencies, run `cargo lichking list` to see all licenses
LABEL org.opencontainers.image.licenses Apache-2.0 AND ISC AND MIT AND MPL-2.0

RUN apk upgrade --no-cache \
	&& apk add --no-cache libgcc \
	&& mkdir -p /usr/local/bin

COPY --from=builder /src/bins /usr/local/bin/

EXPOSE 7181
ENV RUST_LOG=info
CMD ["/usr/local/bin/pollus-backend"]

