# ReengKigo 관리자 서버 시스템 플로우 문서

## 1. 시스템 전체 플로우

### 1.1 서버 시작 프로세스

```mermaid
flowchart TD
    A[애플리케이션 시작] --> B[환경 변수 로드<br/>.env 파일]
    B --> C[로깅 시스템 초기화<br/>tracing 설정]
    C --> D[설정 파일 로드<br/>config.yaml + 환경변수]
    D --> E[MongoDB 연결<br/>데이터베이스 초기화]
    E --> F[애플리케이션 상태 구성<br/>AppState 생성]
    F --> G[라우터 생성<br/>HTTP 엔드포인트 설정]
    G --> H[서버 시작<br/>TCP 리스너 바인드]
    H --> I[우아한 종료 신호 대기<br/>SIGTERM, Ctrl+C]
```

**주요 구성 요소:**
- **설정 우선순위**: 기본값 → config.yaml → 환경변수 (APP_ 접두사)
- **데이터베이스**: MongoDB 비동기 연결
- **관찰성**: tracing을 통한 구조화된 로깅
- **우아한 종료**: 진행 중인 요청 완료 후 종료

### 1.2 요청 처리 플로우

```mermaid
flowchart TD
    A[HTTP 요청 수신] --> B{요청 타입 분류}
    
    B -->|공개 경로| C[정적 파일/로그인]
    B -->|보호된 경로| D[인증 미들웨어]
    
    D --> E{토큰 검증}
    E -->|실패| F[인증 오류 응답<br/>401/302 리다이렉트]
    E -->|성공| G[사용자 정보 추출]
    
    G --> H[권한 확인 미들웨어]
    H --> I{역할 검증}
    I -->|권한 부족| J[권한 오류 응답<br/>403/302 리다이렉트]
    I -->|권한 충족| K[핸들러 실행]
    
    C --> K
    K --> L[비즈니스 로직 처리]
    L --> M[데이터베이스 작업]
    M --> N[응답 생성]
    N --> O[클라이언트 응답]
```

## 2. 인증 및 권한 관리 플로우

### 2.1 사용자 로그인 플로우

```mermaid
sequenceDiagram
    participant C as 클라이언트
    participant H as AuthHandler
    participant S as AuthService
    participant DB as 하드코딩된 사용자 데이터
    
    C->>H: POST /login (계정, 비밀번호)
    H->>S: authenticate_user()
    S->>DB: 사용자 검증
    
    alt 인증 성공
        DB-->>S: AdminUser 반환
        S-->>H: Some(AdminUser)
        H->>S: generate_admin_token()
        S-->>H: JWT 토큰
        H-->>C: 200 OK + 토큰 + 쿠키 설정
    else 인증 실패
        DB-->>S: None
        S-->>H: None
        H-->>C: 401 Unauthorized
    end
```

**인증 처리 세부사항:**
- 사용자 자격증명은 현재 하드코딩됨 (추후 데이터베이스 연동 예정)
- JWT 토큰에는 username, role, exp 정보 포함
- 두 개의 쿠키 설정:
  - `auth_token`: HttpOnly 쿠키 (실제 인증용)
  - `auth_status`: JavaScript 접근 가능 (상태 확인용)

### 2.2 권한 검증 플로우

```mermaid
flowchart TD
    A[보호된 엔드포인트 접근] --> B[AuthMiddleware.auth_middleware]
    B --> C{토큰 추출}
    
    C -->|Authorization 헤더| D[Bearer 토큰 파싱]
    C -->|Cookie 헤더| E[auth_token 쿠키 파싱]
    C -->|토큰 없음| F[401 Unauthorized]
    
    D --> G[JWT 토큰 검증]
    E --> G
    G --> H{토큰 유효성}
    H -->|유효하지 않음| F
    H -->|유효함| I[클레임 추출]
    
    I --> J[AdminUser 생성]
    J --> K[요청 확장에 사용자 정보 저장]
    K --> L[역할별 권한 미들웨어]
    
    L --> M{요구 권한 확인}
    M -->|Admin 필요| N[require_admin_role]
    M -->|Director 필요| O[require_director_role]
    
    N --> P{HEAD_OFFICE 또는<br/>REGIONAL_MANAGER?}
    O --> Q{DIRECTOR?}
    
    P -->|Yes| R[다음 핸들러 실행]
    P -->|No| S[403 Forbidden]
    Q -->|Yes| R
    Q -->|No| S
```

