table! {
    character (id) {
        id -> Int4,
        name -> Varchar,
        status -> Varchar,
        species -> Varchar,
        gender -> Varchar,
        #[sql_name = "type"]
        type_ -> Nullable<Varchar>,
        origin_id -> Nullable<Int4>,
        location_id -> Nullable<Int4>,
    }
}

table! {
    character_episode (character_id, episode_id) {
        character_id -> Int4,
        episode_id -> Int4,
    }
}

table! {
    episode (id) {
        id -> Int4,
        name -> Varchar,
        air_date -> Varchar,
        code -> Varchar,
    }
}

table! {
    location (id) {
        id -> Int4,
        name -> Varchar,
        #[sql_name = "type"]
        type_ -> Varchar,
        dimension -> Varchar,
    }
}

joinable!(character_episode -> character (character_id));
joinable!(character_episode -> episode (episode_id));

allow_tables_to_appear_in_same_query!(
    character,
    character_episode,
    episode,
    location,
);
