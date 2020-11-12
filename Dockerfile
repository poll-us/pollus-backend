FROM alpine:edge AS builder
# using edge because we need rust >=1.45, and 3.12 is at 1.44

# install rust
RUN apk add --no-cache cargo && cargo -V

# install refinery
RUN cargo install refinery_cli --no-default-features --features postgresql

# install postgres
RUN apk add --no-cache postgresql

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
RUN echo 'fn main() {}' >src/main.rs \
	&& cargo build --release \
	&& rm -r src/

# copy source files
COPY src/ src/
COPY migrations/ migrations/

# compile source code with database schema
ENV DATABASE_URL=postgres://postgres:postgres@localhost/pollus
RUN su postgres -c 'pg_ctl start --silent -w --pgdata=/var/lib/postgres/data -o "--data-directory=/var/lib/postgres/data"' \
	&& createdb -U postgres -E UTF-8 pollus \
	&& /root/.cargo/bin/refinery migrate -e DATABASE_URL files \
	&& cargo build --release \
	&& bins="$(find target/release -maxdepth 1 -type f -executable)" \
	&& strip $bins \
	&& mkdir bins/ \
	&& mv $bins bins/

################
FROM alpine:edge

RUN apk add --no-cache libgcc \
	&& mkdir -p /usr/local/bin

COPY --from=builder /src/bins /usr/local/bin/

EXPOSE 7181
CMD ["/usr/local/bin/pollus-backend"]

