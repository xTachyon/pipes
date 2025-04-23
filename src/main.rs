use anyhow::Result;
use pipes::{duplex_pipe, duplex_pipe_from_string};
use std::io::{BufRead, BufReader, Write};

fn main_impl() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let is_parent = args.len() == 1;

    if is_parent {
        let (mut dpipe, dpipe_to_send) = duplex_pipe()?;

        let arg = dpipe_to_send.to_string();
        dbg!(&arg);
        std::process::Command::new(std::env::current_exe().unwrap())
            .arg(arg)
            .spawn()?;
        drop(dpipe_to_send);

        dpipe.s.write_all(b"hello from parent")?;

        let mut buf = String::with_capacity(128);
        let mut rx = BufReader::new(dpipe.r);
        rx.read_line(&mut buf)?;

        assert_eq!(buf.trim(), "Hello from side B!");
        println!("{}", buf);
    } else {
        let mut dpipe = unsafe { duplex_pipe_from_string(args[1].as_str()) }?;

        dpipe.s.write_all(b"Hello from side B!\n")?;

        let mut buf = String::with_capacity(128);
        let mut rx = BufReader::new(dpipe.r);
        rx.read_line(&mut buf)?;

        assert_eq!(buf.trim(), "hello from parent");
        println!("{}", buf);
    }
    Ok(())
}

fn main() {
    match main_impl() {
        Ok(_) => {}
        Err(e) => eprintln!("Error: {}", e),
    }
    println!("bye");
}
