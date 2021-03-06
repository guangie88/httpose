# `httpose`

[![Codefresh build status]( https://g.codefresh.io/api/badges/pipeline/guangie88/public-repos%2Fhttpose?branch=master&key=eyJhbGciOiJIUzI1NiJ9.NWM4MjcyMzg3Y2NkOTUzZTcxM2RiMjRl.cTJ8XB8rM4mRl2LmZBHaIVZ92MxdGgb7Mmib1jt8o4E&type=cf-1)]( https://g.codefresh.io/pipelines/httpose/builds?repoOwner=guangie88&repoName=httpose&serviceName=guangie88%2Fhttpose&filter=trigger:build~Build;branch:master;pipeline:5da908bcba23a425e23f2d3f~httpose)

A quick and dirty solution to "HTTP-expose" secret meant for Docker build.

**Caveat**: This does not employ any SSL over HTTP, and assumes the working
networking environment is safe without encryption (e.g. `localhost`).

## Get Started

Simply run

```bash
# stdin
## Serves at `127.0.0.1:2048` by default
./httpose (secret will be consumed via stdin)
## (CTRL-C to terminate)

## Pipe in also works
echo xxx | ./httpose
## (CTRL-C to terminate)

# env var
HTTPOSE_SECRET=xxx ./httpose
## (CTRL-C to terminate)

# by file
echo xxx > /tmp/secret
./httpose -f /tmp/secret
## (CTRL-C to terminate)
rm /tmp/secret

## Serves at 0.0.0.0 interface and port 12345
## Not recommended to serve out of 127.0.0.1 unless within Docker bridge mode
./httpose -a 0.0.0.0:12345

## Help details
./httpose -h
```

You can get the secret value by `curl`-ing or `wget` when the service is up:

```bash
# curl
curl -s http://127.0.0.1:2048/

# wget
wget -qO - http://127.0.0.1:2048/
```

## Changelogs

See [CHANGELOG.md](CHANGELOG.md) for more details.

## Cargo build

You will need to install `cargo` and `rustc`. See (`rustup`)[<https://rustup.rs]>
for more information.

To build in release mode via `cargo`, simply run:

```bash
cargo build --release
```

To execute, simply run:

```bash
cargo run --release -- [args...]
```

## Docker build

To build the image, simply run:

```bash
docker build . -t httpose

# Host network mode for simplicity
docker run --rm -it --net host httpose

# Bridge mode, requires changing of listening address
docker run --rm -it -p 2048:2048 httpose -a "0.0.0.0:2048"

# Secret by env var (not recommended because docker inspect can see the secret)
docker run -e HTTPOSE_SECRET=xxx --rm -it --net host httpose

# Secret by file
docker run -v "`pwd`/xxx:/xxx" --rm -it --net host httpose -f "/xxx"

# Help details
docker run --rm -it httpose -h
```
