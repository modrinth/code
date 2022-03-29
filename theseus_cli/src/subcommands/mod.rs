use eyre::Result;

mod profile;

#[derive(argh::FromArgs)]
#[argh(subcommand)]
pub enum SubCommand {
    Profile(profile::ProfileCommand),
}

impl crate::Args {
    pub async fn dispatch(&self) -> Result<()> {
        match self.subcommand {
            SubCommand::Profile(ref cmd) => cmd.dispatch(self).await,
        }
    }
}
