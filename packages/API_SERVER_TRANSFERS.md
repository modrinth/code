# Server Transfer API Specification

## Overview

The Server Transfer API provides endpoints for scheduling, managing, and tracking server transfers between nodes. All endpoints require `INFRA_MANAGER` global permissions.

**Base Path**: `/internal/transfers`

**Authentication**: All endpoints require Bearer token authentication with `INFRA_MANAGER` scope.

---

## Endpoints

### 1. Get Transfer History

Retrieve a paginated list of transfer batches.

**Endpoint**: `GET /internal/transfers/history`

**Authentication**: Required (`INFRA_MANAGER`)

**Query Parameters**:

- `page` (optional, default: `1`): Page number (1-indexed)
- `page_size` (optional, default: `50`, max: `100`): Number of batches per page

**Response**: `200 OK`

```json
{
	"batches": [
		{
			"id": 123,
			"created_by": "abcdefg1",
			"created_at": "2026-01-12T18:30:00Z",
			"reason": "Node maintenance",
			"scheduled_at": "2026-01-13T00:00:00Z",
			"cancelled": false,
			"log_count": 5,
			"provision_options": {
				"region": "us-east",
				"node_tags": ["batch20251215", "ovh-gen4"]
			}
		}
	],
	"total": 42,
	"page": 1,
	"page_size": 50
}
```

**Response Fields**:

- `batches`: Array of transfer batch entries
  - `id` (i64): Batch ID
  - `created_by` (string): Modrinth user ID (base62) of the admin who created the batch
  - `created_at` (ISO 8601 datetime): When the batch was created
  - `reason` (string | null): Optional reason/note for the transfer
  - `scheduled_at` (ISO 8601 datetime): When transfers in this batch are scheduled to execute
  - `cancelled` (boolean): Whether this batch has been cancelled
  - `log_count` (i64): Number of individual transfer log entries in this batch
  - `provision_options` (object): Provision options for the transfers
    - `region` (string | null): Target region (optional)
    - `node_tags` (string[]): Preferred node tags (optional)
- `total` (i64): Total number of batches across all pages
- `page` (u32): Current page number
- `page_size` (u32): Number of batches per page

**Error Responses**:

- `401 Unauthorized`: Missing or invalid authentication, or insufficient permissions
- `500 Internal Server Error`: Server error

**Example Request**:

```bash
curl -X GET "https://api.example.com/internal/transfers/history?page=1&page_size=20" \
  -H "Authorization: Bearer <token>"
```

---

### 2. Schedule Server Transfers

Schedule transfers for specific servers.

**Endpoint**: `POST /internal/transfers/schedule/servers`

**Authentication**: Required (`INFRA_MANAGER`)

**Request Body**:

```json
{
	"server_ids": ["123e4569-e89b-12d3-a456-426614174005", "123e9569-e89b-12d3-a456-413678919876"],
	"scheduled_at": "2026-01-13T00:00:00Z",
	"target_region": "us-east",
	"node_tags": ["batch20251215", "high-memory"],
	"reason": "Node maintenance scheduled"
}
```

**Request Fields**:

- `server_ids` (UUID[], required): Array of server UUIDs to transfer
- `scheduled_at` (ISO 8601 datetime, optional): When to schedule the transfer (defaults to current time if not specified)
- `target_region` (string, optional): Target region for the transfer
- `node_tags` (string[], optional): Preferred node tags for target selection
- `reason` (string, optional): Optional reason/note for the transfer

**Response**: `200 OK`

```json
{
	"batch_id": 123,
	"scheduled_count": 2
}
```

**Response Fields**:

- `batch_id` (i64): The batch ID for this batch of transfers
- `scheduled_count` (number): Number of transfer log entries created

**Error Responses**:

- `400 Bad Request`: Empty `server_ids` array or invalid request format
- `401 Unauthorized`: Missing or invalid authentication, or insufficient permissions
- `500 Internal Server Error`: Server error

**Example Request**:

```bash
curl -X POST "https://api.example.com/internal/transfers/schedule/servers" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "server_ids": ["123e4569-e89b-12d3-a456-426614174005"],
    "target_region": "us-east",
    "node_tags": ["batch20251215"],
    "reason": "Moving to new region"
  }'
```

---

### 3. Schedule Node Transfers

Schedule transfers for all servers on specific nodes (node evacuation).

**Endpoint**: `POST /internal/transfers/schedule/nodes`

**Authentication**: Required (`INFRA_MANAGER`)

**Request Body**:

```json
{
	"node_hostnames": ["us-vin200", "as-sin1"],
	"scheduled_at": "2026-01-13T00:00:00Z",
	"target_region": "us-vin",
	"node_tags": ["batch20251215"],
	"reason": "Node decommissioning"
}
```

**Request Fields**:

- `node_ids` (UUID[], required): Array of node UUIDs to evacuate (all servers on these nodes will be transferred)
- `scheduled_at` (ISO 8601 datetime, optional): When to schedule the transfer (defaults to current time if not specified)
- `target_region` (string, optional): Target region for the transfer
- `node_tags` (string[], optional): Preferred node tags for target selection
- `reason` (string, optional): Optional reason/note for the transfer

**Response**: `200 OK`

```json
{
	"batch_id": 124,
	"scheduled_count": 2
}
```

**Response Fields**:

- `batch_id` (i64): The batch ID for this batch of transfers
- `scheduled_count` (number): Number of transfer log entries created (one per node)

