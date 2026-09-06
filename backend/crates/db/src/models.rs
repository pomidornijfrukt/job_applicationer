use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::test)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Test {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub am_i_tripping: bool,
}