use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    #[default]
    LocalJson,
    Mysql,
    Postgresql,
    Mongodb,
    GoogleDrive,
    Dropbox,
}

impl BackendKind {
    pub const ALL: [Self; 6] = [
        Self::LocalJson,
        Self::Mysql,
        Self::Postgresql,
        Self::Mongodb,
        Self::GoogleDrive,
        Self::Dropbox,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::LocalJson => "local_json",
            Self::Mysql => "mysql",
            Self::Postgresql => "postgresql",
            Self::Mongodb => "mongodb",
            Self::GoogleDrive => "google_drive",
            Self::Dropbox => "dropbox",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::LocalJson => "Local JSON (por defecto)",
            Self::Mysql => "MySQL",
            Self::Postgresql => "PostgreSQL",
            Self::Mongodb => "MongoDB",
            Self::GoogleDrive => "Google Drive",
            Self::Dropbox => "Dropbox",
        }
    }

    pub fn is_database(self) -> bool {
        matches!(self, Self::Mysql | Self::Postgresql | Self::Mongodb)
    }

    pub fn is_cloud(self) -> bool {
        matches!(self, Self::GoogleDrive | Self::Dropbox)
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.key() == key)
    }
}
