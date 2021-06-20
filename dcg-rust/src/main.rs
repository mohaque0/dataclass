use app_common::AppError;

fn app() -> Result<(), AppError> {
    app_common::get_ast("dcg-cpp", "DataClass CPP Generator")?;
    Ok(())
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