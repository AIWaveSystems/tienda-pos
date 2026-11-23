use uuid::Uuid;

pub fn new_id() -> String {
    let mut id = Uuid::new_v4().simple().to_string();
    id.truncate(12);
    id
}