**Error Responses**:

- `400 Bad Request`: Empty `node_ids` array or invalid request format
- `401 Unauthorized`: Missing or invalid authentication, or insufficient permissions
- `500 Internal Server Error`: Server error

**Example Request**:

```bash
curl -X POST "https://api.example.com/internal/transfers/schedule/nodes" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "node_ids": ["a23e4567-e89b-12d3-a456-426614175009"],
    "target_region": "us-west",
    "reason": "Node maintenance"
  }'
```

---

### 4. Cancel Transfers

Cancel pending transfers by batch IDs.

**Endpoint**: `POST /internal/transfers/cancel`

**Authentication**: Required (`INFRA_MANAGER`)

**Request Body**:

```json
{
	"batch_ids": [123, 124, 125]
}
```

**Request Fields**:

- `batch_ids` (i64[], required): Array of batch IDs to cancel

**Response**: `200 OK`

```json
{
	"cancelled_count": 8
}
```

**Response Fields**:

- `cancelled_count` (number): Number of transfer log entries that were actually cancelled

**Error Responses**:

- `400 Bad Request`: Empty `batch_ids` array or invalid request format
- `401 Unauthorized`: Missing or invalid authentication, or insufficient permissions
- `500 Internal Server Error`: Server error

**Notes**:

- Cancellation is performed at the batch level. All individual transfer log entries within the specified batches will be marked as cancelled.
- If a batch ID doesn't exist, it will be silently ignored (no error).
- If a batch is already cancelled, the operation will still succeed but may return the count of already-cancelled entries.

**Example Request**:

```bash
curl -X POST "https://api.example.com/internal/transfers/cancel" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "batch_ids": [123, 124]
  }'
```

---

## Data Types

### UUID

Standard UUID v4 format: `123e4569-e89b-12d3-a456-426614174005`

### DateTime

ISO 8601 format with UTC timezone: `2026-01-12T18:30:00Z`

### User ID

Modrinth user ID in base62 format (8 characters): `abcdefg1`

### Batch ID

64-bit integer: `123`

---

## Authentication

All endpoints require Bearer token authentication:

```
Authorization: Bearer <token>
```

The token must belong to a user with `INFRA_MANAGER` global permissions. You cannot check for this ahead of time. Users without this permission will receive a `401 Unauthorized` response.

---

## Pagination

The history endpoint supports pagination:

- **Page numbers are 1-indexed** (first page is `page=1`)
- **Default page size** is `50`
- **Maximum page size** is `100` (values above 100 will be capped)
- **Total count** is provided in the response for calculating total pages

**Example Pagination**:

```bash
# First page
GET /internal/transfers/history?page=1&page_size=20

# Second page
GET /internal/transfers/history?page=2&page_size=20

# Calculate total pages: ceil(total / page_size)
```

---

## Provision Options

Provision options control where and how servers are transferred:

- **`region`** (optional): Target region. If specified, the transfer will only consider nodes in this region. If no nodes are available in the specified region, the transfer may block.
- **`node_tags`** (optional): Array of preferred node tags. The system will prefer nodes with these tags but will not block if no matching nodes are available.

**Example**:

```json
{
	"target_region": "us-east",
	"node_tags": ["batch20251215", "high-memory"]
}
```

---

## Workflow Examples

### Example 1: Schedule and Cancel a Transfer

```bash
# 1. Schedule transfers for specific servers
POST /internal/transfers/schedule/servers
{
  "server_ids": ["123e4569-e89b-12d3-a456-426614174005"],
  "target_region": "us-east",
  "reason": "Moving to new region"
}

# Response: { "batch_id": 123, "scheduled_count": 1 }

# 2. Check transfer history
GET /internal/transfers/history?page=1

# 3. Cancel the transfer if needed
POST /internal/transfers/cancel
{
  "batch_ids": [123]
}

# Response: { "cancelled_count": 1 }
```

### Example 2: Evacuate a Node

```bash
# Schedule transfers for all servers on a node
POST /internal/transfers/schedule/nodes
{
  "node_ids": ["a23e4567-e89b-12d3-a456-426614175009"],
  "target_region": "us-west",
  "node_tags": ["batch20251215"],
  "reason": "Node decommissioning"
}

# Response: { "batch_id": 124, "scheduled_count": 1 }
```

### Example 3: Paginated History Retrieval

```bash
# Get first page
GET /internal/transfers/history?page=1&page_size=10

# Get second page
GET /internal/transfers/history?page=2&page_size=10

# Response includes total count for pagination UI
{
  "batches": [...],
  "total": 42,
  "page": 2,
  "page_size": 10
}
```

---

## Notes

1. **Batch-Level Operations**: All operations work at the batch level. Individual transfer log entries are not exposed in the API.

2. **Scheduling**: Transfers are scheduled but not immediately executed. A background worker processes scheduled transfers and creates actual server provisioning requests.

3. **Cancellation**: Only pending (non-scheduled) transfers can be cancelled. Once a transfer has been processed by the worker, it cannot be cancelled through this API.

4. **Node Transfers**: When scheduling transfers for nodes, one transfer log entry is created per node. The actual server transfers will be expanded by the background worker.

5. **Idempotency**: Scheduling the same servers/nodes multiple times will create multiple batches. There is no automatic deduplication.

6. **Time Zones**: All datetime values are in UTC and should be formatted as ISO 8601 strings with the `Z` suffix.

---

## Version

This API specification is current as of 2026-01-12.
