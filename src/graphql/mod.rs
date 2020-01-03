use crate::db::{self, establish_connection};
use crate::schema::{character, episode, location};
use diesel::{
  dsl::sql,
  pg::Pg,
  prelude::*,
  sql_types::{Bool, Float},
};
use juniper::FieldResult;
use std::sync::RwLock;

pub mod character_model;
use character_model::*;
pub mod episode_model;
use episode_model::*;
pub mod location_model;
use location_model::*;

// ######### CONTEXT ###############
pub struct Ctx {
  character: RwLock<i32>,
  location: RwLock<i32>,
  episode: RwLock<i32>,
}
impl juniper::Context for Ctx {}
impl Ctx {
  pub fn new(counts: db::DbCounts) -> Ctx {
    Ctx {
      character: RwLock::from(counts.character),
      location: RwLock::from(counts.location),
      episode: RwLock::from(counts.episode),
    }
  }
}

// ######### QUERIES ###############

#[derive(juniper::GraphQLInputObject)]
struct CharacterFilter {
  search_text: Option<String>,
  status: Option<Vec<String>>,
  species: Option<Vec<String>>,
  gender: Option<Vec<String>>,
  origin_id: Option<Vec<i32>>,
  location_id: Option<Vec<i32>>,
}

type FilterCharacterExpr<'a> =
  Box<dyn BoxableExpression<character::table, Pg, SqlType = Bool> + 'a>;

fn filter_characters<'a>(filter: &'a CharacterFilter) -> FilterCharacterExpr<'a> {
  let mut filters: Vec<FilterCharacterExpr> = vec![];

  let always_true = Box::new(character::id.eq(character::id));

  if let Some(status) = &filter.status {
    filters.push(Box::new(character::status.eq_any(status)));
  }
  if let Some(species) = &filter.species {
    filters.push(Box::new(character::species.eq_any(species)));
  }
  if let Some(gender) = &filter.gender {
    filters.push(Box::new(character::gender.eq_any(gender)));
  }
  if let Some(origin_id) = &filter.origin_id {
    filters.push(Box::new(character::origin_id.eq_any(origin_id)));
  }
  if let Some(location_id) = &filter.location_id {
    filters.push(Box::new(character::location_id.eq_any(location_id)));
  }
  filters
    .into_iter()
    .fold(always_true, |query, curr| Box::new(query.and(curr)))
}

pub struct Query;
#[juniper::object(
    Context = Ctx,
)]
impl Query {
  fn characters(mut page: i32, context: &Ctx) -> FieldResult<ListResult<Character>> {
    if page == -1 {
      page = 1;
    }
    Ok(load_many(character::table, page, &context.character)?)
  }

  fn characters_filtered(
    limit: i32,
    offset: i32,
    filter: CharacterFilter,
  ) -> FieldResult<Vec<Character>> {
    let query = character::table.filter(filter_characters(&filter));
    match &filter.search_text {
      Some(search_text) => {
        let raw_filter = format!("\"search_text\" @@ plainto_tsquery('{}')", search_text);
        let raw_order = format!(
          "ts_rank(\"search_text\", plainto_tsquery('{}')) desc",
          search_text
        );
        let query = query.filter(sql(&raw_filter)).order(sql::<
          Box<dyn diesel::expression::Expression<SqlType = Float>>,
        >(&raw_order));
        let query = diesel::QueryDsl::limit(query, limit as i64);
        Ok(diesel::QueryDsl::offset(query, offset as i64).load(&establish_connection())?)
      }
      None => {
        let query = diesel::QueryDsl::limit(query, limit as i64);
        Ok(diesel::QueryDsl::offset(query, offset as i64).load(&establish_connection())?)
      }
    }
  }

  fn character(id: i32) -> FieldResult<Character> {
    let db_conn = establish_connection();
    Ok(character::table.find(id).first(&db_conn)?)
  }

  fn episodes(page: i32, context: &Ctx) -> FieldResult<ListResult<Episode>> {
    Ok(load_many(episode::table, page, &context.episode)?)
  }

  fn episode(id: i32) -> FieldResult<Episode> {
    let db_conn = establish_connection();
    Ok(episode::table.find(id).first(&db_conn)?)
  }

  fn locations(page: i32, context: &Ctx) -> FieldResult<ListResult<Location>> {
    Ok(load_many(location::table, page, &context.location)?)
  }

  fn location(id: i32) -> FieldResult<Location> {
    let db_conn = establish_connection();
    Ok(location::table.find(id).first(&db_conn)?)
  }
}

use diesel::dsl::{Limit, Offset};
use diesel::query_dsl::{
  methods::{LimitDsl, OffsetDsl},
  LoadQuery, RunQueryDsl,
};

