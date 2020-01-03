-- Your SQL goes here
CREATE TABLE "episode" (
  "id" SERIAL PRIMARY KEY,
  "name" VARCHAR NOT NULL,
  "air_date" VARCHAR NOT NULL,
  "code" VARCHAR NOT NULL 
);