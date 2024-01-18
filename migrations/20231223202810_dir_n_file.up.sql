CREATE TABLE dir(
  id              TEXT PRIMARY KEY NOT NULL,
  path            TEXT             NOT NULL,
  created_at  DATETIME             NOT NULL
);

CREATE TABLE dir_file(
  id              TEXT PRIMARY KEY NOT NULL,
  dir_id          TEXT             NOT NULL,
  name            TEXT             NOT NULL,
  file_type    INTEGER             NOT NULL,
  version         TEXT             NOT NULL,
  size         INTEGER             NOT NULL,
  state        INTEGER             NOT NULL,
  meta            TEXT             NOT NULL,
  created_at  DATETIME             NOT NULL,
  FOREIGN KEY(dir_id) REFERENCES dir(id)
);