#[derive(juniper::GraphQLObject)]
struct InfoListResult {
  next_page: Option<i32>,
  num_pages: i32,
  item_count: i32,
}

struct ListResult<Model> {
  info: InfoListResult,
  results: Vec<Model>,
}

#[juniper::object(name = "CharacterListResult", Context = Ctx,)]
impl ListResult<Character> {
  fn info(&self) -> &InfoListResult {
    &self.info
  }
  fn results(&self) -> &Vec<Character> {
    &self.results
  }
}
#[juniper::object(name = "LocationListResult", Context = Ctx,)]
impl ListResult<Location> {
  fn info(&self) -> &InfoListResult {
    &self.info
  }
  fn results(&self) -> &Vec<Location> {
    &self.results
  }
}
#[juniper::object(name = "EpisodeListResult", Context = Ctx,)]
impl ListResult<Episode> {
  fn info(&self) -> &InfoListResult {
    &self.info
  }
  fn results(&self) -> &Vec<Episode> {
    &self.results
  }
}

const ITEMS_PER_PAGE: i32 = 30;

fn load_many<Model, Table>(
  table: Table,
  page_input: i32,
  count: &RwLock<i32>,
) -> FieldResult<ListResult<Model>>
where
  Table: OffsetDsl + LoadQuery<diesel::pg::PgConnection, Model>,
  Offset<Table>: LimitDsl,
  Limit<Offset<Table>>: LoadQuery<diesel::pg::PgConnection, Model>,
{
  let page = std::cmp::max(page_input, 1);
  let item_count = *count.read().unwrap();
  let offset = ITEMS_PER_PAGE * (page - 1);
  let num_pages = f64::ceil(item_count as f64 / (ITEMS_PER_PAGE as f64)) as i32;

  let info = InfoListResult {
    next_page: if item_count > offset + ITEMS_PER_PAGE {
      Some(page + 1)
    } else {
      None
    },
    num_pages,
    item_count: item_count,
  };

  let results = if page_input == -1 {
    let db_conn = establish_connection();
    table.load(&db_conn)?
  } else if item_count > offset {
    let db_conn = establish_connection();
    table
      .offset(offset as i64)
      .limit(ITEMS_PER_PAGE as i64)
      .load::<Model>(&db_conn)?
  } else {
    vec![]
  };
  Ok(ListResult { info, results })
}

// ########## MUTATIONS ###############

pub struct Mutation;
#[juniper::object(
    Context = Ctx,
)]
impl Mutation {
  fn reset_db(context: &Ctx) -> FieldResult<bool> {
    let db_conn = establish_connection();
    db::reset_db(&db_conn)?;
    db_conn.transaction::<(), diesel::result::Error, _>(|| {
      let counts = db::get_all_counts(&db_conn)?;
      *context.character.write().unwrap() = counts.character;
      *context.episode.write().unwrap() = counts.episode;
      *context.location.write().unwrap() = counts.location;
      Ok(())
    })?;
    Ok(true)
  }

  fn character_mutation() -> CaracterMutation {
    CaracterMutation
  }

  fn episode_mutation() -> EpisodeMutation {
    EpisodeMutation
  }

  fn location_mutation() -> LocationMutation {
    LocationMutation
  }

  // fn create_character(
  //   creator: CharacterCreator,
  //   relations: CharacterRelations,
  //   context: &Ctx,
  // ) -> FieldResult<Character> {
  //   CaracterMutation::create_character(creator, relations, context)
  // }

  // fn update_character(mut updater: CharacterUpdater) -> FieldResult<Character> {
  //   CaracterMutation::update_character(updater)
  // }

  // fn delete_character(id: i32, context: &Ctx) -> FieldResult<bool> {
  //   CaracterMutation::delete_character(id, context)
  // }
}

pub type GraphqlSchema = juniper::RootNode<'static, Query, Mutation>;

pub fn create_schema() -> GraphqlSchema {
  GraphqlSchema::new(Query, Mutation)
}

// impl Ctx {
//   pub fn new(characters: Vec<Character>) -> Ctx {
//     Ctx(Database {
//       characters: RwLock::new(characters),
//     })
//   }
// }

// struct Database {
//   characters: RwLock<Vec<Character>>,
// }
// impl Database {
//   fn get_characters(&self) -> Vec<Character> {
//     let g = self.characters.read().unwrap();
//     (*g).to_vec()
//   }

//   fn add_character(
//     &self,
//     new_character: NewCharacter,
//   ) -> Result<Character, sync::PoisonError<sync::RwLockWriteGuard<'_, Vec<Character>>>> {
//     let characters = &mut *(self.characters.write()?);
//     let id = characters.len() + 1;
//     let character = Character::from_new_character(id, new_character);
//     characters.push(character.clone());
//     Ok(character)
//   }
// }
