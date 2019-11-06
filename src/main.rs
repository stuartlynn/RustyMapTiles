#![feature(proc_macro_hygiene, decl_macro)]
extern crate postgres;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate diesel;

use diesel::query_dsl::RunQueryDsl;
use diesel::sql_types::{BigInt, Bytea, Integer, Text};

use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::serve::StaticFiles;

// use rocket_contrib::databases::diesel;
use std::f64::consts::PI;

use diesel::sql_query;

#[derive(QueryableByName, PartialEq, Debug)]
struct Tile {
  // #[sql_type = "Integer"]
  // id: i32,
  #[sql_type = "Bytea"]
  mvt: Vec<u8>,
}

#[derive(QueryableByName, PartialEq, Debug, Serialize, Deserialize)]
struct OD {
  #[sql_type = "Text"]
  id: String,
  #[sql_type = "Integer"]
  count: i32,
}

fn bbox(x: u32, y: u32, z: u32) -> [f64; 4] {
  let max = 6378137.0 * PI;
  let res = max * 2.0 / (2.0 as f64).powf(z as f64) as f64;

  [
    -max + (x as f64) * res,
    max - ((y as f64) * res),
    -max + (x as f64) * res + res,
    max - ((y as f64) * res) - res,
  ]
}

#[database("census_geoms")]
struct CensusDbConn(diesel::pg::PgConnection);

#[get("/hello/<name>/<age>")]
fn hello(name: String, age: u8) -> String {
  format!("Hello, {} year old named {}!", age, name)
}

#[get("/od_multi/<origin>/<ids>")]
fn od_multi(conn: CensusDbConn, origin: String, ids: String) -> JsonValue {
  let origin_column = match origin.as_ref() {
    "home" => "h_geocode",
    "work" => "w_geocode",
    _ => panic!("needs to be work or home"),
  };

  let dest_column = match origin.as_ref() {
    "home" => "w_geocode",
    "work" => "h_geocode",
    _ => panic!("needs to be work or home"),
  };

  let query = format!(
    "select {dest_column}::TEXT as id, SUM(\"S000\")::INTEGER as count  from lodes_od where
    {origin_column} IN ({ids}) 
    group by {dest_column} 
    ",
    origin_column = origin_column,
    dest_column = dest_column,
    ids = ids
  );

  let result: Vec<OD> = sql_query(query)
    .get_results(&*conn)
    .expect("query failed to run");
  json!(result)
}

#[get("/od/<origin>/<id>")]
fn od(conn: CensusDbConn, origin: String, id: String) -> JsonValue {
  let origin_column = match origin.as_ref() {
    "home" => "h_geocode",
    "work" => "w_geocode",
    _ => panic!("needs to be work or home"),
  };

  let dest_column = match origin.as_ref() {
    "home" => "w_geocode",
    "work" => "h_geocode",
    _ => panic!("needs to be work or home"),
  };

  let query = format!(
    "select {dest_column}::TEXT as id, \"S000\"::INTEGER as count  from lodes_od where
    {origin_column} = \'{id}\' 
    ",
    origin_column = origin_column,
    dest_column = dest_column,
    id = id
  );

  let result: Vec<OD> = sql_query(query)
    .get_results(&*conn)
    .expect("query failed to run");
  json!(result)
}

#[get("/<tile_type>/<z>/<x>/<y>")]
fn tile(conn: CensusDbConn, tile_type: String, z: u32, x: u32, y: u32) -> Vec<u8> {
  println!("Getting tile {} {} {}", z, x, y);
  let tile_box = bbox(x, y, z);
  println!(
    "tile box is {} {} {} {}",
    tile_box[0], tile_box[1], tile_box[2], tile_box[3]
  );

  let query = format!(
    "
    SELECT ST_AsMVT(q, '{tile_table}', 4096, 'mvt_geom') as mvt 
    FROM (
      SELECT
          \"GEOID\" as id,
          ST_AsMVTGeom(
              geom,
              ST_TRANSFORM(ST_MakeEnvelope({x_min}, {y_min}, {x_max}, {y_max}, 3857), 4326),
              4096,
              256,
              true
          ) mvt_geom
      FROM (
        select \"GEOID\", geometry as geom from {tile_table}
        where geometry && ST_TRANSFORM(ST_MakeEnvelope({x_min}, {y_min}, {x_max}, {y_max}, 3857), 4326)
      ) c
    ) q
    limit 1 
    ;
    ",
    tile_table = tile_type,
    x_min = tile_box[0],
    y_min = tile_box[1],
    x_max = tile_box[2],
    y_max = tile_box[3]
  );
  println!("{}", query);

  let result: Tile = sql_query(query)
    .get_result(&*conn)
    .expect("query failed to run");

  result.mvt
}

fn main() {
  rocket::ignite()
    .attach(CensusDbConn::fairing())
    .mount("/", routes![hello, tile, od, od_multi])
    .mount("/public", StaticFiles::from("./public"))
    .launch();
}
