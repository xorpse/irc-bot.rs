use super::aatxe;
use super::pkg_info;
use super::ErrorKind;
use super::Result;
use serde_yaml;
use smallvec::SmallVec;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

mod inner {
    /// Configuration structure that can be deserialized by Serde.
    ///
    /// This is hidden from the consumer because Serde won't validate the configuration.
    #[derive(Debug, Default, Deserialize)]
    pub(super) struct Config {
        pub(super) nickname: String,

        #[serde(default)]
        pub(super) username: String,

        #[serde(default)]
        pub(super) realname: String,

        #[serde(default)]
        pub(super) admins: Vec<super::Admin>,

        pub(super) servers: Vec<super::Server>,
    }
}

#[derive(Debug)]
pub struct Config {
    pub(crate) nickname: String,

    pub(crate) username: String,

    pub(crate) realname: String,

    pub(crate) admins: SmallVec<[Admin; 8]>,

    pub(crate) servers: SmallVec<[Arc<aatxe::Config>; 8]>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Admin {
    #[serde(default)]
    pub nick: Option<String>,

    #[serde(default)]
    pub user: Option<String>,

    #[serde(default)]
    pub host: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Server {
    pub host: String,

    pub port: u16,

    #[serde(default = "mk_true")]
    pub tls: bool,

    #[serde(default)]
    pub channels: Vec<String>,
}

#[derive(Debug)]
pub struct ConfigBuilder(Result<inner::Config>);

impl Config {
    pub fn try_from<T>(input: T) -> Result<Config>
    where
        T: IntoConfig,
    {
        input.into_config()
    }

    pub fn try_from_path<P>(path: P) -> Result<Config>
    where
        P: AsRef<Path>,
    {
        Self::try_from(File::open(path)?)
    }

    pub fn build() -> ConfigBuilder {
        ConfigBuilder(Ok(Default::default()))
    }
}

impl ConfigBuilder {
    pub fn nickname<S>(self, nickname: S) -> Self
    where
        S: Into<String>,
    {
        let nickname = nickname.into();

        if nickname.is_empty() {
            return ConfigBuilder(
                Err(ErrorKind::Config("nickname".into(), "is empty".into()).into()),
            );
        }

        ConfigBuilder(self.0.map(|cfg| inner::Config { nickname, ..cfg }))
    }

    pub fn username<S>(self, username: S) -> Self
    where
        S: Into<String>,
    {
        ConfigBuilder(self.0.map(|cfg| inner::Config {
            username: username.into(),
            ..cfg
        }))
    }

    pub fn realname<S>(self, realname: S) -> Self
    where
        S: Into<String>,
    {
        ConfigBuilder(self.0.map(|cfg| inner::Config {
            realname: realname.into(),
            ..cfg
        }))
    }
}

// TODO: Switch to `TryFrom` once rustc 1.18 is stable.
pub trait IntoConfig {
    fn into_config(self) -> Result<Config>;
}

impl IntoConfig for Config {
    fn into_config(self) -> Result<Config> {
        Ok(self)
    }
}

impl IntoConfig for Result<Config> {
    fn into_config(self) -> Result<Config> {
        self
    }
}

impl IntoConfig for ConfigBuilder {
    fn into_config(self) -> Result<Config> {
        self.0.and_then(cook_config)
    }
}

impl<'a> IntoConfig for &'a str {
    fn into_config(self) -> Result<Config> {
        read_config(self)
    }
}

impl IntoConfig for String {
    fn into_config(self) -> Result<Config> {
        read_config(&self)
    }
}

impl<R> IntoConfig for BufReader<R>
where
    R: Read,
{
    fn into_config(mut self) -> Result<Config> {
        let mut text = String::new();
        self.read_to_string(&mut text)?;
        text.into_config()
    }
}

impl IntoConfig for File {
    fn into_config(self) -> Result<Config> {
        BufReader::new(self).into_config()
    }
}

fn read_config(input: &str) -> Result<Config> {
    serde_yaml::from_str(input)
        .map_err(Into::into)
        .and_then(cook_config)
}

fn cook_config(mut cfg: inner::Config) -> Result<Config> {
    validate_config(&cfg)?;

    fill_in_config_defaults(&mut cfg)?;

    let nickname = cfg.nickname.to_owned();

    let username = cfg.username.to_owned();

    let realname = cfg.realname.to_owned();

    let admins = cfg.admins.drain(..).collect();

    let servers = cfg.servers
        .drain(..)
        .map(|server_cfg| {
            Arc::new(aatxe::Config {
                // TODO: Allow nickname etc. to be configured per-server.
                nickname: Some(nickname.clone()),
                username: Some(username.clone()),
                realname: Some(realname.clone()),
                server: Some(server_cfg.host),
                port: Some(server_cfg.port),
                use_ssl: Some(server_cfg.tls),
                channels: Some(server_cfg.channels),
                ..Default::default()
            })
        })
        .collect();

    Ok(Config {
        nickname,
        username,
        realname,
        admins,
        servers,
    })
}

fn validate_config(cfg: &inner::Config) -> Result<()> {
    ensure!(
        !cfg.nickname.is_empty(),
        ErrorKind::Config("nickname".into(), "is empty".into())
    );

    ensure!(
        !cfg.servers.is_empty(),
        ErrorKind::Config("servers".into(), "is empty".into())
    );

    ensure!(
        cfg.servers.len() == 1,
        ErrorKind::Config(
            "servers".into(),
            "lists multiple servers, which is not yet supported".into(),
        )
    );

    Ok(())
}

fn fill_in_config_defaults(cfg: &mut inner::Config) -> Result<()> {
    if cfg.username.is_empty() {
        cfg.username = cfg.nickname.clone();
    }

    if cfg.realname.is_empty() {
        cfg.realname = pkg_info::BRIEF_CREDITS_STRING.clone();
    }

    Ok(())
}

fn mk_true() -> bool {
    true
}
