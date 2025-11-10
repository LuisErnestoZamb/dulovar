// @generated automatically by Diesel CLI.

diesel::table! {
    alerts (id) {
        id -> Nullable<Integer>,
        first_name -> Nullable<Text>,
        last_name -> Nullable<Text>,
        description -> Nullable<Text>,
        yob -> Nullable<Integer>,
        url_1 -> Nullable<Text>,
        url_2 -> Nullable<Text>,
        url_3 -> Nullable<Text>,
        country -> Nullable<Text>,
        type_alert -> Nullable<Text>,
        name_alert -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    photos (id) {
        id -> Nullable<Integer>,
        alert_id -> Nullable<Integer>,
        url -> Nullable<Text>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::joinable!(photos -> alerts (alert_id));

diesel::allow_tables_to_appear_in_same_query!(alerts, photos,);
