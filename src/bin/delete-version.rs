// Purge all references to a crate's version from the database.
//
// Please be super sure you want to do this before running this.
//
// Usage:
//      cargo run --bin delete-version crate-name version-number

#![allow(unstable)]

extern crate "cargo-registry" as cargo_registry;
extern crate postgres;
extern crate time;
extern crate semver;

use std::os;
use std::io;

use cargo_registry::{Crate, Version};

fn main() {
    let conn = postgres::Connection::connect(env("DATABASE_URL").as_slice(),
                                             &postgres::SslMode::None).unwrap();
    {
        let tx = conn.transaction().unwrap();
        delete(&tx);
        tx.set_commit();
        tx.finish().unwrap();
    }
}

fn env(s: &str) -> String {
    match os::getenv(s) {
        Some(s) => s,
        None => panic!("must have `{}` defined", s),
    }
}

fn delete(tx: &postgres::Transaction) {
    let name = match os::args().get(1) {
        None => { println!("needs a crate-name argument"); return }
        Some(s) => s.to_string(),
    };
    let version = match os::args().get(2) {
        None => { println!("needs a version argument"); return }
        Some(s) => s.to_string(),
    };
    let version = semver::Version::parse(&version[]).unwrap();

    let krate = Crate::find_by_name(tx, name.as_slice()).unwrap();
    let v = Version::find_by_num(tx, krate.id, &version).unwrap().unwrap();
    print!("Are you sure you want to delete {}#{} ({}) [y/N]: ", name, version,
           v.id);
    let line = io::stdin().read_line().unwrap();
    if !line.starts_with("y") { return }

    println!("deleting version {} ({})", v.num, v.id);
    let n = tx.execute("DELETE FROM version_downloads WHERE version_id = $1",
                       &[&v.id]).unwrap();
    println!("  {} download records deleted", n);
    let n = tx.execute("DELETE FROM version_authors WHERE version_id = $1",
                       &[&v.id]).unwrap();
    println!("  {} author records deleted", n);
    let n = tx.execute("DELETE FROM dependencies WHERE version_id = $1",
                       &[&v.id]).unwrap();
    println!("  {} dependencies deleted", n);
    tx.execute("DELETE FROM versions WHERE id = $1",
               &[&v.id]).unwrap();

    print!("commit? [y/N]: ");
    let line = io::stdin().read_line().unwrap();
    if !line.starts_with("y") { panic!("aborting transaction"); }
}

