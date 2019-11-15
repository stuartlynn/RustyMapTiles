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

#[database("database")]
struct DbConn(diesel::pg::PgConnection);


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
          *,
          \"GEOID\" as id,
          ST_AsMVTGeom(
              geom,
              ST_TRANSFORM(ST_MakeEnvelope({x_min}, {y_min}, {x_max}, {y_max}, 3857), 4326),
              4096,
              256,
              true
          ) mvt_geom
      FROM (
        select *, geometry as geom from {tile_table}
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

  let result: Tile = sql_query(query)
    .get_result(&*conn)
    .expect("query failed to run");

  result.mvt
}

fn main() {
  rocket::ignite()
    .attach(DbConn::fairing())
    .mount("/", routes![ tile])
    .mount("/public", StaticFiles::from("./public"))
    .launch();
}
