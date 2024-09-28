pub mod database;
pub mod models;
pub mod schema;

#[macro_use]
extern crate rocket;
extern crate diesel;

use crate::models::{BrandBuilder, CategoryBuilder, SupplierBuilder};
use std::str::FromStr;

use crate::models::{Brand, ProductBuilder};
use std::env;
use bcrypt::hash;
use bcrypt::verify;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use database::{Manager, Pool};
use diesel::prelude::*;
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use models::Category;
use models::Product;
use models::Supplier;
use models::{PendingOrder, ReceivedOrder, User};
use rocket::serde::json::Json;
use rocket::{
    http::Status,
    request::{FromRequest, Request},
    State,
};

pub struct ServerState {
    pub db_pool: Pool,
}

#[derive(Debug)]
struct AuthGuard {
    user: User,
}

const DEFAULT_COST: usize = 10;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthGuard {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
        let state = req.rocket().state::<ServerState>().unwrap();
        let given_name = req.headers().get_one("username").unwrap().to_string();
        let given_password = req.headers().get_one("password").unwrap().to_string();
        let mut conn = state.db_pool.get().await.unwrap();

        use schema::users::dsl::*;
        println!("{}", given_name);
        let user = users
            .filter(name.eq(given_name))
            .first::<User>(conn.as_mut())
            .await
            .unwrap();
        let password_match = verify(given_password + &env::var("PEPPER").expect("PEPPER must be set"), &user.password).unwrap();

        if !password_match {
            return rocket::request::Outcome::Failure((Status::NonAuthoritativeInformation, ()));
        }

        rocket::request::Outcome::Success(Self { user })
    }
}

#[get("/user?<username>")]
async fn user(auth: AuthGuard, state: &State<ServerState>, username: String) -> Option<Json<User>> {
    let mut conn = state.db_pool.get().await.unwrap();
    let user = auth.user;
    println!("{:?}", user);
    println!("Hello");

    Some(Json(User::from_name(conn.as_mut(), &username).await))
}

#[get("/remove_product/<id>")]
async fn remove_product(_auth: AuthGuard, state: &State<ServerState>, id: i32) {
    let mut conn = state.db_pool.get().await.unwrap();

    Product::get(conn.as_mut(), id)
        .await
        .delete(conn.as_mut())
        .await;
}

#[get("/remove_pending_order/<id>")]
async fn remove_pending_order(_auth: AuthGuard, state: &State<ServerState>, id: i32) {
    let mut conn = state.db_pool.get().await.unwrap();

    PendingOrder::delete(conn.as_mut(), id).await;
}

#[get("/remove_received_order/<id>")]
async fn remove_received_order(_auth: AuthGuard, state: &State<ServerState>, id: i32) {
    let mut conn = state.db_pool.get().await.unwrap();

    ReceivedOrder::delete(conn.as_mut(), id).await;
}

#[get("/update_user/<user_info>")]
async fn update_user(_auth: AuthGuard, state: &State<ServerState>, user_info: String) {
    let user: User = serde_json::from_str(&user_info).unwrap();
    let mut conn = state.db_pool.get().await.unwrap();

    user.update(conn.as_mut()).await;
}

#[get("/update_product/<product_info>")]
async fn update_product(_auth: AuthGuard, state: &State<ServerState>, product_info: String) {
    let product: Product = serde_json::from_str(&product_info).unwrap();
    let mut conn = state.db_pool.get().await.unwrap();

    product.update(conn.as_mut()).await;
}

#[get("/remove_user/<id>")]
async fn remove_user(_auth: AuthGuard, state: &State<ServerState>, id: i32) {
    let mut conn = state.db_pool.get().await.unwrap();
    User::delete(conn.as_mut(), id).await;
}

#[get("/update_category?<category_info>")]
async fn update_category(_auth: AuthGuard, state: &State<ServerState>, category_info: String) {
    let category: Category = serde_json::from_str(&category_info).unwrap();
    let mut conn = state.db_pool.get().await.unwrap();

    category.update(conn.as_mut()).await;
}

