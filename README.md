# memproxy
Microservice wrapping HTTP REST around memcached protocol.

## Using memproxy

Standard rust cargo multi-binary setup:
```
$ cargo build
$ cargo run --bin memproxy
```

## Server configuration

A JSON configuration file is required, to specify memcached configuration.
Command line options are also available.

### Configuration file

See `example-cfg-memproxy.json` for an example configuration file.

### Command line help

Also, limited options are available at the command line.  Run `--help`
to view available options:

```
$ cargo run --bin memproxy -- --help
```

## Server API

Connect to HTTP endpoint using any web client.

### API: Service identity

GET /

### API: Stats and health check

GET /stats

### API: Get cache entry

GET /cache/$KEY

### API: Put cache entry

PUT /cache/$KEY

### API: Delete cache entry

DELETE /cache/$KEY

## Testing

Integration testing is performed via a separate binary, `tester`.
```
$ cargo run --bin tester
```

## Miscellaneous notes

* If the TCP connection to memcached is broken (eg. memcached restarts),
  the rust memcache library does not recover (does not start a new
  TCP connection).

