# ReengKigo 관리자 서버 프로젝트 문서

## 1. 프로젝트 개요

### 1.1 시스템 소개
ReengKigo 관리자 서버는 교육용 콘텐츠 자산 관리를 위한 웹 서버입니다. Rust 언어와 Axum 프레임워크를 기반으로 구축되었으며, MongoDB를 데이터베이스로 사용합니다.

### 1.2 주요 특징
- **교육 콘텐츠 자산 관리**: 책, 비디오, 자막, 유튜브 링크 등의 교육 자료 관리
- **역할 기반 접근 제어**: 관리자(HEAD_OFFICE, REGIONAL_MANAGER)와 감독관(DIRECTOR) 역할 분리
- **파일 업로드 및 관리**: 이미지 파일 업로드와 체계적인 폴더 구조 관리
- **RESTful API**: 완전한 CRUD 작업을 지원하는 REST API 제공
- **웹 관리 인터페이스**: HTML 기반 관리자 대시보드

### 1.3 기술 스택
- **언어**: Rust (Edition 2021)
- **웹 프레임워크**: Axum
- **데이터베이스**: MongoDB
- **인증**: JWT (JSON Web Token)
- **설정 관리**: Figment (YAML + 환경변수)
- **로깅**: tracing crate
- **API 문서**: utoipa (OpenAPI/Swagger)

## 2. 아키텍처 개요

### 2.1 프로젝트 구조

```
src/
├── main.rs              # 애플리케이션 진입점 및 서버 설정
├── lib.rs               # 라이브러리 루트
├── config/              # 설정 관리
│   └── mod.rs          # 애플리케이션 설정 구조체
├── handlers/            # HTTP 요청 처리기
│   ├── admin_head.rs   # 관리자 기능 핸들러
│   ├── auth.rs         # 인증 관련 핸들러
│   └── mod.rs
├── services/            # 비즈니스 로직 계층
│   ├── asset.rs        # 자산 관리 서비스
│   ├── auth.rs         # 인증 서비스
│   ├── database.rs     # 데이터베이스 연결 관리
│   └── mod.rs
├── models/              # 데이터 모델
│   ├── asset.rs        # 자산 데이터 모델
│   ├── user.rs         # 사용자 데이터 모델
│   └── mod.rs
├── dto/                 # 데이터 전송 객체
│   ├── asset.rs        # 자산 관련 DTO
│   ├── auth.rs         # 인증 관련 DTO
│   └── mod.rs
├── middleware/          # 미들웨어
│   ├── auth.rs         # 인증 미들웨어
│   └── mod.rs
├── utils/               # 유틸리티
│   ├── logging.rs      # 로깅 설정
│   └── mod.rs
└── templates/           # HTML 템플릿
    ├── admin-head/     # 관리자 템플릿
    ├── director/       # 감독관 템플릿
    └── login.html      # 로그인 페이지
```

### 2.2 데이터 흐름

1. **요청 수신**: HTTP 요청이 Axum 라우터로 들어옴
2. **인증/인가**: AuthMiddleware에서 JWT 토큰 검증 및 역할 확인
3. **요청 처리**: 적절한 핸들러로 요청 라우팅
4. **비즈니스 로직**: 서비스 계층에서 핵심 로직 실행
5. **데이터 접근**: MongoDB를 통한 데이터 저장/조회
6. **응답 반환**: JSON 또는 HTML 형태로 응답 반환

## 3. 주요 모듈별 기능

### 3.1 main.rs - 애플리케이션 진입점
- **서버 초기화**: 설정 로드, 데이터베이스 연결, 라우터 설정
- **우아한 종료**: SIGTERM, Ctrl+C 신호 처리로 안전한 서버 종료
- **애플리케이션 상태 관리**: AppState 구조체로 전역 상태 관리

**주요 함수**:
- `main()`: 애플리케이션 시작점
- `create_router(state: AppState)`: 라우팅 설정
- `shutdown_signal()`: 종료 신호 처리

### 3.2 handlers/ - HTTP 요청 처리

#### 3.2.1 admin_head.rs - 관리자 기능
관리자 권한이 필요한 모든 자산 관리 기능을 담당합니다.

**주요 함수**:
- `dashboard_page()`: 관리자 대시보드 페이지 (GET /admin_head)
- `asset_management_page()`: 자산 관리 페이지 (GET /admin_head/assets)
- `create_asset()`: 새 자산 생성 (POST /api/assets)
- `list_asset()`: 자산 목록 조회 (GET /api/assets)
- `get_filtered_assets()`: 필터링된 자산 조회 (GET /api/assets/filter)
- `update_asset()`: 자산 정보 수정 (PUT /api/assets/:id)
- `delete_asset()`: 자산 삭제 (DELETE /api/assets/:id)
- `upload_file()`: 파일 업로드 (POST /api/upload)

