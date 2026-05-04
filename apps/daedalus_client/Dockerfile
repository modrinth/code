# syntax=docker/dockerfile:1

FROM debian:trixie-slim

LABEL org.opencontainers.image.source=https://github.com/modrinth/code
LABEL org.opencontainers.image.title=daedalus
LABEL org.opencontainers.image.description="Modrinth game metadata query client"
LABEL org.opencontainers.image.licenses=MIT

RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates openssl \
	&& rm -rf /var/lib/apt/lists/*

COPY daedalus_client /daedalus/daedalus_client

WORKDIR /daedalus_client
CMD ["/daedalus/daedalus_client"]
