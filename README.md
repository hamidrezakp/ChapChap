## ChapChap, Kill distracting Apps

## Install
### Cargo
```sh
$ cargo install chapchap
```

### Build from source
```sh
$ cargo install --path .
```

### Pre-Build Binary
**Soon**

## Config
the config file is `config.toml` that must be present next to binary
file while running.

The format of each `App` section in config file is like following:
```toml
[AppName]
enabled = true
slices = [ [13:10:00, 12:00:00] ] # you can write multiple time slice
black_list = false # time slices are black list or white list?
command = "mpv" # the command that application is running from
```
Note: `slices` filed consist of an array of time slices like [start,
end].

There is a `config.toml` file in repository for example.

## License
Apache v2 or MIT by your choice
