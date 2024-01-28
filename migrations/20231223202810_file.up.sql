CREATE TABLE dir(
  id                  BLOB PRIMARY KEY NOT NULL,
  name                TEXT             NOT NULL,
  created_at      DATETIME             NOT NULL,
  UNIQUE (name)
);

CREATE TABLE file(
  id                  BLOB PRIMARY KEY NOT NULL,
  dir_id              TEXT             NOT NULL,
  name                TEXT             NOT NULL,
  file_type        INTEGER             NOT NULL,
  created_at      DATETIME             NOT NULL,
  FOREIGN KEY (dir_id) REFERENCES dir(id),
  UNIQUE (dir_id, name, file_type)
);

CREATE TABLE file_version(
  id              TEXT PRIMARY KEY  NOT NULL,
  file_id         TEXT              NOT NULL,
  size         INTEGER              NOT NULL,
  version         TEXT              NOT NULL,
  state        INTEGER              NOT NULL,
  created_at  DATETIME              NOT NULL,
  deleted_at  DATETIME                      ,
  FOREIGN KEY (file_id) REFERENCES file(id)
);

CREATE TABLE file_version_tag(
  id                  TEXT PRIMARY KEY     NOT NULL,
  file_version_id     TEXT                 NOT NULL,
  name                TEXT                 NOT NULL,
  created_at      DATETIME                 NOT NULL,
  activated_at    DATETIME                 NOT NULL,
  FOREIGN KEY (file_version_id)  REFERENCES file_version(id),
  UNIQUE (file_version_id, name)
);
