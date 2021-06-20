use std::{io::Read, net::TcpStream, num::ParseIntError, path::PathBuf};

use clap::{App, Arg};

#[derive(Debug)]
pub enum AppError {
    Msg(String)
}

#[derive(Debug)]
struct AppConfig {
    port: i32,
    output_dir: PathBuf
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

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Msg(format!("{:?}", e))
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Msg(format!("{}", s))
    }
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

fn process_stream(config: AppConfig) -> Result<ast::Root, AppError> {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", config.port))?;

    let mut str = String::new();
    let nread = stream.read_to_string(&mut str)?;
    
    println!("Read {} bytes.", nread);

    let ast = serde_json::from_str::<ast::Root>(&str)?;

    Ok(ast)
}

pub fn get_ast(app_name: &str, app_desc: &str) -> Result<ast::Root, AppError> {
    let app = App::new(app_name)
        .version("0.1")
        .about(app_desc)
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