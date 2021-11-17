use sha1::Digest;
use std::{
    env,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

const SHA1_HEX_STRING_LENGTH: usize = 40;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: sha1_cracker <wordlist> <sha1_hash>");
        return Err("Improper Arguments".into());
    }

    let hash = args[2].trim();
    if hash.len() != SHA1_HEX_STRING_LENGTH {
        return Err("provided sha not valid sha1".into());
    }

    let wordlist = File::open(&args[1])?;
    let reader = BufReader::new(wordlist);

    for line in reader.lines() {
        let common_password = line?.trim().to_string();
        if hash == hex::encode(sha1::Sha1::digest(common_password.as_bytes())) {
            println!("Found Password {}", common_password);
            return Ok(());
        }
    }

    Ok(())
}
