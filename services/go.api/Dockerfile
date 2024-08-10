FROM golang:1.22 AS build_base

RUN apt update -y \
    && apt install -y git build-essential cmake zlib1g-dev

WORKDIR /tmp/go.api

RUN cd /tmp/go.api

COPY go.mod .
COPY go.sum .
RUN go mod tidy
COPY . .
RUN CGO_ENABLED=1 GOOS=linux GOARCH=amd64 LD_LIBRARY_PATH='/usr/local/lib' \
    go build -o ./out/go.api

FROM alpine:3
RUN apk add ca-certificates libc6-compat
COPY --from=build_base /usr/local/lib /usr/local/lib
COPY --from=build_base /tmp/go.api/out/go.api /app/go.api
CMD ["/app/go.api"]
LABEL org.opencontainers.image.source go.api