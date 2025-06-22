extern crate core;

mod game;
mod linestore;
pub mod physics;
pub mod rider;

pub use game::*;

#[cfg(test)]
mod tests {
    use std::fs;
    use lr_formatter_rs::trackjson::read;

    #[test]
    fn crash() {
      let track_bytes = fs::read_to_string("./fixtures/crash.track.json").expect("Failed to read file");
      let track = read(&track_bytes).expect("Failed to parse file");
      println!("{}", track.title);
    }

    #[test]
    fn cycloid() {
      let track_bytes = fs::read_to_string("./fixtures/cycloid.track.json").expect("Failed to read file");
      let track = read(&track_bytes).expect("Failed to parse file");
      println!("{}", track.title);
    }

    #[test]
    fn legacy_test() {
      let track_bytes = fs::read_to_string("./fixtures/legacyTestTrack.track.json").expect("Failed to read file");
      let track = read(&track_bytes).expect("Failed to parse file");
      println!("{}", track.title);
    }

    #[test]
    fn modern_test() {
      let track_bytes = fs::read_to_string("./fixtures/testTrack.track.json").expect("Failed to read file");
      let track = read(&track_bytes).expect("Failed to parse file");
      println!("{}", track.title);
    }
}
