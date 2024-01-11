# raiuk

A simple implementation of a chat server using [TCP protocol](https://en.wikipedia.org/wiki/Transmission_Control_Protocol) and [Tokio](https://tokio.rs/).

## Usage

The following command will start the server on port `8000`.

```bash
$ cargo run
```

If you want test the server, you can use `telnet` to connect to the server.

```bash
$ telnet localhost 8000
```
