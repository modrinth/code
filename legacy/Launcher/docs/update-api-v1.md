# Update API v1

Base URL comes from `config.json` → `updateBaseUrl`.  
If empty, the launcher uses built-in **mock://local** fixtures (Stage 2 MVP).

## Endpoints

### `GET /v1/packs`
Returns `{ "packs": [ PackSummary, ... ] }`.

### `GET /v1/packs/{id}/manifest`
Returns pack manifest with `files[]` (`path`, `url`, `sha256`, `size`).

## Local layout

```text
%USERPROFILE%\owyx\
  config.json
  instances\{pack_id}\game\...
  instances\{pack_id}\meta.json
  cache\
  logs\
```

Product name / binary: **Owyx**.
