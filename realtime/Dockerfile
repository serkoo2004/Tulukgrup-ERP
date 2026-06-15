FROM golang:1-bookworm AS builder

WORKDIR /app/realtime
COPY realtime/go.mod ./
RUN go mod download
COPY realtime ./
RUN go build -o /usr/local/bin/tuluklar-realtime ./cmd/gateway

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/tuluklar-realtime /usr/local/bin/tuluklar-realtime
EXPOSE 8090 7070
CMD ["tuluklar-realtime"]
