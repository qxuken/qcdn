CREATE TABLE file(
  id                  BLOB PRIMARY KEY NOT NULL,
  dir                 TEXT             NOT NULL,
  name                TEXT             NOT NULL,
  file_type        INTEGER             NOT NULL,
  created_at      DATETIME             NOT NULL,
  UNIQUE (dir, name)
);

CREATE TABLE file_version(
  id              TEXT PRIMARY KEY  NOT NULL,
  file_id         TEXT              NOT NULL,
  size         INTEGER              NOT NULL,
  version         TEXT              NOT NULL,
  state        INTEGER              NOT NULL,
  created_at  DATETIME              NOT NULL,
  updated_at  DATETIME              NOT NULL,
  deleted_at  DATETIME                      ,
  FOREIGN KEY (file_id) REFERENCES file (id),
  UNIQUE (file_id, version, deleted_at)
);

CREATE TABLE file_latest_version(
  id                  TEXT PRIMARY KEY     NOT NULL,
  file_id             TEXT                 NOT NULL,
  file_version_id     TEXT                 NOT NULL,
  created_at       DATETIME                NOT NULL,
  expired_at       DATETIME                        ,
  FOREIGN KEY (file_id)  REFERENCES file (id),
  FOREIGN KEY (file_version_id)  REFERENCES file_version (id),
  UNIQUE (file_id, file_version_id, expired_at)
);
