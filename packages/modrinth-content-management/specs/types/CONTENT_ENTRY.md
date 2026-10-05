# Content Entry

A content entry stores the local file details for a [content item](./CONTENT_ITEMS.md).

- `id` - Unique entry ID.
- `content_item_id` - Reference to the content item's stable ID.
- `file_path` - File path relative to the instance directory.
- `sha1` - SHA-1 hash of the file.
- `sha512` - SHA-512 hash of the file.
- `size` - File size in bytes.
- `missing` - Whether the file is missing from disk.
