mod args_parser;
mod class_id;
mod fs;
mod ui;
mod unity;
mod util;
mod yaml_parser;

fn main() -> std::io::Result<()> {
    use std::process::exit;

    let path = match args_parser::parse() {
        Ok(path) => path,
        Err(e) => match e {
            args_parser::ArgsError::TooFew => {
                print_usage();
                exit(1);
            }
            args_parser::ArgsError::NotDir => {
                print_usage();
                exit(1);
            }
        },
    };

    let project = fs::find_project_files(&path).unwrap();

    // just shutting up the compiler's warnings about unused code
    {
        if let Some(file) = project.prefabs.first() {
            if let Ok(lines) = fs::get_file_lines(file) {
                let _parsed = yaml_parser::parse(lines);
            }
        }
    }

    ui::run(project)?;

    Ok(())
}

fn print_usage() {
    todo!("the \"print_usage\" functionality")
}
