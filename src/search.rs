use tantivy::schema::Field;

pub struct SearchFields {
    pub translation: Field,
    pub location: Field,
    pub content: Field,
}
