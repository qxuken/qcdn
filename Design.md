# Filestore

An almost immutable storage solution with replication.

## File Server HTTP methods

- GET `<base>/health` - heath (protected)
- GET `<base>/v/<file_version.id>` - download file
- GET `<base>/f/<file.dir>/<file.name>(@<version or tag = latest>)` - download file

## Node Management Server gRPC

### General

- `ping()` - ping
- `version()` - check version

### Files

- `get_dirs()` - get list of all dirs
- `get_dir(dir_id)` - get dir by id
- `get_files(dir_id?)` - get list of all files
- `get_file(file_id)` - get file by id
- `get_file_versions(file_id)` - get list of all file versions
- `get_file_version(file_version_id)` - get file version
- `tag_version(file_version_id, tag)` - tag version
- `upload(file_meta, stream bytes)` - upload file (stream)
- `download(file_version_id)` - download file (stream)
- `delete_version(id)` - delete file

### Nodes communication

- `connect(ip, url, latest_file_ts) -> stream update` - connect to pool
- `get_closest_url(id, ip_addr)` - get closest node url

## DB

### Dir

- `id` (uuid)
- `name`
- `created_at`

### File

- `id` (uuid)
- `dir_id`
- `name`
- `file_type` (other, stylesheets, javascript, image, font, text)
- `created_at`

### FileVersion

- `id` (uuid)
- `file_id` (uuid)
- `size`
- `version`
- `state` (created, downloading, ready)
- `created_at`
- `deleted_at`

### FileVersionTag

- `id` (uuid)
- `file_version_id` (uuid)
- `name`
- `created_at`
- `activated_at`

## Config

- `db_path` - path to sqlite db e.g. `data/filestore.db`
- `storage_dir` - path to storage dir e.g. `data/storage`
- `base_url` - base url e.g. `http://localhost:8080`
- `host` - local interface address e.g. `127.0.0.1`
- `port` - tcp port e.g. `8080`
- `log_level` - debug, info, warn, error
- `master_url` - url to main server (optional)
