use std::fs;

#[path = "src/rle.rs"]
mod rle;

fn main() -> Result<(), std::io::Error> {
    embuild::espidf::sysenv::output();
    
    if !fs::exists("assets/BadApple.smol")? {
        let mut enc = rle::Encoder::new(fs::File::create("assets/BadApple.smol")?);
        std::io::copy(&mut fs::File::open("assets/BadApple.raw")?, &mut enc)?;
        let (_, res) = enc.finish();
        res?;
    }
    
    Ok(())
}