#[get("/remove_category/<id>")]
async fn remove_category(_auth: AuthGuard, state: &State<ServerState>, id: i32) {
    let mut conn = state.db_pool.get().await.unwrap();
    Category::delete(conn.as_mut(), id).await;
}

#[get("/update_brand?<brand_info>")]
async fn update_brand(_auth: AuthGuard, state: &State<ServerState>, brand_info: String) {
    let brand: Brand = serde_json::from_str(&brand_info).unwrap();
    let mut conn = state.db_pool.get().await.unwrap();

    brand.update(conn.as_mut()).await;
}

#[get("/remove_brand/<id>")]
async fn remove_brand(_auth: AuthGuard, state: &State<ServerState>, id: i32) {
    let mut conn = state.db_pool.get().await.unwrap();
    Brand::delete(conn.as_mut(), id).await;
}

#[get("/update_supplier?<supplier_info>")]
async fn update_supplier(_auth: AuthGuard, state: &State<ServerState>, supplier_info: String) {
    let supplier: Supplier = serde_json::from_str(&supplier_info).unwrap();
    let mut conn = state.db_pool.get().await.unwrap();

    supplier.update(conn.as_mut()).await;
}

#[get("/remove_supplier/<id>")]
async fn remove_supplier(_auth: AuthGuard, state: &State<ServerState>, id: i32) {
    let mut conn = state.db_pool.get().await.unwrap();
    Supplier::delete(conn.as_mut(), id).await;
}

#[get("/add_product_supplier/<product_id>/<supplier_id>")]
async fn add_product_supplier(
    _auth: AuthGuard,
    state: &State<ServerState>,
    product_id: i32,
    supplier_id: i32,
) {
    let mut conn = state.db_pool.get().await.unwrap();
    let product = Product::get(conn.as_mut(), product_id).await;
    product.add_supplier(conn.as_mut(), supplier_id).await;
}

#[get("/add_product_brand/<product_id>/<brand_id>")]
async fn add_product_brand(
    _auth: AuthGuard,
    state: &State<ServerState>,
    product_id: i32,
    brand_id: i32,
) {
    let mut conn = state.db_pool.get().await.unwrap();
    let product = Product::get(conn.as_mut(), product_id).await;
    product.add_brand(conn.as_mut(), brand_id).await;
}

#[get("/add_product_category/<product_id>/<category_id>")]
async fn add_product_category(
    _auth: AuthGuard,
    state: &State<ServerState>,
    product_id: i32,
    category_id: i32,
) {
    let mut conn = state.db_pool.get().await.unwrap();
    let product = Product::get(conn.as_mut(), product_id).await;
    product.add_category(conn.as_mut(), category_id).await;
}

#[get("/pending_orders?<limit>&<offset>")]
async fn pending_orders(
    auth: AuthGuard,
    state: &State<ServerState>,
    limit: i64,
    offset: i64,
) -> Option<Json<Vec<PendingOrder>>> {
    let mut conn = state.db_pool.get().await.unwrap();
    use crate::schema::pending_orders::dsl::*;

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.view_pending {
        Some(Json(
            pending_orders
                .limit(limit)
                .offset(offset)
                .load(conn.as_mut())
                .await
                .unwrap(),
        ))
    } else {
        None
    }
}

#[get("/permissions")]
async fn permissions(auth: AuthGuard, state: &State<ServerState>) -> Json<models::Permission> {
    let mut conn = state.db_pool.get().await.unwrap();

    let user = auth.user;
    Json(user.get_permissions(conn.as_mut()).await)
}

#[get("/user_permissions/<user_id>")]
async fn user_permissions(
    auth: AuthGuard,
    state: &State<ServerState>,
    user_id: i32,
) -> Json<models::Permission> {
    let mut conn = state.db_pool.get().await.unwrap();

    let user = User::from_id(conn.as_mut(), user_id).await;

    Json(user.get_permissions(conn.as_mut()).await)
}

