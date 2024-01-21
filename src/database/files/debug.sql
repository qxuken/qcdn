SELECT file.id,
       fv.id fv_id,
       file.dir,
       file.name,
       file.file_type,
       fv.version,
       fv.state,
       fv.created_at,
       fv.deleted_at,
       flv.expired_at
  FROM file
       LEFT JOIN
       file_version fv ON file.id = fv.file_id
       LEFT JOIN
       file_latest_version flv ON file.id = flv.file_id AND 
                                  fv.id = flv.file_version_id;

SELECT * FROM file;
SELECT * FROM file_version;
SELECT true FROM file_version WHERE file_id = '018d4b9b-e37e-74d3-96d8-db2e86e17e30' AND version = '0' AND deleted_at IS NULL;
SELECT * FROM file_latest_version;

DELETE FROM file_latest_version;
DELETE FROM file_version;
DELETE FROM file;
