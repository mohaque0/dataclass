use std::{io::{Read, Write}, net::TcpListener, path::PathBuf, process::{Command, Stdio}, time::Duration};

use ast::Root;

pub struct Generator {
    cmd: String
}

impl Generator {
    pub fn from(gen: &str) -> Generator {
        if gen.contains("/") || gen.contains("\\") {
            return Generator { cmd: gen.to_string() }
        }

        let possible_plugin_name = format!("dcg-{}", gen);
        match which::which(&possible_plugin_name) {
            Ok(_) => return Generator { cmd: possible_plugin_name},
            Err(_) => {},
        }

        panic!("Unable to find generator: {}", gen)
    }

    pub fn run(&self, ast: &Root, output: &PathBuf) -> std::io::Result<()> {
        println!("Running Generator: {}", self.cmd);

        let socket = TcpListener::bind("127.0.0.1:0")?;
        let addr = socket.local_addr()?;

        println!("Generator using port: {}", addr);

        socket.set_nonblocking(true)?;

        let mut cmd = Command::new(&self.cmd);
        cmd
            .arg(format!("{}", addr.port()))
            .arg(format!("{}", output.as_os_str().to_str().expect("Unknown error handling output path.")));
        
        println!("Generator Command Line: {:?}", cmd);

        let mut process = cmd.spawn()?;

        let mut poll = || -> std::io::Result<()> {
            let cmd_result = process.try_wait()?;
            if let Some(e) = cmd_result {
                return std::io::Result::Err(std::io::Error::new(
                    std::io::ErrorKind::Other, 
                    format!("Generator exited before connecting with status: {}", e)
                ));
            }

            let (mut stream, _) = socket.accept()?;
            stream.write_all(serde_json::to_string(&ast)?.as_bytes())?;
            drop(stream);
            Ok(())
        };

        loop {
            match poll() {
                Ok(_) => break,
                Err(_) => {
                    println!("Waiting for generator connection...");
                    std::thread::sleep(Duration::from_millis(500))
                },
            }
        }

        process.wait()?;

        Ok(())
    }
}