#[get("/received_orders?<limit>&<offset>")]
async fn received_orders(
    auth: AuthGuard,
    state: &State<ServerState>,
    limit: i64,
    offset: i64,
) -> Option<Json<Vec<ReceivedOrder>>> {
    let mut conn = state.db_pool.get().await.unwrap();
    use crate::schema::received_orders::dsl::*;

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.view_received {
        Some(Json(
            received_orders
                .limit(limit)
                .offset(offset)
                .load(conn.as_mut())
                .await
                .unwrap(),
        ))
    } else {
        None
    }
}

#[get("/new_pending_order?<product_id>&<amount>")]
async fn new_pending_order(
    auth: AuthGuard,
    state: &State<ServerState>,
    product_id: i32,
    amount: f64,
) -> Json<i32> {
    let mut conn = state.db_pool.get().await.unwrap();
    use models::PendingOrderBuilder;

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.create_orders {
        return Json(
            PendingOrderBuilder::new(product_id, amount)
                .build(conn.as_mut())
                .await,
        );
    }
    panic!("Wrong perms")
}

#[get("/mark_order_as_received?<order_id>&<date>&<actually_received>&<damaged>")]
async fn mark_order_as_received(
    auth: AuthGuard,
    state: &State<ServerState>,
    order_id: i32,
    date: i64,
    actually_received: f64,
    damaged: f64,
) -> Json<i32> {
    let date = NaiveDateTime::from_timestamp(date, 0);
    use rocket::futures::join;
    let mut conn = state.db_pool.get().await.unwrap();
    let mut conn_two = state.db_pool.get().await.unwrap();
    let pending_order = PendingOrder::get(conn.as_mut(), order_id);

    let user = auth.user;

    let permission = user.get_permissions(conn_two.as_mut());
    let (pending_order, permission) = join!(pending_order, permission);

    if permission.edit_received {
        Json(
            pending_order
                .mark_as_received(conn.as_mut(), date, actually_received, damaged)
                .await
                .id,
        )
    } else {
        panic!("Wrong perms");
    }
}

#[get("/mark_order_as_pending?<order_id>")]
async fn mark_order_as_pending(auth: AuthGuard, state: &State<ServerState>, order_id: i32) {
    use models::PendingOrderBuilder;
    use rocket::futures::join;

    let mut conn = state.db_pool.get().await.unwrap();
    let mut conn_two = state.db_pool.get().await.unwrap();

    let received_order = ReceivedOrder::get(conn.as_mut(), order_id);

    let user = auth.user;

    let permission = user.get_permissions(conn_two.as_mut());
    let (received_order, permission) = join!(received_order, permission);

    if permission.edit_received {
        PendingOrderBuilder::new(received_order.product_id, received_order.gross_amount)
            .build(conn_two.as_mut())
            .await;
        ReceivedOrder::delete(conn.as_mut(), received_order.id).await;
    }
}

#[get("/update_pending_order?<order_info>")]
async fn update_pending_order(auth: AuthGuard, state: &State<ServerState>, order_info: String) {
    let order: PendingOrder = serde_json::from_str(&order_info).unwrap();
    let mut conn = state.db_pool.get().await.unwrap();
    let permission = auth.user.get_permissions(conn.as_mut());

    if permission.await.edit_received {
        order.update(conn.as_mut()).await;
    }
}

#[get("/update_received_order?<order_info>")]
async fn update_received_order(auth: AuthGuard, state: &State<ServerState>, order_info: String) {
    let order: ReceivedOrder = serde_json::from_str(&order_info).unwrap();
    let mut conn = state.db_pool.get().await.unwrap();
    let permission = auth.user.get_permissions(conn.as_mut());

    if permission.await.edit_received {
        order.update(conn.as_mut()).await;
    }
}

#[get("/products?<limit>&<offset>")]
async fn products(
    auth: AuthGuard,
    state: &State<ServerState>,
    limit: i64,
    offset: i64,
) -> Option<Json<Vec<Product>>> {
    let mut conn = state.db_pool.get().await.unwrap();
    use crate::schema::products::dsl::*;

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.view_products {
        Some(Json(
            products
                .limit(limit as i64)
                .offset(offset)
                .load(conn.as_mut())
                .await
                .unwrap(),
        ))
    } else {
        None
    }
}

