use crate::schema::*;
use bcrypt::hash;
use bcrypt::verify;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::associations::*;
use diesel::prelude::*;
use diesel::Insertable;
use diesel::QueryDsl;
use diesel::Queryable;
use diesel_async::AsyncPgConnection;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

const PEPPER: &str = "TUbqRXu96kfVDf";
const DEFAULT_COST: usize = 10;

#[derive(Queryable, Insertable, PartialEq, Eq, Debug, Identifiable, Deserialize, Serialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Queryable, PartialEq, Debug, Insertable, Identifiable, Deserialize, Serialize)]
pub struct Product {
    pub id: i32,
    pub upc: String,
    pub name: String,
    pub description: String,
    pub amount: f64,
    pub case_size: Option<i32>,
    pub measure_by_weight: bool,
    pub cost_price_per_unit: BigDecimal,
    pub selling_price_per_unit: BigDecimal,
    pub sale_end: Option<NaiveDateTime>,
    pub buy_level: Option<f64>,
    pub sale_price: Option<BigDecimal>,
}

#[derive(Queryable, PartialEq, Eq, Debug, Insertable, Associations, Deserialize, Serialize)]
#[diesel(belongs_to(User))]
#[diesel(table_name = preferences)]
pub struct Preference {
    pub user_id: i32,
}

#[derive(Queryable, PartialEq, Eq, Debug, Insertable, Associations, Deserialize, Serialize)]
#[diesel(belongs_to(User))]
#[diesel(table_name = permissions)]
pub struct Permission {
    pub user_id: i32,
    pub admin: bool,
    pub view_pending: bool,
    pub view_received: bool,
    pub edit_pending: bool,
    pub create_orders: bool,
    pub edit_received: bool,
    pub remove_orders: bool,
    pub edit_products: bool,
    pub view_products: bool,
    pub view_suppliers: bool,
}

#[derive(Queryable, PartialEq, Eq, Debug, Insertable, Deserialize, Serialize)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: i32,
    pub products: Vec<Option<i32>>,
    pub name: String,
}

#[derive(Queryable, PartialEq, Eq, Debug, Insertable, Deserialize, Serialize)]
pub struct Supplier {
    pub id: i32,
    pub products: Vec<Option<i32>>,
    pub name: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
}

#[derive(Queryable, PartialEq, Eq, Debug, Insertable, Deserialize, Serialize)]
pub struct Brand {
    pub id: i32,
    pub name: String,
    pub products: Vec<Option<i32>>,
}

#[derive(Queryable, PartialEq, Debug, Insertable, Associations, Deserialize, Serialize, Clone)]
#[diesel(belongs_to(Product))]
pub struct ReceivedOrder {
    pub id: i32,
    pub received: Option<NaiveDateTime>,
    pub product_id: i32,
    pub gross_amount: f64,
    pub actually_received: f64,
    pub damaged: f64,
}

#[derive(Queryable, PartialEq, Debug, Insertable, Associations, Deserialize, Serialize)]
#[diesel(belongs_to(Product))]
pub struct PendingOrder {
    pub id: i32,
    pub product_id: i32,
    pub amount: f64,
}

pub struct UserBuilder {
    pub name: String,
    pub email: String,
    pub password: String,
    pub permissions: PermissionsBuilder,
    pub preferences: PreferencesBuilder,
}

impl UserBuilder {
    pub fn new(name: String, password: &str) -> Self {
        UserBuilder {
            name,
            email: String::new(),
            password: hash(password.to_string() + PEPPER, DEFAULT_COST as u32).unwrap(),
            permissions: PermissionsBuilder::default(),
            preferences: PreferencesBuilder::default(),
        }
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.email = email.to_owned();
        self
    }

    pub fn with_permissions(mut self, perms: PermissionsBuilder) -> Self {
        self.permissions = perms;
        self
    }
    pub fn with_preferences(mut self, preferences: PreferencesBuilder) -> Self {
        self.preferences = preferences;
        self
    }

