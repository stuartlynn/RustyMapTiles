# RustyMapTiles

This is a quick and dirty MVT tile server written in rust. It connects to a POSTGIS instance and serves tiles for a given table.

## Installing

This uses rocekt which requires nightly rust. To install rust up

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then install the nightly toolchain

```bash
rustup default nightly
```

Finally run

```bash
cargo run
```

Which should start a server on http://localhost:8000

## Config

To configure your database connect edit the Rocket.toml file to point to whatever database you want

```toml
[global.databases]
census_geoms  = {url = "postgresql://census:census@localhost/census"}
```

If you want to use the built in map viewer you will need to also need to provide a mapboxgl token in
the public/index.html file

```
mapboxgl.accessToken = '';
```

## Tiles

You can get tiles from the endpoint

```
http://localhost:8000/{tablename}/{z}/{x}/{y}
```

Where tablename is the table in your postgis database
These will have the same source-layer id as the name of the table and all the attributes on the table as properties!

Happy mapping!
