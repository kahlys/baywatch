# baywatch

## Usage

### Get started

1. Build the test docker image:

```sh
$ docker build -t myimage testdata
```

2. Run `baywatch`:

```sh
$ cargo run -- --image myimage

Docker infos
host ncpu : 6
host memtotal : 2083807232

+-----+---------------+
| CPU | DURATION (ms) |
+-----+---------------+
| 6   | 6003          |
+-----+---------------+
| 5   | 5002          |
+-----+---------------+
| 4   | 4008          |
+-----+---------------+
| 3   | 3005          |
+-----+---------------+
| 2   | 2006          |
+-----+---------------+
| 1   | 1001          |
+-----+---------------+
```