    pub async fn build(self, conn: &mut AsyncPgConnection) -> i32 {
        let user_id = crate::schema::users::dsl::users
            .select(crate::schema::users::dsl::id)
            .load::<i32>(conn)
            .await
            .unwrap()
            .into_iter()
            .max()
            .unwrap_or(0)
            + 1;
        let row = User {
            id: user_id,
            name: self.name,
            password: self.password,
            email: self.email,
        };
        diesel::insert_into(crate::schema::users::dsl::users)
            .values(row)
            .execute(conn)
            .await
            .unwrap();

        let row = Preference { user_id };
        diesel::insert_into(crate::schema::preferences::dsl::preferences)
            .values(row)
            .execute(conn)
            .await
            .unwrap();

        let row = Permission {
            user_id,
            admin: self.permissions.admin,
            create_orders: self.permissions.create_orders,
            edit_pending: self.permissions.edit_pending,
            edit_products: self.permissions.edit_products,
            view_pending: self.permissions.view_pending,
            edit_received: self.permissions.edit_received,
            view_received: self.permissions.view_received,
            remove_orders: self.permissions.remove_orders,
            view_products: self.permissions.view_products,
            view_suppliers: self.permissions.view_suppliers,
        };
        diesel::insert_into(crate::schema::permissions::dsl::permissions)
            .values(row)
            .execute(conn)
            .await
            .unwrap();
        user_id
    }
}

#[derive(Default)]
pub struct PermissionsBuilder {
    pub admin: bool,
    pub view_pending: bool,
    pub view_received: bool,
    pub edit_pending: bool,
    pub create_orders: bool,
    pub edit_received: bool,
    pub remove_orders: bool,
    pub edit_products: bool,
    pub view_products: bool,
    pub view_suppliers: bool,
}

#[derive(Default)]
pub struct PreferencesBuilder {}
impl User {
    pub async fn from_name(conn: &mut AsyncPgConnection, username: &str) -> Self {
        crate::schema::users::dsl::users
            .filter(crate::schema::users::dsl::name.eq(username))
            .first(conn)
            .await
            .unwrap()
    }

    pub async fn get(
        conn: &mut AsyncPgConnection,
        username: &str,
        userpassword: &str,
    ) -> Option<Self> {
        let user: User = crate::schema::users::dsl::users
            .filter(crate::schema::users::dsl::name.eq(username))
            .first(conn)
            .await
            .unwrap();
        match verify(userpassword.to_string() + PEPPER, &user.password).unwrap() {
            true => Some(user),
            false => None,
        }
    }

    pub async fn from_id(conn: &mut AsyncPgConnection, id: i32) -> Self {
        crate::schema::users::dsl::users
            .filter(crate::schema::users::dsl::id.eq(id))
            .first(conn)
            .await
            .unwrap()
    }