**역할별 권한:**
- **HEAD_OFFICE**: 최고 관리자 (모든 기능 접근)
- **REGIONAL_MANAGER**: 지역 관리자 (관리자 기능 접근)
- **DIRECTOR**: 감독관 (감독관 기능만 접근)

## 3. 자산 관리 플로우

### 3.1 자산 생성 플로우

```mermaid
sequenceDiagram
    participant C as 클라이언트
    participant H as AdminHeadHandler
    participant S as AssetService
    participant FS as 파일시스템
    participant DB as MongoDB
    participant YAML as project_list.yaml
    
    C->>H: POST /api/assets (자산 데이터)
    H->>S: create_asset()
    
    S->>DB: 중복 자산 확인
    alt 중복 존재
        DB-->>S: 기존 자산 발견
        S-->>H: Error: 이미 존재함
        H-->>C: 500 Internal Server Error
    else 중복 없음
        S->>YAML: book_id 매핑 조회
        YAML-->>S: book_id 반환
        
        S->>FS: 업로드된 파일 이동
        S->>FS: 폴더 구조 생성
        Note over FS: asset/{curriculum}/{month}/<br/>├── cover/<br/>├── subtitle/<br/>├── thumbnail/<br/>└── youtube/
        
        S->>FS: JSON 파일 생성
        Note over FS: - data.json<br/>- subtitle.json<br/>- youtube_links.json
        
        S->>DB: 자산 데이터 저장
        DB-->>S: 삽입된 자산 ID
        S-->>H: AssetResponse
        H-->>C: 200 OK + 자산 정보
    end
```

**파일 시스템 구조:**
```
asset/
├── {curriculum}/
│   └── {month}/
│       ├── cover/           # 표지 이미지
│       ├── subtitle/        # 자막 텍스트 파일
│       ├── thumbnail/       # 썸네일 이미지
│       ├── youtube/         # 유튜브 관련 파일
│       ├── data.json        # 자산 메타데이터
│       └── subtitle.json    # 자막 데이터
```

### 3.2 파일 업로드 플로우

```mermaid
flowchart TD
    A[파일 업로드 요청<br/>POST /api/upload] --> B[multipart/form-data 파싱]
    B --> C{파일 타입 검증}
    
    C -->|허용되지 않는 타입| D[400 Bad Request<br/>INVALID_FILE_TYPE]
    C -->|허용된 타입<br/>JPEG,PNG,WEBP| E[파일명 새니타이징]
    
    E --> F[타임스탬프 추가<br/>timestamp_filename.ext]
    F --> G[임시 업로드 폴더에 저장<br/>asset/uploads/]
    G --> H[파일 경로 반환<br/>/asset/uploads/...]
    
    H --> I[자산 생성/수정 시<br/>적절한 폴더로 이동]
```

**보안 고려사항:**
- 허용된 MIME 타입: `image/jpeg`, `image/jpg`, `image/png`, `image/webp`
- 파일명 새니타이징: 영숫자, 점, 언더스코어, 하이픈만 허용
- 타임스탬프 추가로 파일명 충돌 방지

### 3.3 자산 수정 플로우

```mermaid
sequenceDiagram
    participant C as 클라이언트
    participant H as AdminHeadHandler
    participant S as AssetService
    participant FS as 파일시스템
    participant DB as MongoDB
    
    C->>H: PUT /api/assets/{id} (수정 데이터)
    H->>S: update_asset()
    
    S->>DB: 기존 자산 조회
    alt 자산 없음
        DB-->>S: None
        S-->>H: Error: 자산 없음
        H-->>C: 404 Not Found
    else 자산 존재
        DB-->>S: 기존 자산 데이터
        
        alt 업로드된 파일 존재
            S->>FS: uploads 폴더 확인
            S->>FS: 기존 파일 교체
            Note over FS: 원본 파일명 유지하면서<br/>내용만 교체
        else 새 파일 경로만 제공
            S->>FS: 파일 경로 업데이트
        end
        
        S->>FS: JSON 파일 업데이트
        S->>DB: 데이터베이스 업데이트
        DB-->>S: 업데이트 완료
        S-->>H: AssetResponse
        H-->>C: 200 OK + 수정된 자산 정보
    end
```

### 3.4 자산 삭제 플로우

