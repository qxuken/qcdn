# Filestore

A storage solution with geo replication.

## Config

- `db_path` - path to sqlite db e.g. `data/filestore.db`
- `storage_dir` - path to storage dir e.g. `data/storage`
- `base_url` - base url e.g. `http://localhost:8080`
- `host` - local interface address e.g. `127.0.0.1`
- `http_port` - http port e.g. `8080`
- `grpc_port` - grpc port e.g. `8081`
- `log_level` - debug, info, warn, error

## HTTP methods

- GET `<base>/f/<file>` - download file
- GET `<base>/f/<file>.meta` - download file meta

## gRPC

url - `<base>`/grpc (master_only)

### calls

#### general

- `ping()` - ping
- `version()` - check version

#### files

- `upload(file_name, size, stream bytes)` - upload file (stream)
- `delete(id)` - delete file
- `get_closest_url(id, ip_addr)` - get closest node url

#### nodes communication

- `connect(ip, url)` - connect to pool
- `sync(file)` - request to download file
- `updated_since(timestamp)` - updated files since

## DB

### File

- `id` (uuid)
- `dir_id` (uuid)
- `name`
- `size`
- `state` (ready, uploading, downloading, create)
- `file_type`
- `meta` (json)
- `created_at`

### Dir

- `id` (uuid)
- `path`
- `created_at`
