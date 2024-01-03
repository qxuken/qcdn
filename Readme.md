# QXuken's CDN

Application for distributed content delivery

## Setup

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

  > Add `-r` flag for a reversible migration

- **Check** migration

  ```bash
  sqlx migrate info
  ```

- **Run** migrations

  ```bash
  sqlx migrate run
  ```

- **Revert** migrations

  ```bash
  sqlx migrate revert
  ```
  