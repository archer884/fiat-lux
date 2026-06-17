use tantivy::schema::Field;

#[derive(Clone, Copy)]
pub struct SearchFields {
    pub translation: Field,
    pub location: Field,
    pub content: Field,
}
