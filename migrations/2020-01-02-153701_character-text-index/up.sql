-- Your SQL goes here
ALTER TABLE "character"
ADD COLUMN "search_text" tsvector NOT NULL DEFAULT to_tsvector('');
--
UPDATE "character"
SET "search_text" = setweight(to_tsvector("name"), 'A') || 
      setweight(to_tsvector(coalesce("type", '')), 'B');
--
  CREATE INDEX "character_search_text_index" 
  ON "character" USING GIN ("search_text");
--
  CREATE FUNCTION character_search_text_update() 
  RETURNS trigger AS $$ 
begin 
  new."search_text" := setweight(to_tsvector(new."name"), 'A') || 
          setweight(to_tsvector(coalesce(new."type",  '')), 'B');
  return new;
end
$$ LANGUAGE plpgsql;

--
CREATE TRIGGER "character_search_text_update_trigger" 
BEFORE INSERT OR UPDATE ON "character" 
FOR EACH ROW EXECUTE PROCEDURE character_search_text_update();