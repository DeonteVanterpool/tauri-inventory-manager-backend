// @generated automatically by Diesel CLI.

diesel::table! {
    brands (id) {
        id -> Int4,
        name -> Text,
        products -> Array<Nullable<Int4>>,
    }
}

diesel::table! {
    categories (id) {
        id -> Int4,
        products -> Array<Nullable<Int4>>,
        name -> Text,
    }
}

diesel::table! {
    pending_orders (id) {
        id -> Int4,
        product_id -> Int4,
        amount -> Float8,
    }
}

diesel::table! {
    permissions (user_id) {
        user_id -> Int4,
        admin -> Bool,
        view_pending -> Bool,
        view_received -> Bool,
        edit_pending -> Bool,
        create_orders -> Bool,
        edit_received -> Bool,
        view_products -> Bool,
        edit_products -> Bool,
        remove_orders -> Bool,
        view_suppliers -> Bool,
    }
}

diesel::table! {
    preferences (user_id) {
        user_id -> Int4,
    }
}

diesel::table! {
    products (id) {
        id -> Int4,
        upc -> Text,
        name -> Text,
        description -> Text,
        amount -> Float8,
        case_size -> Nullable<Int4>,
        measure_by_weight -> Bool,
        cost_price_per_unit -> Numeric,
        selling_price_per_unit -> Numeric,
        sale_end -> Nullable<Timestamp>,
        buy_level -> Nullable<Float8>,
        sale_price -> Nullable<Numeric>,
    }
}

diesel::table! {
    received_orders (id) {
        id -> Int4,
        received -> Nullable<Timestamp>,
        product_id -> Int4,
        gross_amount -> Float8,
        actually_received -> Float8,
        damaged -> Float8,
    }
}

diesel::table! {
    suppliers (id) {
        id -> Int4,
        products -> Array<Nullable<Int4>>,
        name -> Text,
        phone_number -> Nullable<Text>,
        email -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Text,
        email -> Text,
        password -> Varchar,
    }
}

diesel::joinable!(pending_orders -> products (product_id));
diesel::joinable!(permissions -> users (user_id));
diesel::joinable!(preferences -> users (user_id));
diesel::joinable!(received_orders -> products (product_id));

diesel::allow_tables_to_appear_in_same_query!(
    brands,
    categories,
    pending_orders,
    permissions,
    preferences,
    products,
    received_orders,
    suppliers,
    users,
);