```mermaid
flowchart TD
    A[자산 삭제 요청<br/>DELETE /api/assets/{id}] --> B[자산 ID 검증]
    B --> C[데이터베이스에서 자산 조회]
    
    C --> D{자산 존재?}
    D -->|No| E[404 Not Found]
    D -->|Yes| F[데이터베이스에서 삭제]
    
    F --> G[자산 폴더 경로 확인<br/>asset/{curriculum}/{month}]
    G --> H[전체 폴더 삭제]
    H --> I{파일 시스템 삭제 성공?}
    
    I -->|실패| J[경고 로그 기록<br/>데이터베이스는 이미 삭제됨]
    I -->|성공| K[성공 로그 기록]
    
    J --> L[204 No Content]
    K --> L
```

## 4. 데이터베이스 연동 플로우

### 4.1 MongoDB 연결 관리

```mermaid
flowchart TD
    A[애플리케이션 시작] --> B[데이터베이스 설정 로드]
    B --> C[MongoDB 클라이언트 생성]
    C --> D[연결 테스트]
    
    D --> E{연결 성공?}
    E -->|실패| F[오류 로그 및 종료]
    E -->|성공| G[Database 구조체 생성]
    
    G --> H[AppState에 포함]
    H --> I[핸들러에서 사용]
    
    I --> J[컬렉션 접근<br/>Collection<T>]
    J --> K[CRUD 작업 수행]
```

**MongoDB 작업 패턴:**
```rust
// 컬렉션 접근
let collection: Collection<Asset> = db.database.collection("assets");

// 조회
let filter = doc! {"curriculum": "jelly"};
let cursor = collection.find(filter, None).await?;

// 삽입
let result = collection.insert_one(&asset, None).await?;

// 업데이트
let update = doc! {"$set": bson::to_bson(&asset)?};
collection.update_one(filter, update, None).await?;

// 삭제
collection.delete_one(filter, None).await?;
```

### 4.2 데이터 변환 플로우

```mermaid
flowchart LR
    A[HTTP 요청<br/>JSON] --> B[DTO<br/>CreateAssetRequest]
    B --> C[비즈니스 로직<br/>AssetService]
    C --> D[모델<br/>Asset struct]
    D --> E[MongoDB<br/>BSON 직렬화]
    
    E --> F[데이터베이스 저장]
    F --> G[조회 결과<br/>Asset struct]
    G --> H[응답 변환<br/>AssetResponse]
    H --> I[HTTP 응답<br/>JSON]
```

**데이터 계층 구조:**
- **DTO (Data Transfer Object)**: API 요청/응답 형식
- **Model**: 비즈니스 로직과 데이터베이스 구조
- **BSON**: MongoDB 직렬화 형식

## 5. 파일 시스템 관리 플로우

### 5.1 폴더 구조 생성

```mermaid
flowchart TD
    A[자산 생성 요청] --> B[Asset 모델 생성]
    B --> C[폴더 경로 계산<br/>asset/{curriculum}/{month}]
    
    C --> D[하위 폴더 생성]
    D --> E[cover/ 폴더]
    D --> F[subtitle/ 폴더]
    D --> G[thumbnail/ 폴더]
    D --> H[youtube/ 폴더]
    
    E --> I[파일 이동 및 저장]
    F --> I
    G --> I
    H --> I
    
    I --> J[메타데이터 파일 생성<br/>data.json, subtitle.json]
```

### 5.2 파일 이동 및 관리

```mermaid
sequenceDiagram
    participant Upload as 업로드 폴더<br/>asset/uploads/
    participant Service as AssetService
    participant Target as 대상 폴더<br/>asset/{curr}/{month}/
    
    Note over Upload: 1710675600_cover.png
    Service->>Upload: 업로드된 파일 확인
    Upload-->>Service: 파일 존재 확인
    
    Service->>Target: 대상 폴더 생성
    Service->>Service: 원본 파일명 결정<br/>cover/1_J1R.png
    
    Service->>Target: 파일 복사
    Note over Target: cover/1_J1R.png
    
    Service->>Upload: 임시 파일 삭제
    Service->>Service: 상대 경로 반환<br/>"cover/1_J1R.png"
```

**파일 이동 규칙:**
1. 업로드 시: `asset/uploads/timestamp_filename`
2. 이동 시: `asset/{curriculum}/{month}/{type}/filename`
3. 데이터베이스 저장: `{type}/filename` (상대 경로)

## 6. 오류 처리 및 로깅 플로우

### 6.1 오류 처리 계층

