# ReengKigo 관리자 서버 API 참조 문서

## API 개요

ReengKigo 관리자 서버는 교육용 콘텐츠 자산 관리를 위한 RESTful API를 제공합니다. 모든 API는 JSON 형태로 데이터를 주고받으며, JWT 토큰을 통한 인증을 사용합니다.

### 기본 정보
- **Base URL**: `http://localhost:3000`
- **Content-Type**: `application/json`
- **Authentication**: JWT Bearer Token

## 인증 (Authentication)

### 로그인

사용자 로그인을 통해 JWT 토큰을 발급받습니다.

#### 요청

```http
POST /login
Content-Type: application/x-www-form-urlencoded

account=admin&password=password123
```

#### 응답

**성공 (200 OK)**
```json
{
  "success": true,
  "user": {
    "account_id": 1,
    "account": "admin",
    "role": "HEAD_OFFICE",
    "agency_id": 0,
    "academy_id": 0,
    "is_active": true
  },
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**실패 (401 Unauthorized)**
```json
{
  "success": false,
  "message": "Invalid credentials"
}
```

#### 헤더 설정

로그인 성공 시 다음 쿠키가 설정됩니다:
- `auth_token`: HttpOnly 쿠키로 JWT 토큰 저장
- `auth_status`: JavaScript 접근 가능한 인증 상태 쿠키

## 자산 관리 API

모든 자산 관리 API는 관리자 권한(`HEAD_OFFICE` 또는 `REGIONAL_MANAGER`)이 필요합니다.

### 자산 생성

새로운 교육 콘텐츠 자산을 생성합니다.

#### 요청

```http
POST /api/assets
Authorization: Bearer {token}
Content-Type: application/json

{
  "curriculum": "jelly",
  "month": "Jan", 
  "covers": ["cover/1_J1R.png"],
  "subtitles": [
    {
      "page_num": 1,
      "sentence_num": 1,
      "text": "Hello, welcome to our lesson!"
    }
  ],
  "youtube_links": [
    {
      "thumbnail_file": "thumbnail/J1R_chant_1.png",
      "youtube_url": "https://youtu.be/tWsBKRctjQg",
      "title": "Jelly Chapter 1 Chant"
    }
  ]
}
```

#### 응답

**성공 (200 OK)**
```json
{
  "id": "65f7b4c2e1234567890abcde",
  "curriculum": "jelly",
  "month": "Jan",
  "book_id": "J1R",
  "covers": ["cover/1_J1R.png"],
  "subtitles": [
    {
      "page_num": 1,
      "sentence_num": 1,
      "text": "Hello, welcome to our lesson!"
    }
  ],
  "youtube_links": [
    {
      "thumbnail_file": "thumbnail/J1R_chant_1.png",
      "youtube_url": "https://youtu.be/tWsBKRctjQg",
      "title": "Jelly Chapter 1 Chant"
    }
  ],
  "created_at": "2024-03-17T10:30:00Z",
  "updated_at": "2024-03-17T10:30:00Z"
}
```

**실패 (500 Internal Server Error)**
```json
{
  "error": "CREATE_FAILED",
  "message": "Failed to create asset"
}
```

### 자산 목록 조회

모든 자산의 목록을 조회합니다.

#### 요청

```http
GET /api/assets
Authorization: Bearer {token}
```

#### 응답

**성공 (200 OK)**
```json
{
  "assets": [
    {
      "id": "65f7b4c2e1234567890abcde",
      "curriculum": "jelly",
      "month": "Jan",
      "book_id": "J1R",
      "covers": ["cover/1_J1R.png"],
      "subtitles": [
        {
          "page_num": 1,
          "sentence_num": 1,
          "text": "Hello, welcome to our lesson!"
        }
      ],
      "youtube_links": [
        {
          "thumbnail_file": "thumbnail/J1R_chant_1.png",
          "youtube_url": "https://youtu.be/tWsBKRctjQg",
          "title": "Jelly Chapter 1 Chant"
        }
      ],
      "created_at": "2024-03-17T10:30:00Z",
      "updated_at": "2024-03-17T10:30:00Z"
    }
  ],
  "total_count": 1
}
```

### 필터링된 자산 조회

특정 조건에 맞는 자산들을 조회합니다.

#### 요청

```http
GET /api/assets/filter?curriculum=jelly&month=Jan
Authorization: Bearer {token}
```

#### 쿼리 파라미터

| 파라미터 | 타입 | 설명 | 필수 |
|----------|------|------|------|
| `curriculum` | string | 교육과정명 (jelly, juice, stage_1_1 등) | No |
| `month` | string | 월 정보 (Jan, Feb, Mar 등) | No |
| `book_id` | string | 교재 ID | No |

#### 응답

**성공 (200 OK)**
```json
{
  "curriculum": "jelly",
  "month": "Jan",
  "assets": [
    {
      "id": "65f7b4c2e1234567890abcde",
      "curriculum": "jelly",
      "month": "Jan",
      "book_id": "J1R",
      "covers": ["cover/1_J1R.png"],
      "subtitles": [
        {
          "page_num": 1,
          "sentence_num": 1,
          "text": "Hello, welcome to our lesson!"
        }
      ],
      "youtube_links": [
        {
          "thumbnail_file": "thumbnail/J1R_chant_1.png",
          "youtube_url": "https://youtu.be/tWsBKRctjQg",
          "title": "Jelly Chapter 1 Chant"
        }
      ],
      "created_at": "2024-03-17T10:30:00Z",
      "updated_at": "2024-03-17T10:30:00Z"
    }
  ],
  "total_found": 1
}
```

### 자산 수정

기존 자산의 정보를 수정합니다.

#### 요청

```http
PUT /api/assets/{asset_id}
Authorization: Bearer {token}
Content-Type: application/json

