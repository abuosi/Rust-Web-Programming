#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate diesel;
use rocket_contrib::json::Json;
use diesel::prelude::*;

mod jwt;
mod to_do;
mod schema;
mod database;
mod models;
mod json_serialization;
mod not_found;

use crate::database::DbConn;
use crate::models::item::item::Item;
use crate::to_do::to_do_factory;
use crate::json_serialization::to_do_items::ToDoItems;
use crate::json_serialization::to_do_item::ToDoItem;
use crate::not_found::ReRouter;

use crate::models::item::new_item::NewItem;


#[get("/hello/<name>/<age>")]
fn hello(name: String, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/bye/<name>/<age>")]
fn bye(name: String, age: u8) -> String {
    format!("Goodbye, {} year old named {}!", age, name)
}

#[get("/get")]
fn get_items(conn: DbConn, token: jwt::JwtToken) -> Json<ToDoItems> {

    let items = schema::to_do::table
        .order(schema::to_do::columns::id.asc())
        .filter(schema::to_do::columns::user_id.eq(&token.user_id))
        .load::<Item>(&*conn)
        .unwrap();

    let mut array_buffer = Vec::new();

    for item in items {
        let item = to_do_factory(&item.status, item.title).unwrap();
        array_buffer.push(item);
    }
    return Json(ToDoItems::new(array_buffer))
}

#[post("/input", data="<item>", format = "json")]
fn input(item: Json<ToDoItem>) -> Json<ToDoItem> {
    return Json(item.into_inner())
}

#[post("/create", data="<item>", format = "json")]
fn create(item: Json<ToDoItem>, conn: DbConn, token: jwt::JwtToken) -> Json<ToDoItems> {
    let title: String = item.title.clone();
    let title_ref: String = item.title.clone();


    let items = schema::to_do::table
        .filter(schema::to_do::columns::title.eq(title_ref.as_str()))
        .filter(schema::to_do::columns::user_id.eq(&token.user_id))
        .order(schema::to_do::columns::id.asc())
        .load::<Item>(&*conn)
        .unwrap();

    if items.len() == 0 {
        let new_post = NewItem::new(title, token.user_id.clone());
        let _ = diesel::insert_into(schema::to_do::table).values(&new_post)
            .execute(&*conn);
    }

    let items = schema::to_do::table
        .order(schema::to_do::columns::id.asc())
        .filter(schema::to_do::columns::user_id.eq(&token.user_id))
        .load::<Item>(&*conn)
        .unwrap();

    let mut array_buffer = Vec::new();

    for item in items {
        let item = to_do_factory(&item.status, item.title).unwrap();
        array_buffer.push(item);
    }
    return Json(ToDoItems::new(array_buffer))
}


fn main() {
    rocket::ignite()
        .mount("/", routes![hello, bye, input, create])
        .mount("/items", routes![get_items])
        .attach(DbConn::fairing())
        .attach(ReRouter)
        .launch();
}
