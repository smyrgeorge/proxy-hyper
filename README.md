## Project goals
The goal of this project is to create a *simple reverse proxy* using the Rust programming language. Build on top of [hyper](https://hyper.rs/) (*hyper is a fast and correct HTTP implementation written in Rust*).

## Features
- Fully configurable reverse proxy (see usage below).
- Supports *Bearer Authorization*. If authentication is enabled, through the configuration file, the proxy validates *jwt* token. Then creates a new HTTP header *"x-real-name"*, which contains the user identity. The new header is encoded in base64.
- Logging using [log4rs](https://docs.rs/log4rs/0.13.0/log4rs/).

## Pending
- [ ] Add headers *"[x-forwarded-proto](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-Proto)"* and *"[x-forwarded-for"](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For)*.
- [ ] Better path matching.
- [ ] Test several critical parts.
- [ ] Performance test.

## Build

``` sh
# build release
cargo build --release

# or additioanlly build debug
cargo build
```

## Run

``` sh
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

## Configuration
``` toml
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
  rsa_modulus ='''
  PLACE_MODULUS_HERE
  '''
  rsa_exponent = "PLACE_EXPONENT_HERE"

  [[proxy.hosts]]
  path = "/"
  host = "localhost:3000"
```
