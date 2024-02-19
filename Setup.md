# Setup

## Initial

1. Setup `env` variables

    ```bash
    cp .env.development .env
    ```

2. Install `protoc` compiler using [tonic instructions](https://github.com/hyperium/tonic?tab=readme-ov-file#dependencies)

3. Install `sqlx-cli`

    ```bash
    cargo install sqlx-cli
    ```

## Development

### Migrations

- **Create** migration

  ```bash
  sqlx migrate add <name>
  ```

  > Add `-r` flag for a reversible migration.
  > Add --source ./packages/database/migrations arg to create migrations in a different directory 

- **Check** migration

  ```bash
  sqlx migrate info
  ```
  > Add --source ./packages/database/migrations --database-url=sqlite://data/filestore.db arg to check migrations in a different directory

- **Run** migrations

  ```bash
  sqlx migrate run
  ```

- **Revert** migrations

  ```bash
  sqlx migrate revert
  ```
  .

