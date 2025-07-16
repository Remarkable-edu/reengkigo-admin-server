# Asset Management API Documentation

## Overview

The Asset Management API provides comprehensive CRUD operations for managing educational content assets. Each asset contains collections of books, with each book containing video content.

## Base URL

```
http://localhost:3000/api
```

## Endpoints

### 1. Create Asset

**POST** `/assets`

Creates a new asset with books and video content.

#### Request Body

```json
{
  "curriculum": [
    {
      "id": "curriculum_1",
      "books": [
        {
          "book_id": "J1R",
          "month": "January",
          "cover_img": "path/to/cover.jpg",
          "video_content": [
            {
              "video_img": "path/to/video_thumbnail.jpg",
              "youtube_url": "https://youtube.com/watch?v=example"
            }
          ]
        }
      ]
    }
  ]
}
```

#### Response (201 Created)

```json
{
  "id": "507f1f77bcf86cd799439011",
  "books": [
    {
      "book_id": "BOOK_001",
      "month": "January",
      "cover_img": "path/to/cover.jpg",
      "video_content": [
        {
          "video_img": "path/to/video_thumbnail.jpg",
          "youtube_url": "https://youtube.com/watch?v=example"
        }
      ]
    }
  ]
}
```

### 2. List All Assets

**GET** `/assets`

Retrieves all assets in the system.

#### Response (200 OK)

```json
{
  "assets": [
    {
      "id": "507f1f77bcf86cd799439011",
      "books": [
        {
          "book_id": "J1R",
          "month": "January",
          "cover_img": "path/to/cover.jpg",
          "video_content": [
            {
              "video_img": "path/to/video_thumbnail.jpg",
              "youtube_url": "https://youtube.com/watch?v=example"
            }
          ]
        }
      ]
    }
  ],
  "total_count": 1
}
```

### 3. Get Project Month Data

**GET** `/project/{month}`

Retrieves all books for a specific month across all assets.

#### Path Parameters

- `month` (string, required): The month name (e.g., "January", "February")

#### Response (200 OK)

```json
{
  "month": "January",
  "books": [
    {
      "book_id": "J1R",
      "month": "January",
      "cover_img": "path/to/cover.jpg",
      "video_content": [
        {
          "video_img": "path/to/video_thumbnail.jpg",
          "youtube_url": "https://youtube.com/watch?v=example"
        }
      ]
    }
  ]
}
```

#### Response (404 Not Found)

```json
{
  "error": "NOT_FOUND",
  "message": "No data found for month: January"
}
```

## Data Models

### Asset

| Field | Type | Description |
|-------|------|-------------|
| id | string | Unique identifier for the asset |
| curriculum | Curriculum[] | Array of curriculum contained in this asset |

### Curriculum

| Field | Type | Description |
|-------|------|-------------|
| id | string | Unique identifier for the curriculum |
| books | Book[] | Array of books in this curriculum |

### Book

| Field | Type | Description |
|-------|------|-------------|
| book_id | string | Unique identifier for the book |
| month | string | Month associated with the book |
| cover_img | string | Path to the book's cover image |
| video_content | VideoContent[] | Array of video content for this book |

### VideoContent

| Field | Type | Description |
|-------|------|-------------|
| video_img | string | Path to the video thumbnail image |
| youtube_url | string | YouTube URL for the video content |

## Error Handling

All endpoints return appropriate HTTP status codes and error responses in the following format:

```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable error message"
}
```

### Common Error Codes

- `NOT_FOUND`: Resource not found
- `CREATE_FAILED`: Failed to create resource
- `GET_MONTH_DATA_FAILED`: Failed to retrieve month data
- `LIST_ASSETS_FAILED`: Failed to retrieve assets list

## Example Usage

### Create a new asset with curl

```bash
curl -X POST http://localhost:3000/api/assets \
  -H "Content-Type: application/json" \
  -d '{
    "books": [
      {
        "book_id": "J1R",
        "month": "January",
        "cover_img": "covers/january.jpg",
        "video_content": [
          {
            "video_img": "thumbnails/lesson1.jpg",
            "youtube_url": "https://youtube.com/watch?v=abc123"
          }
        ]
      }
    ]
  }'
```

### List all assets

```bash
curl http://localhost:3000/api/assets
```

### Get project month data

```bash
curl http://localhost:3000/api/project/January
```