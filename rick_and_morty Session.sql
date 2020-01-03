select
  min(id)
from character;


select
  "id",
  "name",
  "type"
from "character"
where
  "search_text" @@ to_tsquery('robot');

  select
  "id",
  "name",
  "type"
from "character"
where
  "search_text" @@ plainto_tsquery('robot')
  order by ts_rank("search_text", to_tsquery('robot')) desc
  limit (20);


select distinct species from character;
select distinct "status" from character;
select distinct gender from character;