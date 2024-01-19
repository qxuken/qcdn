CREATE TABLE file(
  id              TEXT PRIMARY KEY NOT NULL,
  directory_path  TEXT             NOT NULL,
  name            TEXT             NOT NULL,
  file_type    INTEGER             NOT NULL,
  version         TEXT             NOT NULL,
  size         INTEGER             NOT NULL,
  state        INTEGER             NOT NULL,
  meta            TEXT             NOT NULL,
  created_at  DATETIME             NOT NULL,
);
