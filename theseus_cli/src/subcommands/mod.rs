use eyre::Result;

mod profile;
mod user;

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand)]
pub enum Subcommand {
    Profile(profile::ProfileCommand),
    User(user::UserCommand),
}

impl crate::Args {
    pub async fn dispatch(&self) -> Result<()> {
        dispatch!(self.subcommand, (self) => {
            Subcommand::Profile,
            Subcommand::User
        })
    }
}
