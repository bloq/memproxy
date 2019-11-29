# memproxy
Microservice wrapping HTTP REST around memcached protocol.

## Using memproxy

### From rust cargo

Standard rust cargo multi-binary setup:
```
$ cargo build
$ cargo run --bin memproxy
```

### From docker

```
$ mkdir -p /tmp/conf && cp cfg-memproxy.json /tmp/conf
$ docker run --rm -p 8080:8080 -v /tmp/conf:/conf memproxy memproxy --bind-addr 0.0.0.0 --config /conf/cfg-memproxy.json
$ curl http://127.0.0.1:8080/ | json_pp
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
* If memcached is not reachable, program will not start.

