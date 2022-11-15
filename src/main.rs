mod args_parser;
mod class_id;
mod fs;
mod ui;
mod unity;
mod util;

fn main() -> std::io::Result<()> {
    use std::{io::stdout, panic, process::exit};

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

    panic::set_hook(Box::new({
        let default = panic::take_hook();
        move |payload| {
            ui::cleanup_terminal(&mut stdout()).unwrap();
            default(payload);
        }
    }));

    ui::run(project)?;
    ui::cleanup_terminal(&mut stdout())
}

fn print_usage() {
    todo!("the \"print_usage\" functionality")
}
