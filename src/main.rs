#[macro_use] extern crate rocket;
use std::rc::Rc;
use rand::prelude::*;
use rocket::data;
use rocket::Request;
use rocket::http::{Status, ContentType};
use rocket::Data;
use rocket::http::StatusClass::Success;

struct Key {
    val: String,
    expiry: u64,
    for_user_uuid: String
}

struct Transaction {
    for_user: String,
    amount: f64
}
struct User {
    username: String,
    transaction: Vec<Rc<Transaction>>
}
struct KeyTransaction {
    for_user: String,
    expires: u64
}

impl FromDataSimple for KeyTransaction {
    type Error = String;

    fn from_data<'a>(req: &'a Request<'a>, data: Data) -> data::Outcome<'a, Self, String> {
        let mut contents = String::new();

        if let Err(e) = data.open().take(256).read_to_string(&mut contents) {
            return Failure((Status::InternalServerError, format!("{:?}", e)));
        }

        let split = contents.split("\n");
        let for_user = contents[0];
        let expires = contents[1].parse::<i32>().unwrap();;
        Success(KeyTransaction { for_user, expires })
    }
}

#[get("/getKey")]
fn get_key() -> String {
    let mut rng = rand::thread_rng();
    let mut ret = String::from("bearer_api_");
    let mut key : Vec<char> = "qwertyuiopasdfghjklzxcvbnm1234567890".chars().collect();
    key.shuffle(&mut rng);
    let collected : String = key.iter().collect();
    ret.push_str(collected.as_str());
    ret
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, get_key])
}