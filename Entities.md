# Entities

## File process entities

### Files upload

#### State transitions

```text
  s  - File upload requested
  1  -> File meta received(if version already exists -> e1)
  2  -> File chunks received(if stopped before receiving full file -> e1)
  r  -| File upload end(if stopped before receiving full file -> e1)

  e1 -| File upload bail(if ended before receiving full file -> e1)
```

#### State actions

##### File upload requested

- establish connection
- start bytes counter

##### File meta received

- create system file handler
- create or find file record
- create file version with downloading state

##### File chunks received

- write chunk to file

##### File upload end

- check meta size with received amount
- mark version as ready
- transition latest if needed

##### File upload bail

- delete file
- delete version
- delete file if no version remaining