    pub async fn update(self, conn: &mut AsyncPgConnection) {
        diesel::update(
            crate::schema::users::dsl::users.filter(crate::schema::users::dsl::id.eq(self.id)),
        )
        .set((
            crate::schema::users::dsl::name.eq(self.name),
            crate::schema::users::dsl::email.eq(self.email),
            crate::schema::users::dsl::password.eq(self.password),
        ))
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn delete(conn: &mut AsyncPgConnection, id: i32) {
        diesel::delete(
            crate::schema::users::dsl::users.filter(crate::schema::users::dsl::id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
        diesel::delete(
            crate::schema::permissions::dsl::permissions
                .filter(crate::schema::permissions::dsl::user_id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
        diesel::delete(
            crate::schema::preferences::dsl::preferences
                .filter(crate::schema::preferences::dsl::user_id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn get_permissions(&self, conn: &mut AsyncPgConnection) -> Permission {
        crate::schema::permissions::dsl::permissions
            .filter(crate::schema::permissions::dsl::user_id.eq(self.id))
            .first(conn)
            .await
            .unwrap()
    }

    pub async fn get_preferences(&self, conn: &mut AsyncPgConnection) -> Preference {
        crate::schema::preferences::dsl::preferences
            .filter(crate::schema::preferences::dsl::user_id.eq(self.id))
            .first(conn)
            .await
            .unwrap()
    }
}

#[derive(Default)]
pub struct ProductBuilder {
    pub upc: String,
    pub name: String,
    pub description: Option<String>,
    pub measure_by_weight: bool,
    pub cost_price_per_unit: BigDecimal,
    pub selling_price_per_unit: BigDecimal,
    pub case_size: Option<i32>,
    pub categories: Vec<i32>,
    pub suppliers: Vec<i32>,
    pub brand: Option<i32>,
    pub buy_level: Option<f64>,
}

impl ProductBuilder {
    pub fn new(
        upc: &str,
        name: &str,
        measure_by_weight: bool,
        cost_price_per_unit: BigDecimal,
        selling_price_per_unit: BigDecimal,
    ) -> Self {
        Self {
            upc: upc.to_string(),
            name: name.to_string(),
            description: None,
            measure_by_weight,
            cost_price_per_unit,
            selling_price_per_unit,
            categories: Vec::new(),
            case_size: None,
            suppliers: Vec::new(),
            brand: None,
            buy_level: None,
        }
    }

    pub fn with_categories(mut self, categories: &[i32]) -> Self {
        self.categories = categories.to_vec();
        self
    }

    pub fn with_suppliers(mut self, suppliers: &[i32]) -> Self {
        self.suppliers = suppliers.to_vec();
        self
    }

    pub fn with_brand(mut self, brand: i32) -> Self {
        self.brand = Some(brand);
        self
    }

    pub fn with_buy_level(mut self, buy_level: f64) -> Self {
        self.buy_level = Some(buy_level);
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn with_case_size(mut self, case_size: i32) -> Self {
        self.case_size = Some(case_size);
        self
    }

    pub async fn build(self, conn: &mut AsyncPgConnection) -> i32 {
        let product_id = crate::schema::products::dsl::products
            .select(crate::schema::products::dsl::id)
            .load::<i32>(conn)
            .await
            .unwrap()
            .into_iter()
            .max()
            .unwrap_or(0)
            + 1;
        let row = Product {
            id: product_id,
            upc: self.upc,
            name: self.name,
            amount: 0.0,
            case_size: self.case_size,
            description: self.description.unwrap_or_default(),
            cost_price_per_unit: self.cost_price_per_unit,
            selling_price_per_unit: self.selling_price_per_unit,
            measure_by_weight: self.measure_by_weight,
            sale_end: None,
            sale_price: None,
            buy_level: self.buy_level,
        };
        diesel::insert_into(crate::schema::products::dsl::products)
            .values(row)
            .execute(conn)
            .await
            .unwrap();

        if let Some(brand) = self.brand {
            let mut products: Vec<Option<i32>> = crate::schema::brands::dsl::brands
                .filter(crate::schema::brands::dsl::id.eq(brand))
                .select(crate::schema::brands::dsl::products)
                .first(conn)
                .await
                .unwrap();
            products.push(Some(product_id));
            diesel::update(
                crate::schema::brands::dsl::brands.filter(crate::schema::brands::dsl::id.eq(brand)),
            )
            .set(crate::schema::brands::dsl::products.eq(products))
            .execute(conn)
            .await
            .unwrap();
        };

        for category in self.categories.into_iter() {
            let mut products: Vec<Option<i32>> = crate::schema::categories::dsl::categories
                .filter(crate::schema::categories::dsl::id.eq(category))
                .select(crate::schema::categories::dsl::products)
                .first(conn)
                .await
                .unwrap();
            products.push(Some(product_id));
            diesel::update(
                crate::schema::categories::dsl::categories
                    .filter(crate::schema::categories::dsl::id.eq(category)),
            )
            .set(crate::schema::categories::dsl::products.eq(products))
            .execute(conn)
            .await
            .unwrap();
        }

        for supplier in self.suppliers.into_iter() {
            let mut products: Vec<Option<i32>> = crate::schema::suppliers::dsl::suppliers
                .filter(crate::schema::suppliers::dsl::id.eq(supplier))
                .select(crate::schema::suppliers::dsl::products)
                .first(conn)
                .await
                .unwrap();
            products.push(Some(product_id));
            diesel::update(
                crate::schema::suppliers::dsl::suppliers
                    .filter(crate::schema::suppliers::dsl::id.eq(supplier)),
            )
            .set(crate::schema::suppliers::dsl::products.eq(products))
            .execute(conn)
            .await
            .unwrap();
        }
        product_id
    }
}

impl Product {
    pub async fn get(conn: &mut AsyncPgConnection, id: i32) -> Self {
        crate::schema::products::dsl::products
            .filter(crate::schema::products::dsl::id.eq(id))
            .first(conn)
            .await
            .unwrap()
    }

    pub async fn update(self, conn: &mut AsyncPgConnection) {
        diesel::update(
            crate::schema::products::dsl::products
                .filter(crate::schema::products::dsl::id.eq(self.id)),
        )
        .set((
            crate::schema::products::dsl::upc.eq(self.upc),
            crate::schema::products::dsl::name.eq(self.name),
            crate::schema::products::dsl::description.eq(self.description),
            crate::schema::products::dsl::amount.eq(self.amount),
            crate::schema::products::dsl::buy_level.eq(self.buy_level),
            crate::schema::products::dsl::case_size.eq(self.case_size),
            crate::schema::products::dsl::measure_by_weight.eq(self.measure_by_weight),
            crate::schema::products::dsl::cost_price_per_unit.eq(self.cost_price_per_unit),
            crate::schema::products::dsl::selling_price_per_unit.eq(self.selling_price_per_unit),
            crate::schema::products::dsl::sale_end.eq(self.sale_end),
            crate::schema::products::dsl::sale_price.eq(self.sale_price),
        ))
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn add_supplier(&self, conn: &mut AsyncPgConnection, id: i32) {
        let mut supplier = Supplier::get(conn, id).await;
        supplier.products.push(Some(self.id));
        supplier.update(conn).await;
    }

    pub async fn add_category(&self, conn: &mut AsyncPgConnection, id: i32) {
        let mut category = Category::get(conn, id).await;
        category.products.push(Some(self.id));
        category.update(conn).await;
    }

    pub async fn add_brand(&self, conn: &mut AsyncPgConnection, id: i32) {
        let mut brand = Brand::get(conn, id).await;
        brand.products.push(Some(self.id));
        brand.update(conn).await;
    }

    pub async fn remove_supplier(self, conn: &mut AsyncPgConnection, id: i32) {
        let mut supplier = Supplier::get(conn, id).await;
        supplier
            .products
            .retain(|product| product.unwrap() != self.id);
        supplier.update(conn).await;
    }

    pub async fn remove_category(self, conn: &mut AsyncPgConnection, id: i32) {
        let mut category = Category::get(conn, id).await;
        category
            .products
            .retain(|product| product.unwrap() != self.id);
        category.update(conn).await;
    }

    pub async fn remove_brand(self, conn: &mut AsyncPgConnection, id: i32) {
        let mut brand = Brand::get(conn, id).await;
        brand.products.retain(|brand| brand.unwrap() != self.id);
        brand.update(conn).await;
    }

    pub async fn delete(self, conn: &mut AsyncPgConnection) {
        let id = self.id;
        diesel::delete(
            crate::schema::pending_orders::dsl::pending_orders
                .filter(crate::schema::pending_orders::dsl::product_id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
        diesel::delete(
            crate::schema::received_orders::dsl::received_orders
                .filter(crate::schema::received_orders::dsl::product_id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
        let suppliers: Vec<Supplier> = crate::schema::suppliers::dsl::suppliers
            .filter(crate::schema::suppliers::dsl::products.contains(vec![id]))
            .load(conn)
            .await
            .unwrap();
        for supplier in suppliers {
            let mut products: Vec<Option<i32>> = crate::schema::suppliers::dsl::suppliers
                .filter(crate::schema::suppliers::dsl::id.eq(supplier.id))
                .select(crate::schema::suppliers::dsl::products)
                .first(conn)
                .await
                .unwrap();
            products.retain(|&product| product.unwrap() != id);
            diesel::update(
                crate::schema::suppliers::dsl::suppliers
                    .filter(crate::schema::suppliers::dsl::id.eq(supplier.id)),
            )
            .set(crate::schema::suppliers::dsl::products.eq(products)
                )
            .execute(conn)
            .await
            .unwrap();
        }

        let brand: Brand = match crate::schema::brands::dsl::brands
            .filter(crate::schema::brands::dsl::products.contains(vec![id]))
            .first(conn)
            .await
        {
            Ok(val) => val,
            Err(_) => {
                diesel::delete(
                    crate::schema::products::dsl::products
                        .filter(crate::schema::products::dsl::id.eq(id)),
                )
                .execute(conn)
                .await
                .unwrap();
                return;
            }
        };
        {
            let mut products: Vec<Option<i32>> = crate::schema::brands::dsl::brands
                .filter(crate::schema::brands::dsl::id.eq(brand.id))
                .select(crate::schema::brands::dsl::products)
                .first(conn)
                .await
                .unwrap();
            products.retain(|&product| product.unwrap() != id);
            diesel::update(
                crate::schema::brands::dsl::brands
                    .filter(crate::schema::brands::dsl::id.eq(brand.id)),
            )
            .set(crate::schema::brands::dsl::products.eq(products))
            .execute(conn)
            .await
            .unwrap()
        };
        diesel::delete(
            crate::schema::products::dsl::products.filter(crate::schema::products::dsl::id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn get_all(conn: &mut AsyncPgConnection, limit: i64, offset: i64) -> Vec<Self> {
        crate::schema::products::dsl::products
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .unwrap()
    }

    pub async fn get_names(conn: &mut AsyncPgConnection) -> Vec<(String, String, i32)> {
        crate::schema::products::dsl::products.load(conn).await.unwrap().into_iter().map(|product: Product| (product.name, product.upc, product.id)).collect()
    }

    pub async fn get_categories(&self, conn: &mut AsyncPgConnection) -> Vec<Category> {
        crate::schema::categories::dsl::categories
            .filter(crate::schema::categories::dsl::products.contains(vec![self.id]))
            .load(conn)
            .await
            .unwrap()
    }

    pub async fn get_suppliers(&self, conn: &mut AsyncPgConnection) -> Vec<Supplier> {
        crate::schema::suppliers::dsl::suppliers
            .filter(crate::schema::suppliers::dsl::products.contains(vec![self.id]))
            .load(conn)
            .await
            .unwrap()
    }

    pub async fn get_brand(&self, conn: &mut AsyncPgConnection) -> Option<Brand> {
        match crate::schema::brands::dsl::brands
            .filter(crate::schema::brands::dsl::products.contains(vec![self.id]))
            .first(conn)
            .await
        {
            Ok(val) => Some(val),
            Err(_) => None,
        }
    }
}

#[derive(Default)]
pub struct CategoryBuilder {
    pub name: String,
}

impl CategoryBuilder {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub async fn build(self, conn: &mut AsyncPgConnection) -> i32 {
        let category_id = crate::schema::categories::dsl::categories
            .select(crate::schema::categories::dsl::id)
            .load::<i32>(conn)
            .await
            .unwrap()
            .into_iter()
            .max()
            .unwrap_or(0)
            + 1;
        let row = Category {
            id: category_id,
            name: self.name,
            products: Vec::new(),
        };
        diesel::insert_into(crate::schema::categories::dsl::categories)
            .values(row)
            .execute(conn)
            .await
            .unwrap();
        category_id
    }
}

impl Category {
    pub async fn get(conn: &mut AsyncPgConnection, id: i32) -> Self {
        crate::schema::categories::dsl::categories
            .filter(crate::schema::categories::dsl::id.eq(id))
            .first(conn)
            .await
            .unwrap()
    }

    pub async fn update(self, conn: &mut AsyncPgConnection) {
        diesel::update(
            crate::schema::categories::dsl::categories
                .filter(crate::schema::categories::dsl::id.eq(self.id)),
        )
        .set((
            crate::schema::categories::dsl::name.eq(self.name),
            crate::schema::categories::dsl::products.eq(self.products),
        ))
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn delete(conn: &mut AsyncPgConnection, id: i32) {
        diesel::delete(
            crate::schema::categories::dsl::categories
                .filter(crate::schema::categories::dsl::id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn get_all(conn: &mut AsyncPgConnection) -> Vec<Self> {
        crate::schema::categories::dsl::categories
            .load(conn)
            .await
            .unwrap()
    }

    pub async fn get_names(conn: &mut AsyncPgConnection) -> Vec<(String, i32)> {
        crate::schema::categories::dsl::categories.load(conn).await.unwrap().into_iter().map(|category: Category| (category.name, category.id)).collect()
    }

}

#[derive(Default)]
pub struct SupplierBuilder {
    pub name: String,
    pub phone_number: Option<String>,
    pub email: Option<String>,
}

impl SupplierBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            phone_number: None,
            email: None,
        }
    }

    pub fn with_phone_number(mut self, number: String) -> Self {
        self.phone_number = Some(number);
        self
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub async fn build(self, conn: &mut AsyncPgConnection) -> i32 {
        let supplier_id = crate::schema::suppliers::dsl::suppliers
            .select(crate::schema::suppliers::dsl::id)
            .load::<i32>(conn)
            .await
            .unwrap()
            .into_iter()
            .max()
            .unwrap_or(0)
            + 1;
        let row = Supplier {
            id: supplier_id,
            name: self.name,
            phone_number: self.phone_number,
            products: Vec::new(),
            email: self.email,
        };
        diesel::insert_into(crate::schema::suppliers::dsl::suppliers)
            .values(row)
            .execute(conn)
            .await
            .unwrap();
        supplier_id
    }
}

impl Supplier {
    pub async fn get(conn: &mut AsyncPgConnection, id: i32) -> Self {
        crate::schema::suppliers::dsl::suppliers
            .filter(crate::schema::suppliers::dsl::id.eq(id))
            .first(conn)
            .await
            .unwrap()
    }

    pub async fn update(self, conn: &mut AsyncPgConnection) {
        diesel::update(
            crate::schema::suppliers::dsl::suppliers
                .filter(crate::schema::suppliers::dsl::id.eq(self.id)),
        )
        .set((
            crate::schema::suppliers::dsl::name.eq(self.name),
            crate::schema::suppliers::dsl::email.eq(self.email),
            crate::schema::suppliers::dsl::phone_number.eq(self.phone_number),
            crate::schema::suppliers::dsl::products.eq(self.products),
        ))
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn delete(conn: &mut AsyncPgConnection, id: i32) {
        diesel::delete(
            crate::schema::suppliers::dsl::suppliers
                .filter(crate::schema::suppliers::dsl::id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn get_all(conn: &mut AsyncPgConnection) -> Vec<Self> {
        crate::schema::suppliers::dsl::suppliers
            .load(conn)
            .await
            .unwrap()
    }
    pub async fn get_names(conn: &mut AsyncPgConnection) -> Vec<(String, i32)> {
        crate::schema::suppliers::dsl::suppliers.load(conn).await.unwrap().into_iter().map(|supplier: Supplier| (supplier.name, supplier.id)).collect()
    }

}

#[derive(Default)]
pub struct BrandBuilder {
    pub name: String,
}

impl BrandBuilder {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub async fn build(self, conn: &mut AsyncPgConnection) -> i32 {
        let brand_id = crate::schema::brands::dsl::brands
            .select(crate::schema::brands::dsl::id)
            .load::<i32>(conn)
            .await
            .unwrap()
            .into_iter()
            .max()
            .unwrap_or(0)
            + 1;
        let row = Brand {
            id: brand_id,
            name: self.name,
            products: Vec::new(),
        };
        diesel::insert_into(crate::schema::brands::dsl::brands)
            .values(row)
            .execute(conn)
            .await
            .unwrap();
        brand_id
    }
}

impl Brand {
    pub async fn get(conn: &mut AsyncPgConnection, id: i32) -> Self {
        crate::schema::brands::dsl::brands
            .filter(crate::schema::brands::dsl::id.eq(id))
            .first(conn)
            .await
            .unwrap()
    }

    pub async fn update(self, conn: &mut AsyncPgConnection) {
        diesel::update(
            crate::schema::brands::dsl::brands.filter(crate::schema::brands::dsl::id.eq(self.id)),
        )
        .set((
            crate::schema::brands::dsl::name.eq(self.name),
            crate::schema::brands::dsl::products.eq(self.products),
        ))
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn delete(conn: &mut AsyncPgConnection, id: i32) {
        diesel::delete(
            crate::schema::brands::dsl::brands.filter(crate::schema::brands::dsl::id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn get_all(conn: &mut AsyncPgConnection) -> Vec<Self> {
        crate::schema::brands::dsl::brands.load(conn).await.unwrap()
    }
    pub async fn get_names(conn: &mut AsyncPgConnection) -> Vec<(String, i32)> {
        crate::schema::brands::dsl::brands.load(conn).await.unwrap().into_iter().map(|brand: Brand| (brand.name, brand.id)).collect()
    }
}

#[derive(Default)]
pub struct PendingOrderBuilder {
    pub product_id: i32,
    pub amount: f64,
}

impl PendingOrderBuilder {
    pub fn new(product_id: i32, amount: f64) -> Self {
        Self { product_id, amount }
    }

    pub async fn build(self, conn: &mut AsyncPgConnection) -> i32 {
        let order_id = crate::schema::pending_orders::dsl::pending_orders
            .select(crate::schema::pending_orders::dsl::id)
            .load::<i32>(conn)
            .await
            .unwrap()
            .into_iter()
            .max()
            .unwrap_or(0)
            + 1;
        let row = PendingOrder {
            id: order_id,
            product_id: self.product_id,
            amount: self.amount,
        };
        diesel::insert_into(crate::schema::pending_orders::dsl::pending_orders)
            .values(row)
            .execute(conn)
            .await
            .unwrap();
        order_id
    }
}

impl PendingOrder {
    pub async fn get(conn: &mut AsyncPgConnection, id: i32) -> Self {
        crate::schema::pending_orders::dsl::pending_orders
            .filter(crate::schema::pending_orders::dsl::id.eq(id))
            .first(conn)
            .await
            .unwrap()
    }

    pub async fn update(self, conn: &mut AsyncPgConnection) {
        diesel::update(
            crate::schema::pending_orders::dsl::pending_orders
                .filter(crate::schema::pending_orders::dsl::id.eq(self.id)),
        )
        .set((
            crate::schema::pending_orders::dsl::product_id.eq(self.product_id),
            crate::schema::pending_orders::dsl::amount.eq(self.amount),
        ))
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn delete(conn: &mut AsyncPgConnection, id: i32) {
        diesel::delete(
            crate::schema::pending_orders::dsl::pending_orders
                .filter(crate::schema::pending_orders::dsl::id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn get_all(conn: &mut AsyncPgConnection, limit: i64, offset: i64) -> Vec<Self> {
        crate::schema::pending_orders::dsl::pending_orders
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .unwrap()
    }

    pub async fn mark_as_received(
        self,
        conn: &mut AsyncPgConnection,
        date: NaiveDateTime,
        actually_received: f64,
        damaged: f64,
    ) -> ReceivedOrder {
        let order_id = crate::schema::received_orders::dsl::received_orders
            .select(crate::schema::received_orders::dsl::id)
            .load::<i32>(conn)
            .await
            .unwrap()
            .into_iter()
            .max()
            .unwrap_or(0)
            + 1;
        let row = ReceivedOrder {
            id: order_id,
            received: Some(date),
            product_id: self.product_id,
            gross_amount: self.amount,
            actually_received,
            damaged,
        };
        diesel::insert_into(crate::schema::received_orders::dsl::received_orders)
            .values(row.clone())
            .execute(conn)
            .await
            .unwrap();
        Self::delete(conn, self.id).await;
        row
    }
}

impl ReceivedOrder {
    pub async fn get(conn: &mut AsyncPgConnection, id: i32) -> Self {
        crate::schema::received_orders::dsl::received_orders
            .filter(crate::schema::received_orders::dsl::id.eq(id))
            .first(conn)
            .await
            .unwrap()
    }

    pub async fn update(self, conn: &mut AsyncPgConnection) {
        diesel::update(
            crate::schema::received_orders::dsl::received_orders
                .filter(crate::schema::received_orders::dsl::id.eq(self.id)),
        )
        .set((
            crate::schema::received_orders::dsl::product_id.eq(self.product_id),
            crate::schema::received_orders::dsl::damaged.eq(self.damaged),
            crate::schema::received_orders::dsl::actually_received.eq(self.actually_received),
            crate::schema::received_orders::dsl::gross_amount.eq(self.gross_amount),
            crate::schema::received_orders::dsl::received.eq(self.received),
        ))
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn delete(conn: &mut AsyncPgConnection, id: i32) {
        diesel::delete(
            crate::schema::received_orders::dsl::received_orders
                .filter(crate::schema::received_orders::dsl::id.eq(id)),
        )
        .execute(conn)
        .await
        .unwrap();
    }

    pub async fn get_all(conn: &mut AsyncPgConnection, limit: i64, offset: i64) -> Vec<Self> {
        crate::schema::received_orders::dsl::received_orders
            .limit(limit)
            .offset(offset)
            .load(conn)
            .await
            .unwrap()
    }
}