```mermaid
flowchart TD
    A[요청 처리 중 오류 발생] --> B{오류 타입 분류}
    
    B -->|인증 오류| C[AuthMiddleware]
    B -->|권한 오류| D[권한 미들웨어]
    B -->|비즈니스 로직 오류| E[Service Layer]
    B -->|데이터베이스 오류| F[Database Layer]
    B -->|파일 시스템 오류| G[File System]
    
    C --> H[401/302 응답]
    D --> I[403/302 응답]
    E --> J[500 응답 + 오류 메시지]
    F --> K[500 응답 + 데이터베이스 오류]
    G --> L[500 응답 + 파일 오류]
    
    H --> M[클라이언트 응답]
    I --> M
    J --> M
    K --> M
    L --> M
    
    J --> N[오류 로깅]
    K --> N
    L --> N
```

### 6.2 로깅 플로우

```mermaid
flowchart LR
    A[애플리케이션 이벤트] --> B{로그 레벨}
    
    B -->|ERROR| C[오류 및 예외상황]
    B -->|WARN| D[경고 메시지]
    B -->|INFO| E[일반 정보]
    B -->|DEBUG| F[디버깅 정보]
    
    C --> G[구조화된 로그 출력]
    D --> G
    E --> G
    F --> G
    
    G --> H[stdout/stderr]
    G --> I[파일 출력<br/>server.log]
```

**주요 로깅 포인트:**
- 사용자 인증 시도 및 결과
- 자산 CRUD 작업
- 파일 업로드/이동/삭제
- 데이터베이스 연결 및 쿼리
- 서버 시작/종료
- 오류 및 예외 상황

## 7. 설정 관리 플로우

### 7.1 계층적 설정 로딩

```mermaid
flowchart TD
    A[설정 로딩 시작] --> B[기본값 설정<br/>AppConfig::default()]
    B --> C[config.yaml 파일 로드]
    C --> D{파일 존재?}
    
    D -->|Yes| E[YAML 파싱 및 병합]
    D -->|No| F[기본값 유지]
    
    E --> G[환경변수 로드<br/>APP_ 접두사]
    F --> G
    
    G --> H[환경변수 파싱 및 병합]
    H --> I[최종 설정 구성체 생성]
    I --> J[설정 검증]
    
    J --> K{설정 유효?}
    K -->|Yes| L[AppConfig 반환]
    K -->|No| M[오류 반환 및 종료]
```

**설정 우선순위 (높은 순):**
1. 환경변수 (`APP_DATABASE_URL`, `APP_SERVER_PORT` 등)
2. config.yaml 파일
3. 코드 내 기본값

**예시 설정 오버라이드:**
```yaml
# config.yaml
server:
  port: 8080
```

```bash
# 환경변수 (최고 우선순위)
export APP_SERVER_PORT=3000
export APP_DATABASE_URL=mongodb://localhost:27017
```

## 8. 보안 플로우

### 8.1 JWT 토큰 생명주기

```mermaid
sequenceDiagram
    participant C as 클라이언트
    participant S as 서버
    participant JWT as JWT 서비스
    
    C->>S: 로그인 요청
    S->>JWT: 토큰 생성
    Note over JWT: username, role, exp 포함
    JWT-->>S: JWT 토큰
    S-->>C: 토큰 + HttpOnly 쿠키
    
    loop 보호된 요청
        C->>S: API 요청 + 토큰
        S->>JWT: 토큰 검증
        alt 토큰 유효
            JWT-->>S: 클레임 반환
            S-->>C: 요청 처리 결과
        else 토큰 무효/만료
            JWT-->>S: 검증 실패
            S-->>C: 401 Unauthorized
        end
    end
```

### 8.2 파일 업로드 보안

```mermaid
flowchart TD
    A[파일 업로드] --> B[MIME 타입 검증]
    B --> C{허용된 타입?}
    
    C -->|No| D[업로드 거부<br/>400 Bad Request]
    C -->|Yes| E[파일명 새니타이징]
    
    E --> F[위험한 문자 제거<br/>../등 경로 조작 방지]
    F --> G[타임스탬프 추가<br/>충돌 방지]
    G --> H[안전한 경로에 저장<br/>asset/uploads/]
    
    H --> I[파일 크기 제한 확인]
    I --> J[업로드 완료]
```

**보안 조치:**
- MIME 타입 화이트리스트
- 파일명 새니타이징
- 경로 탐색 공격 방지
- 업로드 폴더 격리
- 파일 크기 제한 (multipart 설정)

이 시스템 플로우 문서는 ReengKigo 관리자 서버의 모든 주요 프로세스와 데이터 흐름을 설명합니다. 각 플로우의 세부 구현은 해당 소스 코드 모듈을 참조하시기 바랍니다.