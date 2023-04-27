## Compiling

In order not to recompile everything from scratch to each project the easiest option is to set a common target-dir within a .cargo/config.toml:
 * $Workspace/.cargo/config.toml
 * $Home/.cargo/config.toml

```
cat .cargo/config.toml
[build]
  # ref: https://doc.rust-lang.org/cargo/reference/config.html#buildtarget-dir
  target-dir = "/home/fulup/.cargo/build"
  rustflags= "-L/home/fulup/.cargo/build/debug"

  # if sccache is installed on your platform
  rustc-wrapper = "/usr/bin/sccache"
  rustc-link-search="/usr/local/lib64"

[target.aarch64-unknown-linux-gnu]
  linker = "/usr/bin/aarch64-suse-linux-gcc"
  rustc-env = {CC = "/usr/bin/aarch64-suse-linux-gcc"}
```

- Can-player
  * create virtual can => sudo ./examples/etc/createvcan.sh vcan0
  * dump CAN trace => canplayer vcan0=elmcan -v -I examples/etc/candump.log -l i -g 1


