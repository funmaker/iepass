use std::fs;

#[path = "src/rle.rs"]
mod rle;

fn main() -> Result<(), std::io::Error> {
    embuild::espidf::sysenv::output();
    
    Ok(())
}
