use crate::graphql::{
  character_model::Character,
  episode_model::{CharacterEpisode, Episode},
  location_model::Location,
};
use crate::schema::*;
use csv;
use diesel::{
  dsl::{count_star, Select},
  pg::PgConnection,
  prelude::*,
};
use dotenv::dotenv;
use serde::de::DeserializeOwned;
use std::env;
use std::fs::File;
use std::io::BufReader;

pub fn establish_connection() -> PgConnection {
  dotenv().ok();
  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
  let connection = PgConnection::establish(&database_url).expect("Database connection failed");

  connection
}

#[derive(Debug)]
pub struct DbCounts {
  pub character: i32,
  pub location: i32,
  pub episode: i32,
}

pub fn init_db() -> Result<DbCounts, diesel::result::Error> {
  let conn = establish_connection();
  let res = get_count(character::table, &conn)?;

  if res == 0 {
    reset_db(&conn)?;
  }
  Ok(get_all_counts(&conn)?)
}

pub fn reset_db(conn: &PgConnection) -> Result<(), diesel::result::Error> {
  // let table_with_files = vec![
  //   ("locations", location::table),
  //   ("characters", character::table),
  //   ("episodes", episode::table),
  // ];
  Ok(conn.transaction::<(), diesel::result::Error, _>(|| {
    diesel::delete(character_episode::table).execute(conn)?;
    diesel::delete(episode::table).execute(conn)?;
    diesel::delete(character::table).execute(conn)?;
    diesel::delete(location::table).execute(conn)?;
    // ############  location  ################
    diesel::insert_into(location::table)
      .values(read_tsv::<Location>("raw-data/locations.tsv"))
      .execute(conn)?;
    // #############  character  ################
    diesel::insert_into(character::table)
      .values(read_tsv::<Character>("raw-data/characters.tsv"))
      .execute(conn)?;
    // ##############  episode  ################
    diesel::insert_into(episode::table)
      .values(read_tsv::<Episode>("raw-data/episodes.tsv"))
      .execute(conn)?;
    // ############  character_episode  ################
    diesel::insert_into(character_episode::table)
      .values(read_tsv::<CharacterEpisode>(
        "raw-data/character_episode_join.tsv",
      ))
      .execute(conn)?;

    for t_name in ["character", "location", "episode"].iter() {
      let query = format!(
        "SELECT setval('{}_id_seq', (SELECT MAX(\"id\") FROM \"{}\"));",
        t_name, t_name
      );
      diesel::sql_query(query).execute(conn)?;
    }
    Ok(())
  })?)
}

// fn populate_table<Table, Model>(
//   table: Table,
//   filename: &str,
//   conn: &PgConnection,
// ) -> Result<(), diesel::result::Error>
// where
//   Model: DeserializeOwned
//     + diesel::query_builder::UndecoratedInsertRecord<Table>
//     + diesel::Insertable<Table>,
//   diesel::dsl::InsertStatement<T, U::Values, Op>:
//     diesel::query_dsl::methods::ExecuteDsl<PgConnection>,
// {
//   diesel::insert_into(table)
//     .values(read_tsv::<Model>(&format!("raw-data/{}", filename)))
//     .execute(conn)?;
// }

use diesel::query_dsl::select_dsl::SelectDsl;
use diesel::query_dsl::{LoadQuery, RunQueryDsl};

fn get_count<T: SelectDsl<count_star>>(table: T, conn: &PgConnection) -> QueryResult<i64>
where
  Select<T, count_star>: LoadQuery<PgConnection, i64>,
{
  Ok(table.select(count_star()).get_result(conn)?)
}
use diesel::result::QueryResult;
pub fn get_all_counts(conn: &PgConnection) -> QueryResult<DbCounts> {
  Ok(DbCounts {
    character: get_count(character::table, &conn)? as i32,
    location: get_count(location::table, &conn)? as i32,
    episode: get_count(episode::table, &conn)? as i32,
  })
}

fn read_tsv<T: DeserializeOwned>(filename: &str) -> Vec<T> {
  let file = File::open(filename).unwrap();
  let reader = BufReader::new(file);
  let mut reader = csv::ReaderBuilder::new()
    .delimiter(b'\t')
    .from_reader(reader);
  reader.deserialize::<T>().map(|r| r.unwrap()).collect()
}
