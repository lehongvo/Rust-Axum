use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};

use crate::error::AppError;

pub async fn run(db: &DatabaseConnection) -> Result<(), AppError> {
    let sql = include_str!("../migrations/0001_init.sql");
    for stmt in sql.split(';') {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        db.execute(Statement::from_string(
            DatabaseBackend::Postgres,
            format!("{trimmed};"),
        ))
        .await?;
    }
    Ok(())
}