#### 3.2.2 auth.rs - 인증 관리
사용자 로그인과 인증 토큰 관리를 담당합니다.

**주요 함수**:
- `login_page()`: 로그인 페이지 표시 (GET /login)
- `login_handler()`: 로그인 처리 및 JWT 토큰 발급 (POST /login)

### 3.3 services/ - 비즈니스 로직

#### 3.3.1 asset.rs - 자산 관리 서비스
교육 콘텐츠 자산의 모든 비즈니스 로직을 처리합니다.

**주요 함수**:
- `create_asset()`: 자산 생성 및 파일 시스템 구조 생성
- `list_asset()`: 모든 자산 목록 조회
- `get_filtered_assets()`: 조건부 자산 필터링
- `update_asset()`: 자산 정보 업데이트
- `delete_asset()`: 자산 삭제 및 파일 시스템 정리
- `move_uploaded_files_to_asset_folder()`: 업로드된 파일을 적절한 폴더로 이동
- `get_book_id_from_mapping()`: project_list.yaml에서 book_id 매핑 조회

#### 3.3.2 auth.rs - 인증 서비스
사용자 인증과 JWT 토큰 관리를 담당합니다.

**주요 함수**:
- `authenticate_user()`: 사용자 자격 증명 검증
- `generate_admin_token()`: JWT 토큰 생성
- `validate_token()`: JWT 토큰 검증 및 클레임 추출

#### 3.3.3 database.rs - 데이터베이스 서비스
MongoDB 연결과 기본 데이터베이스 작업을 관리합니다.

**주요 기능**:
- MongoDB 클라이언트 연결 관리
- 데이터베이스 연결 풀링
- 기본 CRUD 작업 인터페이스

### 3.4 models/ - 데이터 모델

#### 3.4.1 asset.rs - 자산 데이터 모델
교육 콘텐츠 자산의 데이터 구조를 정의합니다.

**주요 구조체**:
- `Asset`: 메인 자산 구조체
  - `curriculum`: 교육과정명 (jelly, juice, stage_1_1 등)
  - `month`: 월 정보 (Jan, Feb, Mar 등)
  - `book_id`: 교재 ID
  - `covers`: 표지 이미지 파일 경로 목록
  - `subtitles`: 자막 데이터 목록
  - `youtube_links`: 유튜브 링크 목록

- `SubtitleEntry`: 자막 항목
  - `page_num`: 페이지 번호
  - `sentence_num`: 문장 번호
  - `text`: 자막 텍스트

- `YouTubeLink`: 유튜브 링크 정보
  - `thumbnail_file`: 썸네일 이미지 경로
  - `youtube_url`: 유튜브 URL
  - `title`: 제목 (선택사항)

#### 3.4.2 user.rs - 사용자 모델
시스템 사용자의 정보와 권한을 정의합니다.

**주요 구조체**:
- `AdminUser`: 관리자 사용자
  - `account_id`: 계정 ID
  - `account`: 계정명
  - `role`: 역할 (HEAD_OFFICE, REGIONAL_MANAGER, DIRECTOR)
  - `agency_id`: 에이전시 ID
  - `academy_id`: 아카데미 ID
  - `is_active`: 활성 상태

### 3.5 middleware/ - 미들웨어

#### 3.5.1 auth.rs - 인증 미들웨어
모든 보호된 라우트에서 사용자 인증과 권한 검사를 수행합니다.

**주요 함수**:
- `auth_middleware()`: JWT 토큰 검증 및 사용자 정보 추출
- `require_admin_role()`: 관리자 권한 확인
- `require_director_role()`: 감독관 권한 확인
- `require_any_role()`: 유효한 역할 확인

### 3.6 config/ - 설정 관리

#### 3.6.1 mod.rs - 애플리케이션 설정
Figment를 사용한 계층적 설정 관리를 제공합니다.

**설정 우선순위**:
1. 기본값 (코드 내 정의)
2. config.yaml 파일
3. 환경변수 (APP_ 접두사)

**주요 설정 섹션**:
- `app`: 애플리케이션 메타데이터
- `database`: MongoDB 연결 정보  
- `server`: 서버 호스트 및 포트

## 4. API 엔드포인트

### 4.1 인증 관련 API

| 메서드 | 경로 | 설명 | 권한 |
|--------|------|------|------|
| GET | `/login` | 로그인 페이지 | 공개 |
| POST | `/login` | 로그인 처리 | 공개 |

### 4.2 자산 관리 API

| 메서드 | 경로 | 설명 | 권한 |
|--------|------|------|------|
| POST | `/api/assets` | 새 자산 생성 | 관리자 |
| GET | `/api/assets` | 모든 자산 조회 | 관리자 |
| GET | `/api/assets/filter` | 필터링된 자산 조회 | 관리자 |
| PUT | `/api/assets/:id` | 자산 정보 수정 | 관리자 |
| DELETE | `/api/assets/:id` | 자산 삭제 | 관리자 |
| POST | `/api/upload` | 파일 업로드 | 관리자 |

