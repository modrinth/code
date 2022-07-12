//! User management subcommand
use crate::util::table;
use eyre::Result;
use paris::*;
use tabled::Tabled;
use theseus::prelude::*;
use tokio::sync::oneshot;

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand, name = "user")]
/// user management
pub struct UserCommand {
    #[argh(subcommand)]
    action: UserSubcommand,
}

#[derive(argh::FromArgs, Debug)]
#[argh(subcommand)]
pub enum UserSubcommand {
    Add(UserAdd),
    List(UserList),
}

#[derive(argh::FromArgs, Debug)]
/// add a new user to Theseus
#[argh(subcommand, name = "add")]
pub struct UserAdd {
    #[argh(option)]
    /// the browser to authenticate using
    browser: Option<webbrowser::Browser>,
}

impl UserAdd {
    #[tracing::instrument]
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

#[derive(argh::FromArgs, Debug)]
/// list all known users
#[argh(subcommand, name = "list")]
pub struct UserList {}

#[derive(Tabled)]
struct UserRow<'a> {
    id: uuid::Uuid,
    username: &'a str,
    default: bool,
}

impl<'a> UserRow<'a> {
    pub fn from(
        credentials: &'a Credentials,
        default: Option<uuid::Uuid>,
    ) -> Self {
        Self {
            id: credentials.id,
            username: &credentials.username,
            default: Some(credentials.id) == default,
        }
    }
}

impl UserList {
    #[tracing::instrument]
    pub async fn run(
        &self,
        _args: &crate::Args,
        _largs: &UserCommand,
    ) -> Result<()> {
        let state = State::get().await?;
        let default = state.settings.read().await.default_user;

        let users = auth::users().await?;
        let rows = users.iter().map(|user| UserRow::from(user, default));

        let table = table(rows);
        println!("{table}");

        Ok(())
    }
}

impl UserCommand {
    pub async fn run(&self, args: &crate::Args) -> Result<()> {
        dispatch!(&self.action, (args, self) => {
            UserSubcommand::Add,
            UserSubcommand::List
        })
    }
}