#[get("/brands?<limit>&<offset>")]
async fn brands(
    auth: AuthGuard,
    state: &State<ServerState>,
    limit: i64,
    offset: i64,
) -> Option<Json<Vec<Brand>>> {
    let mut conn = state.db_pool.get().await.unwrap();
    use crate::schema::brands::dsl::*;
    let user = auth.user;
    let permission = user.get_permissions(conn.as_mut()).await;
    Some(Json(
        brands
            .limit(limit as i64)
            .offset(offset)
            .load(conn.as_mut())
            .await
            .unwrap(),
    ))
}

#[get("/new_category?<name>")]
async fn new_category(auth: AuthGuard, state: &State<ServerState>, name: String) -> Json<i32> {
    let mut conn = state.db_pool.get().await.unwrap();

    let builder = CategoryBuilder::new(name);

    return Json(builder.build(conn.as_mut()).await);
}

#[get("/new_supplier?<name>&<phone_number>&<email>")]
async fn new_supplier(
    auth: AuthGuard,
    state: &State<ServerState>,
    name: String,
    phone_number: Option<String>,
    email: Option<String>,
) -> Json<i32> {
    let mut conn = state.db_pool.get().await.unwrap();

    let mut builder = SupplierBuilder::new(name);

    if let Some(phone_number) = phone_number {
        builder = builder.with_phone_number(phone_number);
    }

    if let Some(email) = email {
        builder = builder.with_email(email);
    }

    return Json(builder.build(conn.as_mut()).await);
}

#[get("/new_brand?<name>")]
async fn new_brand(auth: AuthGuard, state: &State<ServerState>, name: String) -> Json<i32> {
    let mut conn = state.db_pool.get().await.unwrap();

    let builder = BrandBuilder::new(name);

    return Json(builder.build(conn.as_mut()).await);
}

#[get("/new_product?<upc>&<name>&<description>&<measure_by_weight>&<cost_price_per_unit>&<selling_price_per_unit>&<categories>&<suppliers>&<brand>&<buy_level>")]
async fn new_product(
    auth: AuthGuard,
    state: &State<ServerState>,
    upc: String,
    name: String,
    description: String,
    measure_by_weight: bool,
    cost_price_per_unit: String,
    selling_price_per_unit: String,
    categories: Option<Vec<i32>>,
    suppliers: Option<Vec<i32>>,
    brand: Option<i32>,
    buy_level: Option<f64>,
) -> Json<i32> {
    let mut conn = state.db_pool.get().await.unwrap();

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.edit_products {
        let mut builder = ProductBuilder::new(
            &upc,
            &name,
            measure_by_weight,
            BigDecimal::from_str(&cost_price_per_unit).unwrap(),
            BigDecimal::from_str(&selling_price_per_unit).unwrap(),
        );
        if let Some(brand) = brand {
            builder = builder.with_brand(brand);
        }
        if let Some(categories) = categories {
            builder = builder.with_categories(&categories);
        }

        if let Some(suppliers) = suppliers {
            builder = builder.with_suppliers(&suppliers);
        }

        if let Some(buy_level) = buy_level {
            builder = builder.with_buy_level(buy_level)
        }

        return Json(
            builder
                .with_description(&description)
                .build(conn.as_mut())
                .await,
        );
    }
    panic!("Wrong permissions");
}

#[get("/product_categories/<product_id>")]
async fn product_categories(
    auth: AuthGuard,
    state: &State<ServerState>,
    product_id: i32,
) -> Option<Json<Vec<Category>>> {
    let mut conn = state.db_pool.get().await.unwrap();

    Some(Json(
        Product::get(conn.as_mut(), product_id)
            .await
            .get_categories(conn.as_mut())
            .await,
    ))
}

