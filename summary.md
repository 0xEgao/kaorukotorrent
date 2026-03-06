# Market and sender updates

## Market module
- Added an in-memory offer store backed by `RwLock` + `HashMap` with upsert, list, and delete.
- Added delete offer API and kept legacy route aliases.
- Refactored startup to avoid unwraps and handle errors cleanly.
- Added integration-style tests for create, list, and delete.

## Sender module
- Implemented sender service with endpoints to publish offers, stream updates, and serve files.
- Added metadata building (file list, total size, info hash) and SSE pub/sub announcements.
- Added HTTP client to publish offers to the market service.
- Added environment-based configuration with sane defaults.
- Added unit tests for metadata building and path sanitization.

## Notes
- Market delete uses address+item in the request body.
- Sender defaults: bind `0.0.0.0:4000`, public `http://127.0.0.1:4000`, market `http://127.0.0.1:3000`, data dir `./data`.
