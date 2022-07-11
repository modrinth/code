//! User management subcommand
use eyre::Result;
use paris::*;
use theseus::prelude::*;
use tokio::sync::oneshot;

#[derive(argh::FromArgs)]
#[argh(subcommand, name = "user")]
/// user management
pub struct UserCommand {
    #[argh(subcommand)]
    action: UserSubcommand,
}

#[derive(argh::FromArgs)]
#[argh(subcommand)]
pub enum UserSubcommand {
    Add(UserAdd),
}

#[derive(argh::FromArgs)]
/// Add a new user to Theseus
#[argh(subcommand, name = "add")]
pub struct UserAdd {
    #[argh(option)]
    /// the browser to authenticate using
    browser: Option<webbrowser::Browser>,
}

impl UserAdd {
    pub async fn run(
        &self,
        _args: &crate::Args,
        _largs: &UserCommand,
    ) -> Result<()> {
        info!("Adding new user account to Theseus");
        info!("A browser window will now open, follow the login flow there.");

        let (tx, rx) = oneshot::channel::<url::Url>();
        let flow = tokio::spawn(auth::authenticate(tx));

        let url = rx.await?;
        match self.browser {
            Some(browser) => webbrowser::open_browser(browser, url.as_str()),
            None => webbrowser::open(url.as_str()),
        }?;

        let credentials = flow.await??;
        success!("Logged in user {}.", credentials.username);
        Ok(())
    }
}

impl UserCommand {
    pub async fn dispatch(&self, args: &crate::Args) -> Result<()> {
        match &self.action {
            UserSubcommand::Add(ref cmd) => cmd.run(args, self).await,
        }
    }
}
