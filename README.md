# SwissArmyBot

A Discord bot that does a whole bunch of things that no one needs. Like a Swiss Army Knife.

## Building and Running

Build in Docker
```sh
docker run --rm -v "$PWD:/src" --workdir="/src" --net=host rust cargo build --release
```

Run as systemd service unit
```sh
systemd-run --user --working-directory="$PWD" --unit=swissarmybot.service -- "$PWD/target/release/swissarmybot"
```
