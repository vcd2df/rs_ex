use polars::prelude::*;

use std::collections::BTreeMap;

const PATH: &str = "../../vcds";

fn main() {
    let paths = std::fs::read_dir(PATH).unwrap();
    let mut iflows = BTreeMap::<String, BTreeMap::<String, u64>>::new();
    for path in paths {
        let path = path.unwrap().path();
        let mut df = rs_ex::vcd2df(&path);
        let names = df.column("Names").unwrap().str().unwrap();
        let mut iflow = BTreeMap::<String, u64>::new();
        let mut vector:Vec<String> = vec!();
        for name in names {
            vector.push(String::from(name.unwrap()));
        }
        df = df.drop("Names").unwrap();
        for colname in df.get_column_names() {
            let column = df.column(colname).unwrap().u64().unwrap();
            let mut i = 0;
            for row in column {
                if row.unwrap() > 0 {
                    if !iflow.contains_key(&vector[i].replace("shadow_","")) {
                        iflow.insert(vector[i].clone().replace("shadow_",""),(&colname[1..]).parse().unwrap());
                    }
                }
                i += 1;
            }

        }
        iflows.insert(path.display().to_string().replace(PATH,"").replace(".vcd",""), iflow);
    }
    println!("{:?}",iflows); 
}
