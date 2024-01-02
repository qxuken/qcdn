# QXuken's CDN

Application for distributet content delivery

## Setup

1. Setup `env` variables

    ```bash
    cp .env.development .env
    ```

2. Install `sqlx-cli`

    ```bash
    cargo install sqlx-cli
    ```

## Development

### Migrations

- **Create** migration

  ```bash
  sqlx migrate add <name>
  ```

  > Add `-r` flag for a reversible migration
  > Add `--source ./filestore_server/migrations` arg to create migrations in a different directory

  Example:

  ```bash
  sqlx migrate add --source ./filestore_server/migrations -r add_users_table
  ```

- **Check** migration

  ```bash
  sqlx migrate info
  ```

  > Add `--source ./filestore_server/migrations --database-url=sqlite://data/filestore.db` arg to check migrations in a different directory

  Example:

  ```bash
  sqlx migrate info --source ./filestore_server/migrations --database-url=sqlite://data/filestore.db
  ```

- **Run** migrations

  ```bash
  sqlx migrate run
  ```

  Example:

  ```bash
  sqlx migrate run --source ./filestore_server/migrations --database-url=sqlite://data/filestore.db
  ```

- **Revert** migrations

  ```bash
  sqlx migrate revert
  ```

  Example:

  ```bash
  sqlx migrate revert --source ./filestore_server/migrations --database-url=sqlite://data/filestore.db
  ```
