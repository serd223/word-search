
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

pub fn read_to(path: &str, buf: &mut String) -> std::io::Result<()> {
    {
        let path = Path::new(path);
        let display = path.display();   
        
        let mut file = File::open(path)?;
        match file.read_to_string(buf) {
            Ok(_) => (),
            Err(why) => panic!("Couldn't read {}: {}", display, why)
        };
    }
    Ok(())
}