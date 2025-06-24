# syntax=docker/dockerfile:1

FROM postgres:17.4-bookworm

RUN printf "deb http://httpredir.debian.org/debian bookworm-backports main non-free\ndeb-src http://httpredir.debian.org/debian bookworm-backports main non-free" > /etc/apt/sources.list.d/backports.list
RUN apt-get update
RUN apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq-dev \
    libpq5 \
    libssl-dev \
    libxml2-dev \
    libxslt1-dev \
    make \
    python3 \
    python3-pip \
    python3-setuptools \
    python3-wheel \
    unzip \
    curl \
    git \
    golang-1.23 \
    postgresql-contrib-$PG_MAJOR

# Set environment variables
ENV POSTGRES_USER=antiraid
ENV POSTGRES_PASSWORD=AnTiRaId123!
ENV POSTGRES_DB=antiraid
ENV POSTGRES_PORT=5432
ENV POSTGRES_INITDB_ARGS="--encoding=UTF8 --locale=C --lc-collate=C --lc-ctype=C"

RUN git clone https://github.com/anti-raid/ibldev /ibl
WORKDIR /ibl
RUN --mount=type=cache,target=/go/pkg/mod \
    --mount=type=cache,target=/root/.cache/go-build \ 
    /usr/lib/go-1.23/bin/go build -v -o ibl

RUN mkdir -p /docker-entrypoint-initdb.d
COPY ./setup_seed.sh /docker-entrypoint-initdb.d/10_setup_seed.sh
