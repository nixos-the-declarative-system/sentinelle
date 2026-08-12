# sentinelle

The CLI of the Sentinelle monitoring platform: probe your HTTPS
services (availability, latency, certificate expiry) from the
terminal or from your scripts.

## Download

Linux x86-64 binary, compatible with all distributions:

```console
$ curl -LO https://github.com/nixos-the-declarative-system/sentinelle/releases/download/v2.8.0/sentinelle-linux-x86_64.tar.gz
$ tar xf sentinelle-linux-x86_64.tar.gz
$ cd sentinelle-2.8.0
$ ./sentinelle --version
sentinelle 2.8.0 (linux/amd64)
```

## Usage

```console
$ sentinelle check example.org
● example.org — online
  HTTP 200 in 132 ms
  certificate: 42 days left (Sep 23 12:00:00 2026 GMT, issued by R3)
```

`--json` prints one JSON line per target, `--timeout` bounds each
probe, `--cert-warning` sets the warning threshold for certificate
expiry (14 days by default). The exit code follows the convention of
monitoring probes: 0 online, 1 warning, 2 offline.

## About this project

`sentinelle` is an extra in the book *NixOS: The Declarative
System*: the "binary from elsewhere" that the chapter of the same
name teaches you to run on NixOS. In the book's story, it is the
proprietary tool of a monitoring platform: no sources, no possible
recipe. Yet the sources are here, public like everything this
account publishes. Play along, and act as if all you had was the
binary from the release. The "compatible with all distributions"
promised above is exactly what the chapter puts to the test.

The tool really works, but the platform it evokes does not exist,
and nobody monitors anything with it.

## Building from source

```console
$ cargo build --release
```

The official release is built in a Debian 12 container, so that the
binary is linked against the glibc and OpenSSL 3 of a mainstream
distribution: that is the whole subject of the chapter. See
[dist.sh](dist.sh).

## License

MIT — see [LICENSE](LICENSE).
