use server_test::models::asset::{Asset, Book, VideoContent};

#[test]
fn test_asset_creation() {
    let video_content = VideoContent::new(
        "test_video.jpg".to_string(),
        "https://youtube.com/watch?v=test".to_string(),
    );
    
    let book = Book::new(
        Some("BOOK_001".to_string()),
        "January".to_string(),
        "cover.jpg".to_string(),
        vec![video_content],
    );
    
    let asset = Asset::new(vec![book]);
    
    assert!(asset.id.is_some());
    assert_eq!(asset.book.len(), 1);
    assert_eq!(asset.book[0].book_id, Some("BOOK_001".to_string()));
    assert_eq!(asset.book[0].month, "January");
    assert_eq!(asset.book[0].cover_img, "cover.jpg");
    assert_eq!(asset.book[0].video_content.len(), 1);
    assert_eq!(asset.book[0].video_content[0].youtube_url, "https://youtube.com/watch?v=test");
}

#[test]
fn test_book_creation() {
    let video_content = VideoContent::new(
        "video_thumb.jpg".to_string(),
        "https://youtube.com/watch?v=example".to_string(),
    );
    
    let book = Book::new(
        None,
        "February".to_string(),
        "february_cover.jpg".to_string(),
        vec![video_content],
    );
    
    assert_eq!(book.book_id, None);
    assert_eq!(book.month, "February");
    assert_eq!(book.cover_img, "february_cover.jpg");
    assert_eq!(book.video_content.len(), 1);
}

#[test]
fn test_video_content_creation() {
    let video_content = VideoContent::new(
        "thumbnail.jpg".to_string(),
        "https://youtube.com/watch?v=abc123".to_string(),
    );
    
    assert_eq!(video_content.video_img, "thumbnail.jpg");
    assert_eq!(video_content.youtube_url, "https://youtube.com/watch?v=abc123");
}

#[test]
fn test_asset_with_multiple_books() {
    let video1 = VideoContent::new(
        "video1.jpg".to_string(),
        "https://youtube.com/watch?v=video1".to_string(),
    );
    
    let video2 = VideoContent::new(
        "video2.jpg".to_string(),
        "https://youtube.com/watch?v=video2".to_string(),
    );
    
    let book1 = Book::new(
        Some("BOOK_001".to_string()),
        "January".to_string(),
        "cover1.jpg".to_string(),
        vec![video1],
    );
    
    let book2 = Book::new(
        Some("BOOK_002".to_string()),
        "February".to_string(),
        "cover2.jpg".to_string(),
        vec![video2],
    );
    
    let asset = Asset::new(vec![book1, book2]);
    
    assert_eq!(asset.book.len(), 2);
    assert_eq!(asset.book[0].book_id, Some("BOOK_001".to_string()));
    assert_eq!(asset.book[1].book_id, Some("BOOK_002".to_string()));
    assert_eq!(asset.book[0].month, "January");
    assert_eq!(asset.book[1].month, "February");
}

#[test]
fn test_book_with_no_video_content() {
    let book = Book::new(
        Some("BOOK_EMPTY".to_string()),
        "March".to_string(),
        "march_cover.jpg".to_string(),
        vec![],
    );
    
    assert_eq!(book.book_id, Some("BOOK_EMPTY".to_string()));
    assert_eq!(book.month, "March");
    assert_eq!(book.video_content.len(), 0);
}

#[test]
fn test_book_with_multiple_videos() {
    let videos = vec![
        VideoContent::new(
            "video1.jpg".to_string(),
            "https://youtube.com/watch?v=lesson1".to_string(),
        ),
        VideoContent::new(
            "video2.jpg".to_string(),
            "https://youtube.com/watch?v=lesson2".to_string(),
        ),
        VideoContent::new(
            "video3.jpg".to_string(),
            "https://youtube.com/watch?v=lesson3".to_string(),
        ),
    ];
    
    let book = Book::new(
        Some("MULTI_VIDEO_BOOK".to_string()),
        "April".to_string(),
        "april_cover.jpg".to_string(),
        videos,
    );
    
    assert_eq!(book.video_content.len(), 3);
    assert_eq!(book.video_content[0].youtube_url, "https://youtube.com/watch?v=lesson1");
    assert_eq!(book.video_content[1].youtube_url, "https://youtube.com/watch?v=lesson2");
    assert_eq!(book.video_content[2].youtube_url, "https://youtube.com/watch?v=lesson3");
}