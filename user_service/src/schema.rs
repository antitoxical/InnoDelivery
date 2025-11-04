// @generated automatically by Diesel CLI.

diesel::table! {
    couriers (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 50]
        status -> Varchar,
        rating -> Float4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        is_blocked -> Bool,
        is_deleted -> Bool,
        rating_sum -> Float8,
        rating_count -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 50]
        phone_number -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        #[max_length = 50]
        role -> Varchar,
        #[max_length = 255]
        favorite_address -> Nullable<Varchar>,
        is_blocked -> Bool,
        is_deleted -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(couriers -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(couriers, users,);
