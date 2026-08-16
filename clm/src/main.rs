use std::time::Duration;

use clm::app::{App, Msg};
use clm::ctx::Ctx;
use color_eyre::eyre::Result;
use tea::browser::Program;

fn main() -> Result<()> {
    color_eyre::install()?;

    let ctx = Ctx::open("clm.db")?;
    let program = Program::new(
        App::start(),
        App::init,
        Msg::RequestedRoute,
        Msg::ChangedRoute,
    );

    let _ = tea::run(
        "Command Line Money",
        ctx,
        program,
        Duration::from_millis(250),
    );

    Ok(())
}
