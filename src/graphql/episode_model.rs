use crate::db::establish_connection;
use crate::graphql::{character_model::Character, Ctx};
use crate::schema::{character, character_episode, episode};
use diesel::{self, prelude::*, Insertable, Queryable};
use juniper::FieldResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "episode"]
pub struct Episode {
  id: i32,
  name: String,
  air_date: String,
  code: String,
}

#[derive(Deserialize, Serialize, Insertable)]
#[table_name = "character_episode"]
pub struct CharacterEpisode {
  pub character_id: i32,
  pub episode_id: i32,
}

#[juniper::object(
  Context = Ctx,
)]
impl Episode {
  fn id(&self) -> i32 {
    self.id
  }
  fn name(&self) -> &str {
    &self.name
  }
  fn air_date(&self) -> &str {
    &self.air_date
  }
  fn code(&self) -> &str {
    &self.code
  }

  fn characters(&self) -> FieldResult<Vec<Character>> {
    let conn = establish_connection();
    Ok(
      character_episode::table
        .inner_join(character::table)
        .inner_join(episode::table)
        .filter(episode::id.eq(self.id))
        .select(character::all_columns)
        .get_results(&conn)?,
    )
  }
}

#[derive(juniper::GraphQLInputObject, AsChangeset, Identifiable)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "episode"]
pub struct EpisodeUpdater {
  id: i32,
  name: String,
  air_date: String,
  code: String,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "episode"]
struct EpisodeCreator {
  name: String,
  air_date: String,
  code: String,
}

pub struct EpisodeMutation;

#[juniper::object(Context= Ctx,)]
impl EpisodeMutation {
  pub fn create_episode(creator: EpisodeCreator, context: &Ctx) -> FieldResult<Episode> {
    let db_conn = establish_connection();
    let ans: Episode = diesel::insert_into(episode::table)
      .values(creator)
      .get_result(&db_conn)?;
    *context.episode.write().unwrap() += 1;
    Ok(ans)
  }

  pub fn delete_episode(id: i32, context: &Ctx) -> FieldResult<bool> {
    let conn = establish_connection();
    conn.transaction(|| {
      diesel::delete(character_episode::table.filter(character_episode::episode_id.eq(id)))
        .execute(&conn)?;
      let made_delete = diesel::delete(episode::table.find(id)).execute(&conn)? == 1;
      if made_delete {
        *context.episode.write().unwrap() -= 1;
      }
      Ok(made_delete)
    })
  }

  pub fn update_episode(updater: EpisodeUpdater) -> FieldResult<Episode> {
    let conn = establish_connection();
    Ok(updater.save_changes(&conn)?)
  }
}
