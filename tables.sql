CREATE TABLE location (
  id INT PRIMARY KEY,
  name VARCHAR NOT NULL,
  type VARCHAR NOT NULL DEFAULT 'unknown',
  dimension VARCHAR NOT NULL DEFAULT 'unknown',
);
CREATE TABLE character (
  id INT PRIMARY KEY,
  name VARCHAR NOT NULL,
  status VARCHAR NOT NULL DEFAULT 'unknown',
  species VARCHAR NOT NULL DEFAULT 'unknown',
  gender VARCHAR NOT NULL DEFAULT 'unknown',
  type VARCHAR,
  origin_id INT REFERENCES location(id),
  location_id INT REFERENCES location(id),
);
CREATE TABLE episode (
  id INT PRIMARY KEY,
  name VARCHAR NOT NULL,
  air_date VARCHAR NOT NULL,
  episode VARCHAR NOT NULL
) CREATE TABLE character_episode (
  character_id INT PRIMARY KEY REFERENCES characte(id),
  episode_id INT PRIMARY KEY REFERENCES episode(id),
)