# ChapChap, Kill distracting Apps
ChapChap is a simple usage control app with time slice support.

## Installation
### Cargo
```sh
$ cargo install chapchap
```

### Build from source
```sh
$ cargo install --path .
```

### Pre-Build Binary
`x86_64` release is available.

## Usage
just run chapchap, it will get process tree and checks for programs you gave there is no need for additional arguments.

for ease you can create an systemd service so it will run on startup
```
[Service]
Type=simple
Restart=always
ExecStart=/bin/chapchap
```

## Config
The app first search config in `$XDG_CONFIG_HOME/chapchap/config.toml`.
If there is not `$XDG_CONFIG_HOME` environment variable, it search for config file
in current working directory or checks for `--config` argument.

The format of each `App` in config file is like following:
```toml
[[apps]]
name = "APPNAME"
enabled = true
slices = [ [13:10:00, 12:00:00], [19:00:10, 23:59:00] ] # you can write multiple time slice
black_list = false # time slices are black list or white list?
command = "mpv" # the command that application is running from
args = "*vid*" # the argument of command(supports regex) and can be empty
```
Note: `slices` filed consist of an array of time slices like [start,
end].

There is a `config.toml` file in repository for example of config file.

## License
Apache v2 or MIT by your choice
