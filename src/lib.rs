// https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
use polars::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub(crate) fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn vcd2df<P: AsRef<Path>>(filename: P) -> DataFrame {
    let mut lines = read_lines(filename).expect("File DNE");
    let mut names = BTreeMap::<String, String>::new();
    let mut track = BTreeMap::<String, ()>::new();
    let mut stage = 0;
    // Stage 0: Read to `dumpvars` and first time point
    while stage < 2 {
        let line = lines
            .next()
            .expect("VCD ill-formed wrt dumpvars.")
            .expect("VCD ill-formed wrt lines.");
        // Basically, we must see both a dumpvars...
        if line.trim() == "$dumpvars" {
            stage += 1;
        }
        // And a timepoint 0 to progress.
        if line.trim() == "#0" {
            stage += 1;
        }
        let mut splits = line.split(" ");
        let word = splits.next().expect("VCD line ill-formed.");
        if word == "$var" {
            splits.next();
            splits.next(); // Consume var/reg and size
            let nick = String::from(splits.next().expect("Varname illformed")); // VCD nickname  
            let name = String::from(splits.next().expect("Varname illformed")); // Verilog reg/var name
            if name.contains("shadow_") {
                if !track.contains_key(&name) {
                    names.insert(nick,name.clone());
                    track.insert(name.clone(),());
                }
            }
        }
    }
    // Intermediate - stage the value storage
    // Polars can interpret integer options as nullable ints
    // Pico uses 32 bit, use 64 until we figure out to how to select.
    let mut curr = BTreeMap::<String, Option<u64>>::new();
    for key in names.keys() {
        curr.insert(key.clone(), None);
    }
    let names: Vec<String> = names.into_values().collect();
    let mut times: Vec<Column> = vec![Column::new("Names".into(), names)];
    let mut time = String::from("#0");
    // Stage 1: Read times into a BTreeMap
    while let Some(Ok(line)) = lines.next() {
        if line.chars().nth(0).expect("Line ill-formed") == '#' {
            let tmp: Vec<Option<u64>> = curr.values().cloned().collect();
            times.push(Column::new(time.into(), tmp));
            time = String::from(&line);
        }
        // Two cases - singular or plural
        if line.contains(char::is_whitespace) {
            // Plural
            let mut splits = line.split(" ");
            let mut num = splits.next().unwrap().chars();
            num.next(); // Clip the 'b'
            let num = num.as_str();
            let num = u64::from_str_radix(num, 2).ok();
            let reg = splits.next().unwrap();
            if curr.contains_key(reg) {
                curr.insert(String::from(reg), num);
            }
        } else {
            let mut line = line.chars();
            let num = u64::from_str_radix(&line.next().unwrap().to_string(), 2).ok();
            let reg = line.as_str();
            if curr.contains_key(reg) {
                curr.insert(String::from(reg), num);
            }
        }
    }
    return DataFrame::new(times).unwrap();
}
