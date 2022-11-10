use std::process::exit;

mod args_parser;
mod class_id;
mod fs;
mod unity;
mod util;
mod yaml_parser;

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

    let uos = yaml_parser::parse(fs::get_file_lines(project.scenes.last().unwrap()).unwrap());
    for uo in uos {
        println!("{}", uo);
    }

    todo!("proper error handling")
}

fn print_usage() {
    todo!("the \"print_usage\" functionality")
}
