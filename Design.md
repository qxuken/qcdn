# Filestore

An almost immutable storage solution with replication __**(later)**__.

## HTTP

- GET `<base>/health` - heath (protected)
- GET `<base>/v/<file_version.id>` - download file
- GET `<base>/f/<file.dir>/<file.name>(@<version or tag = latest>)` - download file

## gRPC

### General

- `ping()` - ping
- `version()` - check version

### File

#### Queries

- `get_dirs()` - get list of all dirs
- `get_dir(dir_id)` - get dir by id
- `get_files(dir_id?)` - get list of all files
- `get_file(file_id)` - get file by id
- `get_file_versions(file_id)` - get list of all file versions
- `get_file_version(file_version_id)` - get file version
- `download(file_version_id)` - download file (stream)

#### Updates

- `upload(file_meta, stream bytes)` - upload file (stream)
- `tag_version(file_version_id, tag)` - tag version
- `delete_version(id)` - delete file

## DB

### dir

- `id`
- `name`
- `created_at`

### file

- `id`
- `dir_id`
- `name`
- `file_type` (other, stylesheets, javascript, image, font, text)
- `created_at`

### file_version

- `id`
- `file_id`
- `size`
- `version`
- `state` (created, downloading, ready)
- `created_at`
- `deleted_at`

### file_version_tag

- `id`
- `file_version_id`
- `name`
- `created_at`
- `activated_at`

## Config

- `data` - path to data dir e.g. `data`
- `bind` - bind ip address e.g. `0.0.0.0:8080`
- `...logger`
