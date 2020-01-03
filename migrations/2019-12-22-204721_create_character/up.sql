-- Your SQL goes here
CREATE TABLE "character" (
  "id" SERIAL PRIMARY KEY,
  "name" VARCHAR NOT NULL,
  "status" VARCHAR NOT NULL DEFAULT 'unknown',
  "species" VARCHAR NOT NULL DEFAULT 'unknown',
  "gender" VARCHAR NOT NULL DEFAULT 'unknown',
  "type" VARCHAR,
  "origin_id" INT REFERENCES "location"("id"),
  "location_id" INT REFERENCES "location"("id")
);
CREATE INDEX "character_origin_id_index" ON "character"("origin_id");
CREATE INDEX "character_location_id_index" ON "character"("location_id");