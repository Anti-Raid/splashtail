
pub enum ObjectStore {
    S3(object_store::aws::AmazonS3),
    Local(object_store::local::LocalFileSystem),
}

impl ObjectStore {
    pub fn get(&self) -> &dyn object_store::ObjectStore {
        match self {
            ObjectStore::S3(store) => store,
            ObjectStore::Local(store) => store,
        }
    }
}
