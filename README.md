# proc-modules

Rust crate that provides easy access to active kernel modules in `/proc/modules`.

```rust
extern crate proc_modules;

use proc_modules::ModuleIter;
use std::io;

fn main() -> io::Result<()> {
    for module in ModuleIter::new()? {
        println!("{:#?}", module?);
    }

    Ok(())
}
```