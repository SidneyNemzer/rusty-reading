use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub mod units;

pub fn fixture_file_name(size: &units::Bytes) -> String {
    let (value, unit) = size.format_bytes();
    format!("{}{}.bin", value, unit.abbreviation().to_ascii_lowercase())
}

pub fn fixture_path(fixtures_dir: &Path, size: &units::Bytes) -> PathBuf {
    fixtures_dir.join(fixture_file_name(size))
}

/// Generates a fixture file of the given size. If the file already exists, this
/// function is a noop. The given fixtures_dir will be created if necessary.
pub fn gen_fixture(fixtures_dir: &Path, size: &units::Bytes) -> io::Result<()> {
    if size > &units::Bytes::from_unit(5, units::ByteUnit1024::Gibibytes) {
        panic!("Size {} exceeds maximum fixture size of 5 GiB.", size);
    }

    fs::create_dir_all(&fixtures_dir)?;

    let file_name = fixture_file_name(size);
    let file_path = fixtures_dir.join(file_name);

    if file_path.try_exists()? {
        eprintln!("Fixture exists: {}", file_path.to_string_lossy());
        return Ok(());
    }

    eprintln!("Generating fixture: {}", file_path.to_string_lossy());

    let mut f = File::create(file_path)?;

    const BUF_SIZE: usize = 1024 * 250;
    // ascii a = 97
    let mut buf = [97; BUF_SIZE];

    for _ in 0..(size.to_bytes() / BUF_SIZE) {
        f.write_all(&mut buf)?;
    }

    f.write_all(&mut buf[0..(size.to_bytes() % BUF_SIZE) as usize])?;

    Ok(())
}
