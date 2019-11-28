# memproxy
Microservice wrapping HTTP REST around memcached protocol.

## Using memproxy

Standard rust cargo multi-binary setup:
```
$ cargo build
$ cargo run --bin memproxy
```

## Server configuration

A JSON configuration file is required, to specify database.  Command line 
options are also available.

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

### API: Service identity and status

```
$ curl http://localhost:8080/
```

Returns JSON describing service:
```
{
   "databases" : [
      {
         "name" : "db"
      }
   ],
   "version" : "0.1.0",
   "name" : "memproxy"
}
```

## Testing

Integration testing is performed via a separate binary, `tester`.
```
$ cargo run --bin tester
```

