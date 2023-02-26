use once_cell::sync::Lazy;
use rtc;

static RT: Lazy<rtc::RemoteTerminal> = Lazy::new(|| {
    rtc::RemoteTerminal::new("127.0.0.1:7777").expect("could not initialise remote terminal")
});

fn main() -> std::io::Result<()> {
    RT.println_tagged("Hello, world!")?;
    Ok(())
}
