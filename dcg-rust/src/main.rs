use std::{io::Read, net::TcpStream, num::ParseIntError, path::PathBuf};

use clap::{App, Arg};

#[derive(Debug)]
enum AppError {
    Msg(String)
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Msg(format!("{:?}", e))
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Msg(format!("{:?}", e))
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Msg(format!("{}", s))
    }
}

#[derive(Debug)]
struct AppConfig {
    port: i32,
    output_dir: PathBuf
}

fn process_args(args: App) -> Result<AppConfig, AppError> {
    let args = args.get_matches();

    let port = args.value_of("port").ok_or("No port specified.")?;
    let output_dir = args.value_of("output_dir").ok_or("No output dir specified.")?;

    let port = port.parse::<i32>()?;
    let output_dir = PathBuf::from(output_dir);

    Ok(AppConfig {
        port,
        output_dir,
    })
}

fn process_stream(config: AppConfig) -> Result<(), AppError> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", config.port))?;

    let mut str = String::new();
    let nread = stream.read_to_string(&mut str)?;
    
    println!("Read {} bytes.", nread);

    Ok(())
}

fn app() -> Result<(), AppError> {
    let app = App::new("dcg-rust")
        .version("0.1")
        .about("DataClass Rust Generator")
        .arg(
            Arg::with_name("port")
                .help("Port to connect to ast server.")
                .multiple(false)
                .required(true)
                .index(1)
        )
        .arg(
            Arg::with_name("output_dir")
                .help("Output directory.")
                .multiple(false)
                .required(true)
        );

    let config = process_args(app)?;
    println!("{:#?}", config);
    process_stream(config)
}


fn main() {
    match app() {
        Ok(_) => {},
        Err(e) => {
            println!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}