#[get("/product_suppliers/<product_id>")]
async fn product_suppliers(
    auth: AuthGuard,
    state: &State<ServerState>,
    product_id: i32,
) -> Option<Json<Vec<Supplier>>> {
    let mut conn = state.db_pool.get().await.unwrap();

    Some(Json(
        Product::get(conn.as_mut(), product_id)
            .await
            .get_suppliers(conn.as_mut())
            .await,
    ))
}

#[get("/product_brand/<product_id>")]
async fn product_brand(
    auth: AuthGuard,
    state: &State<ServerState>,
    product_id: i32,
) -> Option<Json<Option<Brand>>> {
    let mut conn = state.db_pool.get().await.unwrap();

    Some(Json(
        Product::get(conn.as_mut(), product_id)
            .await
            .get_brand(conn.as_mut())
            .await,
    ))
}

#[get("/categories?<limit>&<offset>")]
async fn categories(
    auth: AuthGuard,
    state: &State<ServerState>,
    limit: i64,
    offset: i64,
) -> Option<Json<Vec<Category>>> {
    let mut conn = state.db_pool.get().await.unwrap();
    use crate::schema::categories::dsl::*;

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.view_products {
        Some(Json(
            categories
                .limit(limit)
                .offset(offset)
                .load(conn.as_mut())
                .await
                .unwrap(),
        ))
    } else {
        None
    }
}

#[get("/suppliers/names")]
async fn supplier_names(
    auth: AuthGuard,
    state: &State<ServerState>,
) -> Option<Json<Vec<(String, i32)>>> {
    let mut conn = state.db_pool.get().await.unwrap();
    use crate::schema::suppliers::dsl::*;

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.view_suppliers {
        Some(Json(Supplier::get_names(conn.as_mut()).await))
    } else {
        None
    }
}

#[get("/brands/names")]
async fn brand_names(
    auth: AuthGuard,
    state: &State<ServerState>,
) -> Option<Json<Vec<(String, i32)>>> {
    let mut conn = state.db_pool.get().await.unwrap();
    use crate::schema::brands::dsl::*;

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.view_products {
        Some(Json(Brand::get_names(conn.as_mut()).await))
    } else {
        None
    }
}

#[get("/products/names")]
async fn product_names(
    auth: AuthGuard,
    state: &State<ServerState>,
) -> Option<Json<Vec<(String, String, i32)>>> {
    let mut conn = state.db_pool.get().await.unwrap();
    use crate::schema::products::dsl::*;

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.view_products {
        Some(Json(Product::get_names(conn.as_mut()).await))
    } else {
        None
    }
}

#[get("/categories/names")]
async fn category_names(
    auth: AuthGuard,
    state: &State<ServerState>,
) -> Option<Json<Vec<(String, i32)>>> {
    let mut conn = state.db_pool.get().await.unwrap();
    use crate::schema::categories::dsl::*;

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.view_products {
        Some(Json(Category::get_names(conn.as_mut()).await))
    } else {
        None
    }
}


#[get("/suppliers?<limit>&<offset>")]
async fn suppliers(
    auth: AuthGuard,
    state: &State<ServerState>,
    limit: i64,
    offset: i64,
) -> Option<Json<Vec<Supplier>>> {
    let mut conn = state.db_pool.get().await.unwrap();
    use crate::schema::suppliers::dsl::*;

    let user = auth.user;

    let permission = user.get_permissions(conn.as_mut()).await;

    if permission.view_suppliers {
        Some(Json(
            suppliers
                .limit(limit)
                .offset(offset)
                .load(conn.as_mut())
                .await
                .unwrap(),
        ))
    } else {
        None
    }
}

