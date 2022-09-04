

pub fn read_to(path: &str, buf: &mut String) {
    {
        use std::io::prelude::*;
        use std::fs::File;
        use std::path::Path;

        let path = Path::new(path);
        let display = path.display();   
        
        let mut file = match File::open(path) {
             Ok(f) => f,
             Err(why) => panic!("Couldn't open {}: {}", display, why)
        };
        match file.read_to_string(buf) {
            Ok(_) => (),
            Err(why) => panic!("Couldn't read {}: {}", display, why)
        }
    }
}