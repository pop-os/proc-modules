extern crate proc_modules;

use proc_modules::ModuleIter;
use std::io;

fn main() -> io::Result<()> {
    for module in ModuleIter::new()? {
        println!("{:#?}", module?);
    }

    Ok(())
}
