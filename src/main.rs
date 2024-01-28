#[macro_use] extern crate rocket;

use std::fmt::Display;
use std::rc::Rc;
use rand::prelude::*;
use rocket::data::{FromData, Outcome, ToByteUnit};
use rocket::{Data, Request, Response};
use std::fs::File;
use std::io::{Read, Write};
use rocket::http::Status;
use rocket::response::status;

struct Key {
    val: String,
    expiry: u64,
    for_user_uuid: String
}

struct Transaction {
    for_user: String,
    name: String,
    time: u64,
    amount: f64
}
struct User {
    username: String,
    transaction: Vec<Rc<Transaction>>,
    password_hash: String
}
struct KeyTransaction {
    for_user: String,
    expires: u64
}
struct SignUp {
    username: String,
    password: String
}
static mut KEYS: Vec<Key> = Vec::new();
static mut USERS: Vec<User> = Vec::new();

fn search_for_user(username: String) -> bool {
    unsafe {
        for user in &USERS {
            if user.username == username {
                return true;
            }
        }
    }
    false
}
fn get_unix() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
    since_the_epoch.as_secs()
}
fn key_valid(k: &String) -> bool {
    unsafe {
        for key in &KEYS {
            if key.val == *k {
                if (key.expiry - get_unix()) > 0 {
                    return true;
                } else {
                    //remove the key
                    KEYS.retain(|x| x.val != *k);
                    return false;
                }
            }
        }
    }
    false
}
fn get_key(k: String) -> Option<&'static Key> {
    unsafe {
        for key in &KEYS {
            if key.val == k {
                return Some(*Rc::new(key));
            }
        }
    }
    None
}

#[async_trait]
impl<'a> FromData<'a> for KeyTransaction {
    type Error = ();

    async fn from_data(_req: &'a Request<'_>, data: Data<'a>) -> Outcome<'a, Self> {
        //read data from request
        let inner = data.open(2048.mebibytes()).into_string().await.unwrap().into_inner();
        let result = inner.trim();
        let mut split = result.split(';');
        let for_user = split.next().unwrap().to_string();
        let hash = split.next().unwrap().to_string();
        let expires = split.next().unwrap().parse::<u64>().unwrap();
        if !search_for_user(for_user.clone()) {
            return Outcome::Error((Status::Unauthorized, ()));
        }
        unsafe {
            let mut valid = false;
            for user in &USERS {
                if user.username == for_user && user.password_hash == hash {
                    valid = true;
                }
            }
            if !valid {
                return Outcome::Error((Status::Unauthorized, ()));
            }
        }
        Outcome::Success(KeyTransaction {
            for_user,
            expires
        })
    }
}
#[async_trait]
impl<'a> FromData<'a> for Transaction {
    type Error = ();

    async fn from_data(req: &'a Request<'_>, data: Data<'a>) -> Outcome<'a, Self> {
        //read header
        let header = req.headers();
        if !header.contains("x-bearer") {
            return Outcome::Error((Status::Unauthorized, ()));
        }
        let key = header.get_one("x-bearer").unwrap().to_string();
        if !key_valid(&key) {
            return Outcome::Error((Status::Unauthorized, ()));
        }
        let user = get_key(key).unwrap().for_user_uuid.clone();
        //read data from request
        let inner = data.open(2048.mebibytes()).into_string().await.unwrap().into_inner();
        let mut split = inner.split(';');
        let name = split.next().unwrap().to_string();
        let result = split.next().unwrap().to_string();

        Outcome::Success(Transaction {
            for_user: user,
            name,
            time: get_unix(),
            amount: result.parse().unwrap_or(0.0)
        })
    }
}
#[async_trait]
impl<'a> FromData<'a> for SignUp {
    type Error = ();

