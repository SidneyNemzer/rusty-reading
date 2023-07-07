use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

pub fn read_bufreader(buffer_size: usize, file: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::with_capacity(buffer_size, File::open(file)?);

    loop {
        let buf = reader.fill_buf()?;
        let n = buf.len();

        if n == 0 {
            break;
        }

        reader.consume(n);
    }

    Ok(())
}

pub fn read(buffer_size: usize, file: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = File::open(file)?;
    let mut buf = vec![0; buffer_size];

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
    }

    Ok(())
}

pub fn read_sizes(buffer_size: usize, file: &str) -> Result<Vec<usize>, Box<dyn Error>> {
    let mut reader = File::open(file)?;
    let mut buf = vec![0; buffer_size];
    let mut read_sizes = Vec::new();

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }

        read_sizes.push(n);
    }

    Ok(read_sizes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let buffer_size = 1024;
        let file = "assets/01_number.s";
        read(buffer_size, file).unwrap();
    }

    #[test]
    fn test_read_sizes() {
        let buffer_size = 1024;
        let file = "assets/01_number.s";
        let read_sizes = read_sizes(buffer_size, file).unwrap();

        assert!(read_sizes.len() > 0);
        assert!(read_sizes.iter().all(|&x| x <= buffer_size && x > 0));
    }
}
