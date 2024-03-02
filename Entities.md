# Entities

Page describes internal logic

## File process entities

### Files upload

#### State transitions

```text
  s  -  File upload requested
        (internal error -> e1)

  1  -> File meta received 
        (version already exists -> e1)

  ----> chunk received
        (stopped before receiving full file -> e1)

  r  -| File upload end
        (ended before receiving full file -> e1)

  e1 -| File upload bail
```

#### State actions

##### File upload requested

init:

- establish db connection

##### File meta received

init from `s`:

- start bytes counter
- create system file handler
- create or find file record
- create file version with downloading state
  - throw if file already exists

chunk received:

- write chunk to file

##### File upload end

init from `1`:

- check meta size with received amount
- check meta hash with resulted file hash
- mark version as ready
- transition latest if needed
- send update message

##### File upload bail

bailed:

- delete system file
- delete version
- delete file if no version remaining
- notify update