    async fn from_data(_req: &'a Request<'_>, data: Data<'a>) -> Outcome<'a, Self> {
        //read data from request
        let inner = data.open(2048.mebibytes()).into_string().await.unwrap().into_inner();
        let result = inner.trim();
        let mut split = result.split(';');
        let username = split.next().unwrap().to_string();
        let password = split.next().unwrap().to_string();
        if search_for_user(username.clone()) {
            return Outcome::Error((Status::Unauthorized, ()));
        }
        Outcome::Success(SignUp {
            username,
            password
        })
    }
}
impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted = format!("{}:{}:{}", self.time, self.name, self.amount);
        write!(f, "{}", formatted)
    }
}
#[get("/key", data = "<key>")]
fn key(key: KeyTransaction) -> String {
    let mut rng = thread_rng();
    let mut val = String::from("bearer_");
    for _ in 0..32 {
        val.push(rng.gen_range(0..9).to_string().chars().next().unwrap());
    }
    let val_clone = val.clone();
    unsafe {
        KEYS.push(Key {
            val: val_clone,
            expiry: key.expires,
            for_user_uuid: key.for_user
        });
    }
    val
}
#[post("/transact", data="<transact>")]
fn transact(transact: Transaction) -> String {
    unsafe {
        for user in &mut USERS {
            if user.username == transact.for_user {
                user.transaction.push(Rc::new(transact));
                return String::from("OK");
            }
        }
    }
    String::from("Unable to find user")
}
#[get("/balance/<bearer>")]
fn balance(bearer: String) -> status::Custom<String> {
    let mut total = 0.0;
    if !key_valid(&bearer) {
        return status::Custom(Status::Unauthorized, String::from("Unauthorized"));
    }
    let u = get_key(bearer).unwrap().for_user_uuid.clone();
    unsafe {
        for user in &USERS {
            if user.username == u {
                for transaction in &user.transaction {
                    total += transaction.amount;
                }
            }
        }
    }
    status::Custom(Status::Ok, total.to_string())
}
#[get("/transactions/<bearer>")]
fn get_transactions(bearer: String) -> status::Custom<String> {
    if !key_valid(&bearer) {
        return status::Custom(Status::Unauthorized, String::from("Unauthorized"));
    }
    let u = get_key(bearer).unwrap().for_user_uuid.clone();
    let mut transactions = String::new();
    unsafe {
        for user in &USERS {
            if user.username == u {
                for transaction in &user.transaction {
                    transactions.push_str(&transaction.to_string());
                    transactions.push('\n');
                }
            }
        }
    }
    status::Custom(Status::Ok, transactions)
}
#[post("/dump/<bearer>")]
fn delete_all(bearer: String) -> status::Custom<String> {
    if !key_valid(&bearer) {
        return status::Custom(Status::Unauthorized, String::from("Unauthorized"));
    }
    let u = get_key(bearer).unwrap().for_user_uuid.clone();
    unsafe {
        for user in &mut USERS {
            if user.username == u {
                user.transaction.clear();
            }
        }
    }
    status::Custom(Status::Ok, String::from("OK"))
}
#[get("/")]
fn index() -> &'static str {
    "What are you looking for here, buddy?"
}
#[post("/create", data="<form>")]
fn sign_up(form: SignUp) -> Result<String, status::Custom<String>> {
    let mut file = File::open("users").unwrap();
    file.write(format!("{};{};\n", form.username, form.password).as_bytes()).map_err(|e| status::Custom(Status::InternalServerError, e.kind().to_string()))?;
    unsafe {
        USERS.push(User {
            username: form.username.clone(),
            password_hash: form.password.clone(),
            transaction: Vec::new()
        });
    }
    Ok(String::from("Success"))
}
#[launch]
async fn rocket() -> _ {
    let mut file = File::open("users").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let split = contents.split('\n');
    for line in split {
        let line = line.trim();
        let mut split = line.split(';');
        unsafe {
            USERS.push(User {
                username: split.next().unwrap().parse().unwrap(),
                password_hash: split.next().unwrap().parse().unwrap(),
                transaction: Vec::new()
            });
        }
    }
    println!("Parsed {} users", unsafe { USERS.len() });
    rocket::build()
        .mount("/", routes![index, key, transact, balance, get_transactions, sign_up, delete_all])
}