{
  "covers": ["cover/1_J1R_updated.png"],
  "subtitles": [
    {
      "page_num": 1,
      "sentence_num": 1,
      "text": "Hello, welcome to our updated lesson!"
    }
  ],
  "youtube_links": [
    {
      "thumbnail_file": "thumbnail/J1R_chant_1_updated.png",
      "youtube_url": "https://youtu.be/newVideoId",
      "title": "Updated Jelly Chapter 1 Chant"
    }
  ]
}
```

#### 응답

**성공 (200 OK)**
```json
{
  "id": "65f7b4c2e1234567890abcde",
  "curriculum": "jelly",
  "month": "Jan",
  "book_id": "J1R",
  "covers": ["cover/1_J1R_updated.png"],
  "subtitles": [
    {
      "page_num": 1,
      "sentence_num": 1,
      "text": "Hello, welcome to our updated lesson!"
    }
  ],
  "youtube_links": [
    {
      "thumbnail_file": "thumbnail/J1R_chant_1_updated.png",
      "youtube_url": "https://youtu.be/newVideoId",
      "title": "Updated Jelly Chapter 1 Chant"
    }
  ],
  "created_at": "2024-03-17T10:30:00Z",
  "updated_at": "2024-03-17T11:45:00Z"
}
```

**실패 (404 Not Found)**
```json
{
  "error": "NOT_FOUND",
  "message": "Asset with id 65f7b4c2e1234567890abcde not found"
}
```

### 자산 삭제

기존 자산을 삭제합니다. 데이터베이스에서 삭제와 동시에 관련 파일들도 삭제됩니다.

#### 요청

```http
DELETE /api/assets/{asset_id}
Authorization: Bearer {token}
```

#### 응답

**성공 (204 No Content)**
```
(응답 본문 없음)
```

**실패 (404 Not Found)**
```json
{
  "error": "NOT_FOUND",
  "message": "Asset with id 65f7b4c2e1234567890abcde not found"
}
```

## 파일 관리 API

### 파일 업로드

이미지 파일을 서버에 업로드합니다.

#### 요청

```http
POST /api/upload
Authorization: Bearer {token}
Content-Type: multipart/form-data

