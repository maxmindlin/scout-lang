use colored::Colorize;
use scout_interpreter::object::Object;

pub fn pprint(obj: Object) {
    match obj {
        Object::Null => println!("{}", "Null".green()),
        Object::Str(s) => println!("{}", s.yellow()),
        Object::Node(_) => println!("Node"),
        Object::Map(hash) => {
            println!("{}", "{".green());
            for (i, o) in hash.iter() {
                print!("\t{}: ", i.name.green());
                println!("{}", o);
            }
            println!("{}", "}".green());
        }
        Object::List(v) => println!("[Node; {}]", v.len()),
    }
}
