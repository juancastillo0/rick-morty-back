-- Your SQL goes here
CREATE TABLE "character_episode" (
  "character_id" INT REFERENCES "character"("id"),
  "episode_id" INT REFERENCES "episode"("id"),
  PRIMARY KEY ("character_id", "episode_id")
);
CREATE INDEX "character_episode_episode_id_index" ON "character_episode"("episode_id");