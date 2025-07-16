use mongodb::{Client, Database as MongoDatabase, options::ClientOptions};
use tracing::info;

/// Database connection wrapper for MongoDB
/// 
/// This struct provides a centralized way to manage MongoDB connections
/// and database operations throughout the application. It wraps both the
/// MongoDB client and a specific database instance.
#[derive(Clone)]
pub struct Database {
    /// MongoDB client instance for connection management
    pub client: Client,
    /// Specific database instance for this application
    pub database: MongoDatabase,
}

impl Database {
    /// Create a new database connection to MongoDB
    /// 
    /// This function establishes a connection to MongoDB and performs
    /// initial setup including:
    /// - Parsing the connection URL
    /// - Setting application name for better monitoring
    /// - Testing the connection to ensure it's working
    /// 
    /// # Parameters
    /// - `database_url`: MongoDB connection string (e.g., "mongodb://localhost:27017")
    /// - `db_name`: Name of the database to connect to
    /// 
    /// # Returns
    /// - `Ok(Database)`: Successfully connected to MongoDB
    /// - `Err(mongodb::error::Error)`: Connection failed
    /// 
    /// # Example
    /// ```
    /// let db = Database::new("mongodb://localhost:27017", "admin_system").await?;
    /// ```
    pub async fn new(database_url: &str, db_name: &str) -> Result<Self, mongodb::error::Error> {
        info!("Connecting to MongoDB: {}", database_url);
        
        // Parse connection string and set client options
        let mut client_options = ClientOptions::parse(database_url).await?;
        client_options.app_name = Some("reengkigo-admin-system".to_string());
        
        // Create client and select database
        let client = Client::with_options(client_options)?;
        let database = client.database(db_name);
        
        // Test the connection by listing databases
        client.list_database_names(None, None).await?;
        
        info!("MongoDB connection established successfully");
        
        Ok(Self { client, database })
    }
}

