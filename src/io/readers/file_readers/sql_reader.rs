pub mod frames;
pub mod metadata;
pub mod pasef_frame_msms;
pub mod precursors;

use std::{collections::HashMap, path::Path};

use rusqlite::Connection;

#[derive(Debug)]
pub struct SqlReader {
    connection: Connection,
}

impl SqlReader {
    pub fn open(file_name: impl AsRef<Path>) -> Result<Self, SqlError> {
        let path = file_name.as_ref().to_path_buf();
        let connection = Connection::open(&path)?;
        Ok(Self { connection })
    }

    pub fn read_column_from_table<T: rusqlite::types::FromSql + Default>(
        &self,
        column_name: &str,
        table_name: &str,
    ) -> Result<Vec<T>, SqlError> {
        let query = format!("SELECT {} FROM {}", column_name, table_name);
        let mut stmt = self.connection.prepare(&query)?;
        let rows = stmt.query_map([], |row| match row.get::<usize, T>(0) {
            Ok(value) => Ok(value),
            _ => Ok(T::default()),
        })?;
        let result = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }
}

pub trait ReadableSqlTable {
    fn get_sql_query() -> String;

    fn from_sql_row(row: &rusqlite::Row) -> Self;

    fn from_sql_reader(reader: &SqlReader) -> Result<Vec<Self>, SqlError>
    where
        Self: Sized,
    {
        let query = Self::get_sql_query();
        let mut stmt = reader.connection.prepare(&query)?;
        let rows = stmt.query_map([], |row| Ok(Self::from_sql_row(row)))?;
        let result = rows.collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }
}

pub trait ReadableSqlHashMap {
    fn get_sql_query() -> String;

    fn from_sql_reader(
        reader: &SqlReader,
    ) -> Result<HashMap<String, String>, SqlError>
    where
        Self: Sized,
    {
        let query = Self::get_sql_query();
        let mut stmt = reader.connection.prepare(&query)?;
        let mut result = HashMap::new();
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            result.insert(key, value);
            Ok(())
        })?;
        rows.collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("SqlError: {0}")]
pub struct SqlError(#[from] rusqlite::Error);
