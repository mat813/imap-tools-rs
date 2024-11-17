mod commands;
mod libs;
mod run;

fn main() -> libs::error::OurResult<()> {
    run::run()
}