--boundary
Content-Disposition: form-data; name="file"; filename="cover.png"
Content-Type: image/png

[이미지 바이너리 데이터]
--boundary
Content-Disposition: form-data; name="curriculum"

jelly
--boundary
Content-Disposition: form-data; name="month"

Jan
--boundary--
```

#### 폼 필드

| 필드명 | 타입 | 설명 | 필수 |
|--------|------|------|------|
| `file` | file | 업로드할 이미지 파일 (JPEG, JPG, PNG, WEBP) | Yes |
| `curriculum` | string | 교육과정명 | No |
| `month` | string | 월 정보 | No |

#### 응답

**성공 (200 OK)**
```json
{
  "success": true,
  "file_path": "/asset/uploads/1710675600_cover.png",
  "file_type": "image/png",
  "message": "File uploaded successfully"
}
```

**실패 (400 Bad Request)**
```json
{
  "error": "INVALID_FILE_TYPE",
  "message": "File type image/gif not allowed"
}
```

**실패 (400 Bad Request)**
```json
{
  "error": "NO_FILE_UPLOADED",
  "message": "No file was uploaded"
}
```

## 정적 파일 접근

### 자산 파일 접근

업로드된 자산 파일들에 접근할 수 있습니다.

#### 요청

```http
GET /asset/{curriculum}/{month}/{type}/{filename}
```

**예시:**
```http
GET /asset/jelly/Jan/cover/1_J1R.png
GET /asset/jelly/Jan/thumbnail/J1R_chant_1.png
```

### 정적 파일 접근

CSS, JavaScript 등 정적 파일에 접근할 수 있습니다.

#### 요청

```http
GET /static/{type}/{filename}
```

**예시:**
```http
GET /static/css/admin.css
GET /static/js/admin.js
```

### 프로젝트 목록 파일

교육과정과 교재 ID 매핑 정보를 담은 YAML 파일에 접근할 수 있습니다.

#### 요청

```http
GET /project_list.yaml
```

#### 응답

```yaml
jelly:
  month_01: J1R
  month_02: J1O
  month_03: J1Y
  month_04: J1G
  month_06: J1P
  month_07: J2R
  month_08: J2O
  month_12: J2P

juice:
  month_01: JU1
  month_02: JU2
  # ... 기타 매핑 정보
