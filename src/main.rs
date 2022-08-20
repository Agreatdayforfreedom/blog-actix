use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[macro_use]
extern crate diesel;

pub mod schema;
pub mod models;

use tera::Tera;
use dotenv::dotenv;
use std::env;

use diesel::r2d2::{self, ConnectionManager};
use diesel::r2d2::Pool;

use diesel::prelude::*;
use diesel::pg::PgConnection;

use self::models::{Post, NewPost, NewPostHandler};
use self::schema::posts;
use self::schema::posts::dsl::*;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/")]
async fn index(pool: web::Data<DbPool>, template_manager: web::Data<tera::Tera>) -> impl Responder {
    let conn = pool.get().expect("Problemas al conectar con la db");

    match web::block(move || { posts.load::<Post>(&conn)}).await {
        Ok(data) => {
            let mut ctx = tera::Context::new();
            let data = data.unwrap();
            ctx.insert("posts", &data);
            HttpResponse::Ok().content_type("text/html").body(
            template_manager.render("index.html", &ctx).unwrap())
     //   HttpResponse::Ok().body(format!("{:?}", data))
    },
        Err(err) => HttpResponse::Ok().body("Error")
    }
    
}

#[get("/blog/{blog_slug}")]
async fn get_slug(pool: web::Data<DbPool>, template_manager: web::Data<tera::Tera>, blog_slug: web::Path<String>) -> impl Responder {
    let conn = pool.get().expect("Problemas al conectar con la db");

    let url_slug = blog_slug.into_inner();

    match web::block(move || { posts.filter(slug.eq(url_slug)).load::<Post>(&conn)}).await {
        Ok(data) => {
            let data = data.unwrap();
            
            if data.len() == 0 {
                return HttpResponse::NotFound().finish();
            }

            let data = &data[0];
            
            let mut ctx = tera::Context::new();
            ctx.insert("post", &data);
            HttpResponse::Ok().content_type("text/html").body(
            template_manager.render("post.html", &ctx).unwrap())
     //   HttpResponse::Ok().body(format!("{:?}", data))
    },
        Err(err) => HttpResponse::Ok().body("Error")
    }
    
}

#[post("/new_post")]
async fn new_post(pool: web::Data<DbPool>, item: web::Json<NewPostHandler>) -> impl Responder {
    let conn = pool.get().expect("Problemas al conectar con la db");

    match web::block(move || {Post::create_post(&conn, &item)}).await {
        Ok(data) => {
        HttpResponse::Ok().body(format!("{:?}", data))
    },
        Err(err) => HttpResponse::Ok().body("Error")
    }
    
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("error to find db url");
    let port = env::var("PORT").expect("error port");
    let port: u16 = port.parse().unwrap();
    let conn = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder().build(conn).expect("No build pool");

    HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        App::new()
        .service(index)
        .service(new_post)
        .service(get_slug)
        .app_data(web::Data::new(pool.clone()))
        .app_data(web::Data::new(tera))
    }).bind(("0.0.0.0", port))?.run().await
    
}





/* ! crud with diesel */
/* #[macro_use]
extern crate diesel;

pub mod schema;
pub mod models;

use dotenv::dotenv;
use std::env;

use diesel::prelude::*;
use diesel::pg::PgConnection;


fn main() {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("error to find db url");

    let connection = PgConnection::establish(&db_url).expect("Error connecting to db");

    use self::models::{Post, NewPost};
    use self::schema::posts;
    use self::schema::posts::dsl::*;

    let new_post = NewPost {
        title: "second blogpost",
        body: "some body",
        slug: "fist post" 
    };

    diesel::insert_into(posts::table).values(&new_post).get_result::<Post>(&connection).expect("error inserting");

    diesel::update(posts.filter(id.eq(3))).set((slug.eq("tercer post"), title.eq("tercer title"))).get_result::<Post>(&connection).expect("update error");

    diesel::delete(posts.filter(slug.eq("tercer post"))).execute(&connection).expect("Ha fallado la eliminacion");

    let posts_result = posts.load::<Post>(&connection).expect("Query execuiton error");

    for post in posts_result {
        println!("{:?}", post);
    } 
} */
