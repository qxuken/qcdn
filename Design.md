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
- GET `<base>/f/<file.id>.meta` - download file meta
- GET `<base>/f/<file.dir>/<file.name>.<file.file_type>` - download file
- GET `<base>/f/<file.dir>/<file.name>.<file.file_type>.meta` - download file meta

## Node Management Server gRPC

### General

- `ping()` - ping
- `version()` - check version

### Files

- `upload(file_dir, file_name, size, stream bytes)` - upload file (stream)
- `download(file_id)` - download file (stream)
- `delete(id)` - delete file
- `get_closest_url(id, ip_addr)` - get closest node url

### Nodes communication

- `connect(ip, url)` - connect to pool
- `sync(file)` - request to download file
- `updated_since(timestamp)` - updated files since

## DB

### File

- `id` (uuid)
- `dir_id` (uuid)
- `name`
- `file_type` (other, stylesheets, javascript, image, font)
- `version` default: latest
- `size`
- `state` (created, uploading, downloading, ready)
- `meta` (json)
- `created_at`

### Dir

- `id` (uuid)
- `path`
- `created_at`
