// xfail-pretty
// chameneos

import io::reader_util;

use std;
import std::map;
import std::map::hashmap;
import std::sort;
import stream::{stream, chan, port};

// After a snapshot, this should move into core, or std.
mod stream {
    import option::unwrap;

    proto! streamp {
        open:send<T: send> {
            data(T) -> open<T>
        }
    }

    type chan<T:send> = { mut endp: option<streamp::client::open<T>> };
    type port<T:send> = { mut endp: option<streamp::server::open<T>> };

    fn stream<T:send>() -> (chan<T>, port<T>) {
        let (c, s) = streamp::init();
        ({ mut endp: some(c) }, { mut endp: some(s) })
    }

    impl chan<T: send> for chan<T> {
        fn send(+x: T) {
            let mut endp = none;
            endp <-> self.endp;
            self.endp = some(
                streamp::client::data(unwrap(endp), x))
        }
    }

    impl port<T: send> for port<T> {
        fn recv() -> T {
            let mut endp = none;
            endp <-> self.endp;
            let streamp::data(x, endp) = unwrap(
                pipes::recv(unwrap(endp)));
            self.endp = some(endp);
            x
        }
    }

   fn select<T:send>(ps: ~[port<T>]) -> (uint, T) {
      // endp swapping
      let mut endps;
      for vec::each(ps) |pp| {
         let mut endp = none;
         endp <-> pp.endp;
         vec::push(endps, endp);
      }

      let unwrapped_endps = vec::map(endps, |x| unwrap(x));

      let (ready, result, remaining) = pipes::select(unwrapped_endps);
      let streamp::data(x, (endps[ready])) = unwrap(result);

      // endp swapping
      for vec::eachi(ps) |ii, pp| {
         pp.endp = some(endps[ii]);
      }

      (ready, x)
   }

}

fn print_complements() {
   let all = ~[Blue, Red, Yellow];
   for vec::each(all) |aa| {
      for vec::each(all) |bb| {
         io::println(show_color(aa) + " + " + show_color(bb) + " -> " + show_color(transform(aa,bb)));
      }
   }
}

// can I combine these two lines?
enum color_e { Red, Yellow, Blue }
type color = color_e;

type creature_info = { name: uint, color: color };

fn show_color(cc: color) -> str {
   alt (cc) {
      Red    {"red"}
      Yellow {"yellow"}
      Blue   {"blue"}
   }
}

fn show_color_list(set: ~[color]) -> str {
   let mut out = "";
   for vec::eachi(set) |ii, col| {
      out += " ";
      out += show_color(col);
   }
   ret out;
}

fn show_digit(nn: uint) -> str {
   alt (nn) {
      0 {"zero"}
      1 {"one"}
      2 {"two"}
      3 {"three"}
      4 {"four"}
      5 {"five"}
      6 {"six"}
      7 {"seven"}
      8 {"eight"}
      9 {"nine"}
      _ {fail "expected digits from 0 to 9..."}
   }
}

fn show_number(nn: uint) -> str {
   let mut out = "";
   let mut num = nn;
   let mut dig;
   
   if num == 0 { out = show_digit(0) };

   while num != 0 {
      dig = num % 10;
      num = num / 10;
      out = show_digit(dig) + " " + out;
   }

   ret out;
}

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
   from_rendezvous: stream::port<option<creature_info>>,
   to_rendezvous: stream::chan<creature_info>,
   to_rendezvous_log: stream::chan<str>
) {
   let mut color = color;
   let mut creatures_met = 0;
   let mut evil_clones_met = 0;

   loop {
      // ask for a pairing
      to_rendezvous.send({name: name, color: color});
      let resp = from_rendezvous.recv();

      // log and change, or print and quit
      alt resp {
         option::some(other_creature) {
            color = transform(color, other_creature.color);

            // track some statistics
            creatures_met += 1;
            if other_creature.name == name {
               evil_clones_met += 1;
            }
         }
         option::none {
            // log creatures met and evil clones of self
            let report = #fmt("%u", creatures_met) + " " + show_number(evil_clones_met);
            to_rendezvous_log.send(report);
            break;
         }
      }
   }
}

fn rendezvous(nn: uint, set: ~[color]) {
   let streams = vec::map(set, |_col| some(stream()));
   let streams = vec::to_mut(streams);
   let mut from_creatures     = ~[];

   let streams_log = vec::map(set, |_col| some(stream()));
   let streams_log = vec::to_mut(streams_log);
   let mut from_creatures_log = ~[];

   let to_creature = vec::mapi(set, |ii, col| {
      let mut stream = none;
      stream <-> streams[ii];
      let (to_rendezvous_, from_creature_) = option::unwrap(stream);

      let mut stream_log = none;
      stream_log <-> streams_log[ii];
      let (to_rendezvous_LOG_, from_creature_LOG_) = option::unwrap(stream_log);

      vec::push(from_creatures, from_creature_);
      vec::push(from_creatures_log, from_creature_LOG_);

      let (to_creature, from_rendezvous) = stream::stream();

      do task::spawn_with(from_rendezvous) |from_parent| {
         creature(ii, col, from_rendezvous, to_rendezvous_, to_rendezvous_LOG_);
      };

      to_creature
   });

   let mut meetings = 0;
   let mut creatures_met = 0;
   let mut creatures_present = 0;

   let mut first_creature  = { name: 0, color: Red }; // use option type instead
   let mut second_creature = { name: 0, color: Red }; // of initializing to junk?

   // set up meetings...
   while meetings < nn {
      let (_creature_num, creature_req): (uint, creature_info) = stream::select(from_creatures);
      creatures_met += 1;

      alt creatures_present {
         0 {
             first_creature = creature_req;
             creatures_present = 1;
           }
         1 {
             second_creature = creature_req;
             to_creature[first_creature.name].send(some(second_creature));
             to_creature[second_creature.name].send(some(first_creature));
             creatures_present = 0;
             meetings += 1;
           }
         _ { fail "too many creatures are here!" }
      }
   }

   // tell each creature to stop
   for vec::eachi(to_creature) |ii, to_one| {
      to_one.send(none);
   }

   // save each creature's meeting stats
   // note, the order isn't important, it doesn't need to be vec::each
   let mut reports = ~[];
   for vec::each(to_creature) |_to_one| {
      let (_num, report): (uint, str) = stream::select(from_creatures_log);
      vec::push(reports, report);
   }

   // print each color in the set
   io::println(show_color_list(set));

   // print each creature's stats
   for vec::each(reports) |rep| {
      io::println(rep);
   }

   // print the total number of creatures met
   io::println(show_number(creatures_met));
}

fn main(args: ~[str]) {
   let nn = if os::getenv("RUST_BENCH").is_some() {
      600
   } else {
      option::get(uint::from_str(args[1]))
   };

   print_complements();
   io::println("");

   rendezvous(nn, ~[Blue, Red, Yellow]);
   io::println("");

   rendezvous(nn, ~[Blue, Red, Yellow, Red, Yellow, Blue, Red, Yellow, Red, Blue]);
}

