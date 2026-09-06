diesel::table! {
    test (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
        am_i_tripping -> Bool,
    }
}