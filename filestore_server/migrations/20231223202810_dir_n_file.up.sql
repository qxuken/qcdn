CREATE TABLE dir(
  id            TEXT PRIMARY       KEY,
  path          TEXT          NOT NULL,
  created_at INTEGER          NOT NULL
);

CREATE TABLE dir_file(
  id            TEXT PRIMARY       KEY,
  dir_id        TEXT          NOT NULL,
  name          TEXT          NOT NULL,
  size       INTEGER          NOT NULL,
  state      INTEGER          NOT NULL,
  file_type  INTEGER          NOT NULL,
  meta          TEXT          NOT NULL,
  created_at INTEGER          NOT NULL
);
