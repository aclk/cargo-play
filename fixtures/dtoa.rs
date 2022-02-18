//# rand = "0.8.5"
//# dtoa = { git = "https://github.com/dtolnay/dtoa.git" }

fn main() -> std::io::Result<()> {
    let mut buf = dtoa::Buffer::new();

    println!("{:?}", buf.format(2.71828f64));

    Ok(())
}
