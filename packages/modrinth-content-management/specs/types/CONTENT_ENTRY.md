# Content Entry

A content entry stores the local file details for a [content item](./CONTENT_ITEMS.md).

A content item has zero or one entry.

- `id`
- `content_item_id` - Content item this entry belongs to. **Only one entry per item.**
- `file_path` - File path relative to the instance directory, without the `.disabled` suffix.
- `sha512` - Optional, expected SHA-512 hash.
- `size` - File size in bytes.
- `status` - `Present`, `Missing`, or `Conflict`. Conflict means the path contains unexpected content or cannot hold the file.
