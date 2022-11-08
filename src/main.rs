use std::process::exit;

mod args_parser;
mod class_id;
mod fs;
mod unity;
mod yaml_parser;
mod util;

fn main() {
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

    for path in project.scenes {
        println!("{:?}", path)
    }

    todo!("proper error handling")
}

fn print_usage() {
    todo!("the \"print_usage\" functionality")
}
