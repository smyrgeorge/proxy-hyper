## Status

![Build](https://github.com/smyrgeorge/proxy-hyper/workflows/Build/badge.svg)

## Project goals

The goal of this project is to create a _simple reverse proxy_ using the Rust programming language. Build on top of [hyper](https://hyper.rs/) (_hyper is a fast and correct HTTP implementation written in Rust_).

## Features

- Fully configurable reverse proxy (see usage below).
- Supports _Bearer Authorization_. If authentication is enabled, through the configuration file, the proxy validates _jwt_ token. Then creates a new HTTP header _"x-real-name"_, which contains the user identity. The new header is encoded in base64.
- Logging using [log4rs](https://docs.rs/log4rs/0.13.0/log4rs/).

## Pending

- [ ] Use tracing.
- [ ] Repackage several parts of the code, using 'mod.rs'.
- [ ] Add headers _"[x-forwarded-proto](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-Proto)"_ and _"[x-forwarded-for"](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For)_.
- [ ] Better path matching.
- [ ] Test several critical parts.
- [ ] Performance test.

## Build

```sh
# build release
cargo build --release

# or additioanlly build debug
cargo build
```

## Run

```sh
# run release
caro run --release
```

## Command line options

```
~/ » ./proxy-hyper --help

proxy-hyper 0.1
George S. <smyrgeorge@gmail.com>
Command line arguments

USAGE:
    proxy-hyper [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config-file <config-file>
            Sets a custom config file [default: config/default.toml]

    -l, --log-file <log-file>
            Sets a custom log config file [default: config/log4rs.yml]

        --private-config-file <private-config-file>
            Sets a private config file (overrides config file) [default: config/private.toml]

```

## Configuration :: default.conf

```toml
[server]
host = "127.0.0.1"
port = 8000

[proxy]
scheme = "http"

  [proxy.auth]
  # Enable/Disable auth.
  auth = true

  # Do not commit tokens/keys in the code.
  # IMPORTANT: jwt tokens are credentials.
  # NOTE: spaces and new line characters will be trimmed.
  # For local development please use config/local.toml.
  alg = "RS256"

  # In order to extract modulus and exponent from the RSA public key,
  # you can use a small Java program (located in scripts folder).
  rsa_modulus ='''
  PLACE_MODULUS_HERE
  '''
  rsa_exponent = "PLACE_EXPONENT_HERE"

  [[proxy.hosts]]
  path = "/"
  host = "localhost:3000"
```

## Configuration :: log4rs.yml

```yaml
# Sample file can be found here:
# https://docs.rs/log4rs/0.13.0/log4rs/#configuration-via-a-yaml-file

# Scan this file for changes every 30 seconds
refresh_rate: 30 seconds

appenders:
  # An appender named "stdout" that writes to stdout
  stdout:
    kind: console
    encoder:
      # documentation can be found here:
      # https://github.com/estk/log4rs/blob/master/src/encode/pattern/mod.rs
      pattern: "{d}{h([{l}]):>8.15} [{T}] - {m}{n}"

# Set the default logging level to "info" and attach the "stdout" appender to the root
root:
  level: info
  appenders:
    - stdout
```