#[get("/initialize/<username>/<userpassword>")]
async fn initialize(
    state: &State<ServerState>,
    username: String,
    userpassword: String,
) -> Option<()> {
    let mut conn = state.db_pool.get().await.unwrap();
    use self::models::Permission;
    use self::schema::users::dsl::*;
    use crate::models::Preference;
    use crate::models::User;
    use crate::schema::permissions::dsl::*;

    let n_users: i64 = users.count().get_result(conn.as_mut()).await.unwrap();

    if n_users == 0 {
        use self::schema::preferences::dsl::*;

        let row = User {
            id: 0,
            email: String::from(""),
            name: username,
            password: hash(userpassword + &env::var("PEPPER").expect("PEPPER must be set"), DEFAULT_COST as u32).unwrap(),
        };

        diesel::insert_into(users)
            .values(row)
            .execute(conn.as_mut())
            .await
            .unwrap();

        let perms = Permission {
            user_id: 0,
            admin: true,
            view_pending: true,
            view_received: true,
            edit_pending: true,
            create_orders: true,
            edit_received: true,
            remove_orders: true,
            edit_products: true,
            view_products: true,
            view_suppliers: true,
        };

        diesel::insert_into(permissions)
            .values(perms)
            .execute(conn.as_mut())
            .await
            .unwrap();

        diesel::insert_into(preferences)
            .values(Preference { user_id: 0 })
            .execute(conn.as_mut())
            .await
            .unwrap();

        Some(())
    } else {
        None
    }
}

#[get("/signup/<username>/<userpassword>")]
async fn signup(
    auth: AuthGuard,
    state: &State<ServerState>,
    username: String,
    userpassword: String,
) -> Option<()> {
    let mut conn = state.db_pool.get().await.unwrap();
    use self::models::Permission;
    use self::schema::users::dsl::*;
    use crate::models::Preference;
    use crate::models::User;
    use crate::schema::permissions::dsl::*;

    let permission = permissions
        .filter(user_id.eq(auth.user.id))
        .first::<Permission>(conn.as_mut())
        .await
        .unwrap();

    if permission.admin {
        use self::schema::preferences::dsl::*;
        let maximum_user = users
            .select(crate::schema::users::dsl::id)
            .load::<i32>(conn.as_mut())
            .await
            .unwrap()
            .into_iter()
            .max()
            .unwrap_or(0);

        let row = User {
            id: maximum_user + 1,
            email: String::from(""),
            name: username,
            password: hash(userpassword + &env::var("PEPPER").expect("PEPPER must be set"), DEFAULT_COST as u32).unwrap(),
        };

        diesel::insert_into(users)
            .values(row)
            .execute(conn.as_mut())
            .await
            .unwrap();

        let perms = Permission {
            user_id: maximum_user + 1,
            admin: false,
            view_pending: false,
            view_received: false,
            edit_pending: false,
            create_orders: false,
            edit_received: false,
            remove_orders: false,
            edit_products: false,
            view_products: false,
            view_suppliers: false,
        };

        diesel::insert_into(permissions)
            .values(perms)
            .execute(conn.as_mut())
            .await
            .unwrap();

        diesel::insert_into(preferences)
            .values(Preference {
                user_id: maximum_user + 1,
            })
            .execute(conn.as_mut())
            .await
            .unwrap();

        Some(())
    } else {
        None
    }
}

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let mut conn = database::establish_connection().await;
    database::run_migrations(&mut conn).unwrap();

    let db_pool = Pool::builder(Manager {}).build().unwrap();

    let _rocket = rocket::build()
        .manage(ServerState { db_pool })
        .mount(
            "/",
            routes![
                index,
                products,
                user,
                signup,
                received_orders,
                pending_orders,
                suppliers,
                categories,
                new_product,
                initialize,
                update_user,
                update_category,
                update_brand,
                update_supplier,
                remove_user,
                remove_category,
                remove_brand,
                remove_supplier,
                update_received_order,
                update_pending_order,
                new_pending_order,
                mark_order_as_received,
                mark_order_as_pending,
                permissions,
                add_product_brand,
                add_product_category,
                add_product_supplier,
                update_product,
                remove_product,
                product_brand,
                product_suppliers,
                product_categories,
                new_category,
                new_supplier,
                user_permissions,
                new_brand,
                remove_pending_order,
                remove_received_order,
                brands,
                brand_names,
                supplier_names,
                product_names,
                category_names
            ],
        )
        .launch()
        .await?;

    Ok(())
}
