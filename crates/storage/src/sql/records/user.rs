use pos_domain::{Permission, Role, User};

use crate::sql::schema::{column, Column, Index, SqlRecord, Table};
use crate::sql::value::{Kind, Row, SqlValue};
use crate::Result;

const COLUMNS: &[Column] = &[
    column("id", Kind::Id),
    column("username", Kind::Short),
    column("password_hash", Kind::Short),
    column("full_name", Kind::Short),
    column("role", Kind::Short),
    column("is_default", Kind::Bool),
    column("active", Kind::Bool),
    column("last_login", Kind::OptionalMoment),
    column("created_at", Kind::Moment),
    column("updated_at", Kind::Moment),
];

const PERMISSION_COLUMNS: &[Column] = &[
    column("user_id", Kind::Id),
    column("position", Kind::Int),
    column("permission", Kind::Short),
];

impl SqlRecord for User {
    const TABLE: Table = Table {
        name: "pos_users",
        columns: COLUMNS,
        primary_key: "id",
        parent: None,
        indexes: &[Index {
            columns: "username",
            unique: true,
        }],
    };

    const CHILD: Option<Table> = Some(Table {
        name: "pos_user_permissions",
        columns: PERMISSION_COLUMNS,
        primary_key: "user_id, position",
        parent: Some("pos_users"),
        indexes: &[],
    });

    fn values(&self) -> Vec<SqlValue> {
        vec![
            (&self.id).into(),
            (&self.username).into(),
            (&self.password_hash).into(),
            (&self.full_name).into(),
            self.role.key().into(),
            self.is_default.into(),
            self.active.into(),
            self.last_login.into(),
            self.created_at.into(),
            self.updated_at.into(),
        ]
    }

    fn from_row(row: &mut Row) -> Result<Self> {
        Ok(Self {
            id: row.text()?,
            username: row.text()?,
            password_hash: row.text()?,
            full_name: row.text()?,
            role: Role::from_key(&row.text()?),
            permissions: Vec::new(),
            is_default: row.bool()?,
            active: row.bool()?,
            last_login: row.optional_moment()?,
            created_at: row.moment()?,
            updated_at: row.moment()?,
        })
    }

    fn child_values(&self) -> Vec<Vec<SqlValue>> {
        self.permissions
            .iter()
            .enumerate()
            .map(|(position, permission)| {
                vec![
                    (&self.id).into(),
                    (position as i64).into(),
                    permission.key().into(),
                ]
            })
            .collect()
    }

    fn attach_child(&mut self, row: &mut Row) -> Result<()> {
        row.int()?;
        if let Some(permission) = Permission::from_key(&row.text()?) {
            self.permissions.push(permission);
        }
        Ok(())
    }
}