```

## 관리자 페이지

### 대시보드 페이지

관리자 메인 대시보드에 접근합니다.

#### 요청

```http
GET /admin_head
Authorization: Bearer {token} (또는 쿠키를 통한 인증)
```

#### 응답

**성공 (200 OK)**
```html
<!DOCTYPE html>
<html>
<!-- 관리자 대시보드 HTML 내용 -->
</html>
```

**인증 실패 (302 Found)**
```http
Location: /login
```

### 자산 관리 페이지

자산 관리 인터페이스에 접근합니다.

#### 요청

```http
GET /admin_head/assets
Authorization: Bearer {token} (또는 쿠키를 통한 인증)
```

#### 응답

**성공 (200 OK)**
```html
<!DOCTYPE html>
<html>
<!-- 자산 관리 페이지 HTML 내용 -->
</html>
```

## 감독관 페이지

### 감독관 대시보드

감독관 전용 페이지에 접근합니다.

#### 요청

```http
GET /director
Authorization: Bearer {token} (또는 쿠키를 통한 인증)
```

**권한 요구사항**: `DIRECTOR` 역할

#### 응답

**성공 (200 OK)**
```html
<!DOCTYPE html>
<html>
<!-- 감독관 페이지 HTML 내용 -->
</html>
```

## API 문서

### Swagger UI

OpenAPI/Swagger UI를 통한 대화형 API 문서에 접근합니다.

#### 요청

```http
GET /admin_head/api-docs
Authorization: Bearer {token} (또는 쿠키를 통한 인증)
```

**권한 요구사항**: 관리자 권한

#### 응답

**성공 (200 OK)**
```html
<!-- Swagger UI HTML 인터페이스 -->
```

### OpenAPI JSON

OpenAPI 3.0 스펙 JSON 파일에 접근합니다.

#### 요청

```http
GET /admin_head/api-docs/openapi.json
Authorization: Bearer {token} (또는 쿠키를 통한 인증)
```

#### 응답

**성공 (200 OK)**
```json
{
  "openapi": "3.0.3",
  "info": {
    "title": "ReengKi Admin API",
    "version": "1.0.0",
    "description": "REST API for ReengKi educational asset management system",
    "contact": {
      "name": "API Support",
      "email": "support@reengki.com"
    }
  },
  "servers": [
    {
      "url": "http://localhost:3000",
      "description": "Development server"
    }
  ],
  "paths": {
    // API 경로 정의들...
  },
  "components": {
    // 스키마 정의들...
  }
}
```

## 오류 처리

### HTTP 상태 코드

| 상태 코드 | 설명 |
|-----------|------|
| 200 | OK - 요청 성공 |
| 204 | No Content - 성공적으로 삭제됨 |
| 302 | Found - 리다이렉트 (주로 로그인 페이지로) |
| 400 | Bad Request - 잘못된 요청 |
| 401 | Unauthorized - 인증 필요 |
| 403 | Forbidden - 권한 없음 |
| 404 | Not Found - 리소스 없음 |
| 500 | Internal Server Error - 서버 오류 |

### 오류 응답 형식

모든 API 오류는 다음 형식으로 반환됩니다:

```json
{
  "error": "ERROR_CODE",
  "message": "Human readable error message"
}
```

### 주요 오류 코드

| 오류 코드 | 설명 |
|-----------|------|
| `UNAUTHORIZED` | 인증 토큰이 없거나 유효하지 않음 |
| `FORBIDDEN` | 권한 부족 |
| `CREATE_FAILED` | 자산 생성 실패 |
| `UPDATE_FAILED` | 자산 수정 실패 |
| `DELETE_FAILED` | 자산 삭제 실패 |
| `LIST_ASSETS_FAILED` | 자산 목록 조회 실패 |
| `GET_FILTERED_ASSETS_FAILED` | 필터링된 자산 조회 실패 |
| `NOT_FOUND` | 요청한 리소스를 찾을 수 없음 |
| `INVALID_FILE_TYPE` | 허용되지 않는 파일 타입 |
| `NO_FILE_UPLOADED` | 업로드된 파일이 없음 |

## 인증 헤더

API 요청 시 다음 중 하나의 방법으로 인증을 수행할 수 있습니다:

### Authorization 헤더

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

### Cookie 인증

```http
Cookie: auth_token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## 데이터 타입

### 자산 관련 데이터 타입

#### CreateAssetRequest
```typescript
{
  curriculum: string;      // 교육과정명
  month: string;          // 월 정보
  covers: string[];       // 표지 이미지 경로들
  subtitles: SubtitleEntry[];
  youtube_links: YouTubeLink[];
}
```

#### SubtitleEntry
```typescript
{
  page_num: number;       // 페이지 번호
  sentence_num: number;   // 문장 번호
  text: string;          // 자막 텍스트
}
```

#### YouTubeLink
```typescript
{
  thumbnail_file: string;  // 썸네일 파일 경로
  youtube_url: string;     // 유튜브 URL
  title?: string;         // 제목 (선택사항)
}
```

#### AssetResponse
```typescript
{
  id: string;             // 자산 ID
  curriculum: string;     // 교육과정명
  month: string;          // 월 정보
  book_id: string;        // 교재 ID
  covers: string[];       // 표지 이미지 경로들
  subtitles: SubtitleEntry[];
  youtube_links: YouTubeLink[];
  created_at?: string;    // 생성 시간
  updated_at?: string;    // 수정 시간
}
```

이 API 참조 문서는 ReengKigo 관리자 서버의 모든 공개 API 엔드포인트를 설명합니다. 각 API의 상세한 동작과 비즈니스 로직은 소스 코드의 handlers 및 services 모듈을 참조하시기 바랍니다.