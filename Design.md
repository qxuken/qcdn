# Filestore

A storage solution with geo replication.

## Config

- `db_path` - path to sqlite db e.g. `data/filestore.db`
- `storage_dir` - path to storage dir e.g. `data/storage`
- `base_url` - base url e.g. `http://localhost:8080`
- `host` - local interface address e.g. `127.0.0.1`
- `port` - tcp port e.g. `8080`
- `log_level` - debug, info, warn, error
- `master_url` - url to main server (optional)

## File Server HTTP methods

- GET `<base>/health` - heath (protected)
- GET `<base>/f/<file.dir>/<file.name>.<file.file_type>?(@<version>)` - download file
- GET `<base>/f/<file.dir>/<file.name>.<file.file_type>?(@<version>).meta` - download file meta

## Node Management Server gRPC

### General

- `ping()` - ping
- `version()` - check version

### Files

- `upload(file_meta, stream bytes)` - upload file (stream)
- `delete_version(id)` - delete file
- `get_closest_url(id, ip_addr)` - get closest node url

### Nodes communication

- `connect(ip, url)` - connect to pool
- `sync(file)` - request to download file
- `updated_since(timestamp)` - updated files since

## DB

### File

- `id` (uuid)
- `dir`
- `name`
- `file_type` (other, stylesheets, javascript, image, font)
- `updated_at`
- `created_at`

### FileVersion

- `id` (uuid)
- `file_id` (uuid)
- `size`
- `version`
- `state` (created, downloading, ready)
- `created_at`
- `updated_at`
- `deleted_at`

### FileLatestVersion

- `id` (uuid)
- `file_id` (uuid)
- `file_version_id` (uuid)
- `created_at`
- `deleted_at`
