use crate::db::establish_connection;
use crate::graphql::{character_model::Character, Ctx};
use crate::schema::{character, location};
use diesel::{self, prelude::*, Insertable, Queryable};
use juniper::FieldResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "location"]
pub struct Location {
  id: i32,
  name: String,
  #[serde(rename = "type")]
  type_: String,
  dimension: String,
}

#[juniper::object(Context = Ctx,)]
impl Location {
  fn id(&self) -> i32 {
    self.id
  }
  fn name(&self) -> &str {
    &self.name
  }
  #[graphql(name = "type")]
  fn type_(&self) -> &str {
    &self.type_
  }
  fn dimension(&self) -> &str {
    &self.dimension
  }

  fn characters_with_origin(&self) -> FieldResult<Vec<Character>> {
    let conn = establish_connection();
    Ok(
      character::table
        .filter(character::origin_id.eq(self.id))
        .load(&conn)?,
    )
  }

  fn characters_with_location(&self) -> FieldResult<Vec<Character>> {
    let conn = establish_connection();
    Ok(
      character::table
        .filter(character::location_id.eq(self.id))
        .load(&conn)?,
    )
  }
}

#[derive(juniper::GraphQLInputObject, AsChangeset, Identifiable)]
#[changeset_options(treat_none_as_null = "true")]
#[table_name = "location"]
struct LocationUpdater {
  id: i32,
  name: String,
  #[graphql(name = "type")]
  type_: String,
  dimension: String,
}

#[derive(juniper::GraphQLInputObject, Insertable)]
#[table_name = "location"]
struct LocationCreator {
  name: String,
  #[graphql(name = "type")]
  type_: String,
  dimension: String,
}

pub struct LocationMutation;

#[juniper::object(Context= Ctx,)]
impl LocationMutation {
  pub fn create_location(creator: LocationCreator, context: &Ctx) -> FieldResult<Location> {
    let db_conn = establish_connection();
    let ans: Location = diesel::insert_into(location::table)
      .values(creator)
      .get_result(&db_conn)?;
    *context.location.write().unwrap() += 1;
    Ok(ans)
  }

  pub fn delete_location(id: i32, context: &Ctx) -> FieldResult<bool> {
    let conn = establish_connection();
    conn.transaction(|| {
      diesel::update(character::table.filter(character::origin_id.eq(id)))
        .set(character::origin_id.eq(None::<i32> {}))
        .execute(&conn)?;
      diesel::update(character::table.filter(character::location_id.eq(id)))
        .set(character::location_id.eq(None::<i32> {}))
        .execute(&conn)?;
      let made_delete = diesel::delete(location::table.find(id)).execute(&conn)? == 1;
      if made_delete {
        *context.location.write().unwrap() -= 1;
      }
      Ok(made_delete)
    })
  }

  pub fn update_location(updater: LocationUpdater) -> FieldResult<Location> {
    let conn = establish_connection();
    Ok(updater.save_changes(&conn)?)
  }
}
