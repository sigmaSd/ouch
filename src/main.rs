use ouch::{commands, Opts, Result};

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        std::process::exit(ouch::EXIT_FAILURE);
    }
}

fn run() -> Result<()> {
    let (args, skip_questions_positively, progress_bar_policy) = Opts::parse_args()?;
    commands::run(args, skip_questions_positively, progress_bar_policy)
}
