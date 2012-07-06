// xfail-pretty
// chameneos

import io::reader_util;

use std;
import std::map;
import std::map::hashmap;
import std::sort;

/*
fn make_sequence_processor(sz: uint, from_parent: comm::port<~[u8]>,
                           to_parent: comm::chan<str>) {
   
   let freqs: hashmap<~[u8], uint> = map::bytes_hash();
   let mut carry: ~[u8] = ~[];
   let mut total: uint = 0u;

   let mut line: ~[u8];

   loop {

      line = comm::recv(from_parent);
      if line == ~[] { break; }

       carry = windows_with_carry(carry + line, sz, |window| {
         update_freq(freqs, window);
         total += 1u;
      });
   }

   let buffer = alt sz { 
   };

   //comm::send(to_parent, #fmt["yay{%u}", sz]);
   comm::send(to_parent, buffer);
}
*/

fn print_complements() {
}

// can I combine these two lines?
enum color_e { Red, Yellow, Blue }
type color = color_e;

type creature_info = { name: uint, color: color };

fn living_creature(
   name: uint,
   color: color,
   from_rendezvous: comm::port<option<creature_info>>,
   to_rendezvous: comm::chan<creature_info>
) {
   loop {
      // ask for a pairing
      comm::send(to_rendezvous, {name: name, color: color});
      let resp = comm::recv(from_rendezvous);

      // log and change, or print and quit
      alt resp {
         option::some(other_creature) { }
         option::none { }
      }
   }
}

fn rendezvous(nn: uint, set: ~[color]) {
   let from_creatures: comm::port<creature_info> = comm::port();
   let to_rendezvous = comm::chan(from_creatures);
   let to_creature: ~[comm::chan<option<creature_info>>] =
      vec::mapi(set, fn@(ii: uint, col: color) -> comm::chan<option<creature_info>> {
         ret do task::spawn_listener |from_rendezvous| {
            living_creature(ii, col, from_rendezvous, to_rendezvous);
         };
      });

   let request = comm::recv(from_creatures);
         
/*
   
   // latch stores true after we've started
   // reading the sequence of interest
   let mut proc_mode = false;

   while !rdr.eof() {
      let line: str = rdr.read_line();

      if str::len(line) == 0u { cont; }

      alt (line[0], proc_mode) {

         // start processing if this is the one
         ('>' as u8, false) {
            alt str::find_str_from(line, "THREE", 1u) {
               option::some(_) { proc_mode = true; }
               option::none    { }
            }
         }

         // break our processing
         ('>' as u8, true) { break; }

         // process the sequence for k-mers
         (_, true) {
            let line_bytes = str::bytes(line);

           for sizes.eachi |ii, _sz| {
               let mut lb = line_bytes;
               comm::send(to_child[ii], lb);
            }
         }

         // whatever
         _ { }
      }
   }

   // finish...
    for sizes.eachi |ii, _sz| {
      comm::send(to_child[ii], ~[]);
   }

   // now fetch and print result messages
    for sizes.eachi |ii, _sz| {
      io::println(comm::recv(from_child[ii]));
   }
*/
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

