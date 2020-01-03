-- Your SQL goes here
CREATE TABLE "location" (
  "id" SERIAL PRIMARY KEY,
  "name" VARCHAR NOT NULL,
  "type" VARCHAR NOT NULL DEFAULT 'unknown',
  "dimension" VARCHAR NOT NULL DEFAULT 'unknown'
);