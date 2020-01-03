use crate::db::establish_connection;
use crate::graphql::{
  episode_model::{CharacterEpisode, Episode},
  location_model::Location,
  Ctx,
};
use crate::schema::{character, character_episode, episode, location};
use diesel::{self, prelude::*, Insertable, Queryable};
use juniper::FieldResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "character"]
pub struct Character {
  id: i32,
  name: String,
  status: String,
  species: String,
  gender: String,
  #[serde(rename = "type")]
  type_: Option<String>,
  origin_id: Option<i32>,
  location_id: Option<i32>,
}

#[juniper::object(
  Context = Ctx,
)]
impl Character {
  fn id(&self) -> i32 {
    self.id
  }
  fn name(&self) -> &str {
    &self.name
  }
  fn status(&self) -> &str {
    &self.status
  }
  fn species(&self) -> &str {
    &self.species
  }
  fn gender(&self) -> &str {
    &self.gender
  }
  fn origin_id(&self) -> &Option<i32> {
    &self.origin_id
  }
  fn location_id(&self) -> &Option<i32> {
    &self.location_id
  }
  #[graphql(name = "type")]
  fn type_(&self) -> &Option<String> {
    &self.type_
  }

  fn origin(&self) -> Option<Location> {
    let conn = establish_connection();
    match self.origin_id {
      Some(origin_id) => Some(location::table.find(origin_id).get_result(&conn).unwrap()),
      None => None,
    }
  }

  fn location(&self) -> Option<Location> {
    let conn = establish_connection();
    match self.location_id {
      Some(location_id) => Some(location::table.find(location_id).get_result(&conn).unwrap()),
      None => None,
    }
  }

  fn episodes(&self) -> FieldResult<Vec<Episode>> {
    let conn = establish_connection();
    Ok(
      character_episode::table
        .inner_join(character::table)
        .inner_join(episode::table)
        .filter(character::id.eq(self.id))
        .select(episode::all_columns)
        .get_results(&conn)?,
    )
  }
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "character"]
pub struct CharacterCreator {
  name: String,
  status: String,
  species: String,
  gender: String,
  #[graphql(name = "type")]
  type_: Option<String>,
  origin_id: Option<i32>,
  location_id: Option<i32>,
}

#[derive(juniper::GraphQLInputObject, AsChangeset, Identifiable)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "character"]
pub struct CharacterUpdater {
  id: i32,
  name: String,
  status: String,
  species: String,
  gender: String,
  #[graphql(name = "type")]
  pub type_: Option<String>,
  pub origin_id: Option<i32>,
  pub location_id: Option<i32>,
}

#[derive(juniper::GraphQLInputObject)]
pub struct CharacterRelations {
  episode_ids: Vec<i32>,
}

fn insert_character_relations(
  id: i32,
  relations: CharacterRelations,
  conn: &PgConnection,
) -> diesel::result::QueryResult<()> {
  let values: Vec<CharacterEpisode> = relations
    .episode_ids
    .iter()
    .map(|episode_id| CharacterEpisode {
      episode_id: *episode_id,
      character_id: id,
    })
    .collect();
  diesel::insert_into(character_episode::table)
    .values(&values)
    .execute(conn)?;
  Ok(())
}
pub struct CaracterMutation;

#[juniper::object(
  Context = Ctx,
)]
impl CaracterMutation {
  pub fn create_character(
    creator: CharacterCreator,
    relations: CharacterRelations,
    context: &Ctx,
  ) -> FieldResult<Character> {
    let db_conn = establish_connection();
    Ok(
      db_conn.transaction::<Character, diesel::result::Error, _>(|| {
        let ans: Character = diesel::insert_into(character::table)
          .values(creator)
          .get_result(&db_conn)?;
        if relations.episode_ids.len() > 0 {
          insert_character_relations(ans.id, relations, &db_conn)?;
        }
        *context.character.write().unwrap() += 1;
        Ok(ans)
      })?,
    )
  }

  pub fn delete_character(id: i32, context: &Ctx) -> FieldResult<bool> {
    let conn = establish_connection();
    conn.transaction(|| {
      diesel::delete(character_episode::table.filter(character_episode::character_id.eq(id)))
        .execute(&conn)?;
      let made_delete = diesel::delete(character::table.find(id)).execute(&conn)? == 1;
      if made_delete {
        *context.character.write().unwrap() -= 1;
      }
      Ok(made_delete)
    })
  }

  pub fn update_character(
    mut updater: CharacterUpdater,
    relations: Option<CharacterRelations>,
  ) -> FieldResult<Character> {
    let conn = establish_connection();
    // if let Some(Some(v)) = &updater.location_id {
    //   if *v < 0 {
    //     updater.location_id = Some(None);
    //   }
    // }
    // if let Some(Some(v)) = &updater.origin_id {
    //   if *v < 0 {
    //     updater.origin_id = Some(None);
    //   }
    // }
    // if let Some(Some(v)) = &updater.type_ {
    //   if v == "_null" {
    //     updater.type_ = Some(None);
    //   }
    // }
    Ok(conn.transaction::<Character, diesel::result::Error, _>(|| {
      if let Some(relations) = relations {
        diesel::delete(character_episode::table)
          .filter(character_episode::character_id.eq(updater.id))
          .execute(&conn)?;
        insert_character_relations(updater.id, relations, &conn)?;
      }
      Ok(updater.save_changes(&conn)?)
    })?)
  }
}
