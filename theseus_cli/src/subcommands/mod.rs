use eyre::Result;

mod profile;
mod user;

#[derive(argh::FromArgs)]
#[argh(subcommand)]
pub enum SubCommand {
    Profile(profile::ProfileCommand),
    User(user::UserCommand),
}

impl crate::Args {
    pub async fn dispatch(&self) -> Result<()> {
        match self.subcommand {
            SubCommand::Profile(ref cmd) => cmd.dispatch(self).await,
            SubCommand::User(ref cmd) => cmd.dispatch(self).await,
        }
    }
}
