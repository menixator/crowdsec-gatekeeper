mod diagnostics;

fn main() -> Result<(), color_eyre::Report> {
    diagnostics::setup()?;
    start()
}

#[tokio::main]
async fn start() -> Result<(), color_eyre::Report> {
    Ok(())
}
