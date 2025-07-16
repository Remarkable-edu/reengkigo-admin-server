use anyhow::{Ok, Result};
use mongodb::{
    bson::{self, doc, oid::ObjectId},
    options::ClientOptions,
    Client, Collection,
};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// C: 새로운 Asset 문서를 생성하고, 생성된 Asset의 ID를 반환합니다.
async fn create_asset(collection: &Collection<Asset>) -> Result<ObjectId> {
    // tracing::info!("--- 1. Creating new asset ---");
    let new_asset = Asset {
        // 최솽위라 고유 id 생성안해도 됌
        id: None,
        book: vec![Book {
            // 최상위가 아니라 고유 id 생성
            book_id: Some("R1R".to_string()), // <<< Book에 대한 고유 ID를 여기서 생성
            cover_img: "initial/cover.jpg".to_string(),
            video_content: vec![VideoContent {
                img: "initial/video.jpg".to_string(),
                youtube_url: "http://initial-test.com".to_string(),
            }],
        }],
    };

    let result = collection.insert_one(&new_asset, None).await?;
    let asset_id = result.inserted_id.as_object_id().unwrap();
    // tracing::info!("✅ Asset created with ID: {}", asset_id);
    Ok(asset_id)
}

// R: ID로 특정 Asset 문서를 찾아 반환합니다.
async fn find_asset_by_id(collection: &Collection<Asset>, asset_id: &ObjectId) -> Result<Option<Asset>> {
    // tracing::info!("--- 2. Finding asset by ID ---");
    let asset = collection.find_one(doc! {"_id": asset_id}, None).await?;
    // tracing::info!("✅ Found asset: {:?}", asset);
    Ok(asset)
}

// R: 특정 book_id를 가진 Book 하나만 찾기
async fn find_book_by_id(collection: &Collection<Asset>, book_id: String) -> Result<Option<Book>> {
    // tracing::info!("--- 2.5. Finding book by book_id ---");

    let filter = doc! {
        "book.book_id": &book_id
    };

    let projection = doc! {
        // filter에서 거른 첫번째 요소만 결과에 포함
        "book.$": 1 // book 배열 중 첫 번째 매칭 항목만 추출
    };

    let find_result = collection
        .find_one(
            filter,
            mongodb::options::FindOneOptions::builder()
                .projection(projection)
                .build(),
        )
        .await?;

    if let Some(asset) = find_result {
        // MongoDB는 매칭된 book 하나만 가져오게 projection함
        if let Some(book_list) = asset.book.get(0) {
            return Ok(Some(book_list.clone()));
        }
    }

    Ok(None)
}


// U (1): 특정 Asset에 새로운 Book을 추가합니다.
async fn add_book_to_asset(collection: &Collection<Asset>, asset_id: &ObjectId) -> Result<()> {
    tracing::info!("--- 3. Adding a new book to asset ---");
    let new_book = Book {
        book_id: Some("R1O".to_string()),
        cover_img: "second/book.jpg".to_string(),
        video_content: vec![],
    };

    let filter = doc! {"_id": asset_id};
    let update = doc! {"$push": {"book": bson::to_bson(&new_book)?}};
    collection.update_one(filter, update, None).await?;
    tracing::info!("✅ Pushed a new book to asset");
    Ok(())
}

// U (2): 특정 Book에 새로운 VideoContent를 추가합니다.
async fn add_video_to_book(
    collection: &Collection<Asset>,
    asset_id: &ObjectId,
    book_id: &str,
) -> Result<()> {
    // tracing::info!("--- 4. Adding video content to a specific book ---");
    let new_video = VideoContent {
        img: "updated/video.jpg".to_string(),
        youtube_url: "http://updated-video.com".to_string(),
    };

    let filter = doc! {"_id": asset_id, "book.book_id": book_id};
    let update = doc! {"$push": {"book.$.video_content": bson::to_bson(&new_video)?}};
    collection.update_one(filter, update, None).await?;
    tracing::info!("✅ Pushed new video content to book ID: {}", book_id);
    Ok(())
}

// D: 특정 Asset 문서를 삭제합니다.
async fn delete_asset(collection: &Collection<Asset>, asset_id: &ObjectId) -> Result<()> {
    // tracing::info!("--- 5. Deleting asset ---");
    collection.delete_one(doc! {"_id": asset_id}, None).await?;
    tracing::info!("✅ Deleted asset with ID: {}", asset_id);
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct VideoContent {
    img: String,
    youtube_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Book {
    #[serde(rename= "book_id", skip_serializing_if = "Option::is_none")]
    book_id: Option<String>,
    cover_img: String,
    video_content: Vec<VideoContent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Asset {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    // 필드 이름도 book
    book: Vec<Book>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- 초기 설정 ---
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "reengkigo-admin-app=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv::dotenv().ok();
    tracing::info!("Starting MongoDB CRUD Example with Functions");

    let client_uri = std::env::var("MONGO_URI").expect("failed to initialize db");
    let mut client_options = ClientOptions::parse(&client_uri).await?;
    client_options.app_name = Some("test-server".to_string());
    let client = Client::with_options(client_options)?;
    let db = client.database("test_db");
    let collection = db.collection::<Asset>("assets");

    // --- CRUD 시나리오 실행 ---

    // 1. 새로운 Asset 생성
    let asset_id = create_asset(&collection).await?;

    // 2. 생성된 Asset 조회 및 첫 번째 Book의 ID 확보
    let created_asset = find_asset_by_id(&collection, &asset_id)
        .await?
        .expect("Created asset not found");
    
    // 업데이트에 사용할 book_id 추출
    let first_book_id = created_asset.book.first().unwrap().book_id.clone().unwrap();

    println!("first book id is {:?}", first_book_id);

    // 3. Asset에 새로운 Book 추가
    add_book_to_asset(&collection, &asset_id).await?;

    // 4. 첫 번째 Book에 VideoContent 추가
    add_video_to_book(&collection, &asset_id, &first_book_id).await?;

    let maybe_book = find_book_by_id(&collection, "R1R".to_string()).await?;
    if let Some(book) = maybe_book {
        tracing::info!("found book: {:?}", book);
    } else {

    };
    
    // 5. 모든 업데이트가 적용된 최종 Asset 조회
    tracing::info!("--- Final asset state ---");
    find_asset_by_id(&collection, &asset_id).await?;

    // 6. 생성했던 Asset 삭제 (정리)
    delete_asset(&collection, &asset_id).await?;

    Ok(())
}