// xfail-pretty
// chameneos

import io::reader_util;

use std;
import std::map;
import std::map::hashmap;
import std::sort;

fn print_complements() {
}

// can I combine these two lines?
enum color_e { Red, Yellow, Blue }
type color = color_e;

type creature_info = { name: uint, color: color };

fn transform(aa: color, bb: color) -> color {
   alt (aa, bb) {
      (Red,    Red   ) { Red    }
      (Red,    Yellow) { Blue   }
      (Red,    Blue  ) { Yellow }
      (Yellow, Red   ) { Blue   }
      (Yellow, Yellow) { Yellow }
      (Yellow, Blue  ) { Red    }
      (Blue,   Red   ) { Yellow }
      (Blue,   Yellow) { Red    }
      (Blue,   Blue  ) { Blue   }
   }
}

fn creature(
   name: uint,
   color: color,
   from_rendezvous: comm::port<option<creature_info>>,
   to_rendezvous: comm::chan<creature_info>
) {
   let mut color = color;

   loop {
      // ask for a pairing
      comm::send(to_rendezvous, {name: name, color: color});
      let resp = comm::recv(from_rendezvous);

      // log and change, or print and quit
      alt resp {
         option::some(other_creature) {
            io::print("x");
            color = transform(color, other_creature.color);
         }
         option::none { break; }
      }
   }
}

fn rendezvous(nn: uint, set: ~[color]) {
   let from_creatures: comm::port<creature_info> = comm::port();
   let to_rendezvous = comm::chan(from_creatures);
   let to_creature: ~[comm::chan<option<creature_info>>] =
      vec::mapi(set, fn@(ii: uint, col: color) -> comm::chan<option<creature_info>> {
         ret do task::spawn_listener |from_rendezvous| {
            creature(ii, col, from_rendezvous, to_rendezvous);
         };
      });

   let mut meetings = 0;
   let mut creatures_met = 0;
   let mut creatures_present = 0;

   // TODO: option type rather than invite bugs using these values
   let mut first_creature = { name: 0, color: Red };
   let mut second_creature = { name: 0, color: Red };

   // set up meetings...
   while meetings < nn {
      let creature_req: creature_info = comm::recv(from_creatures);
      creatures_met += 1;

      alt creatures_present {
         0 {
             first_creature = creature_req;
             creatures_present = 1;
           }
         1 {
             io::print(".");
             second_creature = creature_req;
             comm::send(to_creature[first_creature.name], some(second_creature));
             comm::send(to_creature[second_creature.name], some(first_creature));
             creatures_present = 0;
             meetings += 1;
           }
         _ { fail "too many creatures are here!" }
      }
   }

   // tell each creature to stop
   for vec::eachi(to_creature) |ii, to_one| {
      comm::send(to_one, none);
   }
}

fn main(args: ~[str]) {
   let nn = if os::getenv("RUST_BENCH").is_some() {
      600
   } else {
      // TODO: convert arg0 to uint
      600
   };

   print_complements();
   rendezvous(nn, ~[Blue, Red, Yellow]);
   rendezvous(nn, ~[Blue, Red, Yellow, Red, Yellow, Blue, Red, Yellow, Red, Blue]);
}