### 4.3 관리 페이지

| 메서드 | 경로 | 설명 | 권한 |
|--------|------|------|------|
| GET | `/admin_head` | 관리자 대시보드 | 관리자 |
| GET | `/admin_head/assets` | 자산 관리 페이지 | 관리자 |
| GET | `/director` | 감독관 페이지 | 감독관 |

### 4.4 파일 서빙

| 메서드 | 경로 | 설명 | 권한 |
|--------|------|------|------|
| GET | `/asset/*` | 자산 파일 접근 | 공개 |
| GET | `/static/*` | 정적 파일 접근 | 공개 |
| GET | `/project_list.yaml` | 프로젝트 목록 파일 | 공개 |

## 5. 데이터베이스 스키마

### 5.1 assets 컬렉션

```json
{
  "_id": "ObjectId",
  "curriculum": "string",    // 교육과정명
  "month": "string",         // 월 정보
  "book_id": "string",       // 교재 ID
  "covers": ["string"],      // 표지 이미지 파일 경로들
  "subtitles": [             // 자막 데이터
    {
      "page_num": "number",
      "sentence_num": "number", 
      "text": "string"
    }
  ],
  "youtube_links": [         // 유튜브 링크들
    {
      "thumbnail_file": "string",
      "youtube_url": "string",
      "title": "string?"
    }
  ],
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

## 6. 파일 시스템 구조

### 6.1 자산 폴더 구조

```
asset/
├── {curriculum}/          # 교육과정별 폴더
│   └── {month}/          # 월별 폴더
│       ├── cover/        # 표지 이미지
│       ├── subtitle/     # 자막 파일
│       ├── thumbnail/    # 썸네일 이미지
│       ├── youtube/      # 유튜브 관련 파일
│       ├── data.json     # 자산 메타데이터
│       └── subtitle.json # 자막 데이터
└── uploads/              # 임시 업로드 폴더
```

### 6.2 정적 파일 구조

```
static/
├── css/
│   └── admin.css         # 관리자 스타일
├── js/
│   └── admin.js          # 관리자 JavaScript
└── images/               # 이미지 파일들
```

## 7. 보안 및 인증

### 7.1 JWT 토큰 구조

```json
{
  "username": "string",     // 사용자명
  "role": "string",        // 사용자 역할
  "exp": "number"          // 만료 시간
}
```

### 7.2 역할 기반 권한

- **HEAD_OFFICE**: 최고 관리자 권한
- **REGIONAL_MANAGER**: 지역 관리자 권한  
- **DIRECTOR**: 감독관 권한

### 7.3 파일 업로드 보안

- 허용된 파일 타입: JPEG, JPG, PNG, WEBP
- 파일명 새니타이징 (타임스탬프 추가)
- 업로드 크기 제한

## 8. 로깅 및 모니터링

### 8.1 로깅 수준
- `ERROR`: 오류 및 예외 상황
- `WARN`: 경고 메시지
- `INFO`: 일반 정보 메시지
- `DEBUG`: 디버깅 정보

### 8.2 주요 로깅 포인트
- 사용자 인증 시도
- 자산 생성/수정/삭제
- 파일 업로드/이동
- 데이터베이스 작업
- 서버 시작/종료

## 9. 환경 설정

### 9.1 필수 환경 변수

- `MONGO_URI`: MongoDB 연결 문자열
- `APP_DATABASE_URL`: 데이터베이스 URL 오버라이드
- `APP_SERVER_HOST`: 서버 호스트 오버라이드
- `APP_SERVER_PORT`: 서버 포트 오버라이드

### 9.2 설정 파일 (config.yaml)

```yaml
app:
  name: "reengkigo"
  version: "1.0.0"
  debug: true

database:
  url: "mongodb://localhost:27017"
  name: "admin_system"

server:
  host: "0.0.0.0"
  port: 3000
```

## 10. 개발 및 배포

### 10.1 개발 명령어

```bash
# 개발 모드 실행
cargo run

# 테스트 실행
cargo test

# 코드 포맷팅
cargo fmt

# 린팅
cargo clippy

# 빌드
cargo build --release
```

### 10.2 Docker 지원

프로젝트는 Docker 및 docker-compose를 통한 컨테이너화를 지원합니다.

```bash
# Docker Compose로 실행
docker-compose up

# 개별 컨테이너 빌드
docker build -t reengkigo-admin .
```

이 문서는 ReengKigo 관리자 서버의 전체적인 구조와 기능을 설명합니다. 각 모듈과 함수의 상세한 구현 내용은 소스 코드를 참조하시기 바랍니다.