use super::schema::{Column, Table};
use super::value::Kind;

#[derive(Clone, Copy, PartialEq)]
pub enum Dialect {
    MySql,
    Postgres,
}

impl Dialect {
    pub fn placeholder(self, position: usize) -> String {
        match self {
            Self::MySql => "?".to_owned(),
            Self::Postgres => format!("${position}"),
        }
    }

    pub fn placeholders(self, count: usize) -> String {
        (1..=count)
            .map(|position| self.placeholder(position))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn insert(self, table: &Table) -> String {
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            table.name,
            table.column_list(),
            self.placeholders(table.columns.len())
        )
    }

    pub fn upsert(self, table: &Table) -> String {
        let changes = table
            .columns
            .iter()
            .filter(|column| column.name != table.primary_key)
            .map(|column| match self {
                Self::MySql => format!("{0} = VALUES({0})", column.name),
                Self::Postgres => format!("{0} = EXCLUDED.{0}", column.name),
            })
            .collect::<Vec<_>>()
            .join(", ");
        let clause = match self {
            Self::MySql => format!("ON DUPLICATE KEY UPDATE {changes}"),
            Self::Postgres => {
                format!(
                    "ON CONFLICT ({}) DO UPDATE SET {changes}",
                    table.primary_key
                )
            }
        };
        format!("{} {clause}", self.insert(table))
    }

    pub fn create(self, table: &Table) -> Vec<String> {
        let mut parts: Vec<String> = table
            .columns
            .iter()
            .map(|column| format!("{} {}", column.name, self.column_type(column.kind)))
            .collect();
        parts.push(format!("PRIMARY KEY ({})", table.primary_key));
        if let Some(parent) = table.parent {
            parts.push(format!(
                "FOREIGN KEY ({}) REFERENCES {parent} (id) ON DELETE CASCADE",
                table.columns[0].name
            ));
        }
        if self == Self::MySql {
            parts.extend(table.indexes.iter().map(|index| {
                let unique = if index.unique { "UNIQUE " } else { "" };
                format!("{unique}INDEX ({})", index.columns)
            }));
        }
        let suffix = match self {
            Self::MySql => " ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
            Self::Postgres => "",
        };
        let mut statements = vec![format!(
            "CREATE TABLE IF NOT EXISTS {} ({}){suffix}",
            table.name,
            parts.join(", ")
        )];
        if self == Self::Postgres {
            statements.extend(table.indexes.iter().map(|index| {
                let unique = if index.unique { "UNIQUE " } else { "" };
                format!(
                    "CREATE {unique}INDEX IF NOT EXISTS {0}_{1}_idx ON {0} ({1})",
                    table.name, index.columns
                )
            }));
        }
        statements
    }

    pub fn columns_query(self) -> &'static str {
        match self {
            Self::MySql => {
                "SELECT CAST(column_name AS CHAR(128)) FROM information_schema.columns \
                 WHERE table_schema = DATABASE() AND table_name = ?"
            }
            Self::Postgres => {
                "SELECT CAST(column_name AS VARCHAR(128)) FROM information_schema.columns \
                 WHERE table_schema = current_schema() AND table_name = $1"
            }
        }
    }

    pub fn add_column(self, table: &Table, column: &Column) -> String {
        let default = match column.kind {
            Kind::Id | Kind::Short | Kind::Long => " DEFAULT ('')",
            Kind::Int | Kind::Money => " DEFAULT 0",
            Kind::Bool => " DEFAULT FALSE",
            Kind::Moment => " DEFAULT CURRENT_TIMESTAMP",
            Kind::OptionalMoment => "",
        };
        format!(
            "ALTER TABLE {} ADD COLUMN {} {}{default}",
            table.name,
            column.name,
            self.column_type(column.kind)
        )
    }

    fn column_type(self, kind: Kind) -> &'static str {
        match (kind, self) {
            (Kind::Id, _) => "VARCHAR(128) NOT NULL",
            (Kind::Short, _) => "VARCHAR(255) NOT NULL",
            (Kind::Long, _) => "TEXT NOT NULL",
            (Kind::Int, _) => "BIGINT NOT NULL",
            (Kind::Bool, _) => "BOOLEAN NOT NULL",
            (Kind::Money, _) => "DECIMAL(18,4) NOT NULL",
            (Kind::Moment, Self::MySql) => "DATETIME(6) NOT NULL",
            (Kind::Moment, Self::Postgres) => "TIMESTAMP NOT NULL",
            (Kind::OptionalMoment, Self::MySql) => "DATETIME(6) NULL",
            (Kind::OptionalMoment, Self::Postgres) => "TIMESTAMP NULL",
        }
    }
}
