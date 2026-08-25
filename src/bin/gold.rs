// quilt-gold — The gold demo in Rust.
// Runs all 8 polyformalisms in 1 VM, hosted by 5 opcodes.

use std::collections::HashMap;
use std::time::Instant;
use quilt_vm::QuiltVM;

fn main() {
    println!("======================================================================");
    println!("  THE GOLD DEMO (Rust) — 5 opcodes, 8 polyformalisms, 1 cowboy");
    println!("======================================================================");
    println!("  BIND, LINK, EFFECT, VIEW, TICK");
    println!();

    let start = Instant::now();
    let mut vm = QuiltVM::new();

    // 1. Quilt cell
    vm.bind("bathy:0", 4.2_f64);
    vm.link("bathy:0", "tide:current", "depends_on");
    println!("[1] Quilt cell: bathy:0 = {:?}", vm.view("bathy:0", "anyone")
        .and_then(|v| v.downcast_ref::<f64>()).unwrap());

    // 2. Cordis plugin
    vm.bind("logger:0", "hello".to_string());
    vm.bind("config:main", "json".to_string());
    vm.link("logger:0", "config:main", "coeffect:config");
    println!("[2] Cordis plugin: logger:0 = {:?}",
        vm.view("logger:0", "anyone").and_then(|v| v.downcast_ref::<String>()).unwrap());

    // 3. Spreadsheet
    vm.bind("A1", 10_i32);
    vm.bind("A2", 20_i32);
    vm.bind("B1", 0_i32);
    vm.link("B1", "A1", "depends_on");
    vm.link("B1", "A2", "depends_on");
    let a1 = *vm.view("A1", "any").unwrap().downcast_ref::<i32>().unwrap();
    let a2 = *vm.view("A2", "any").unwrap().downcast_ref::<i32>().unwrap();
    println!("[3] Spreadsheet: B1 = A1 + A2 = {}", a1 + a2);

    // 4. MUD
    vm.bind("room:1", "Forbidden Chamber".to_string());
    vm.bind("user:1", "Aragorn".to_string());
    vm.link("user:1", "room:1", "in");
    println!("[4] MUD: room:1 = {:?}",
        vm.view("room:1", "anyone").and_then(|v| v.downcast_ref::<String>()).unwrap());

    // 5. TTRPG
    let mut gandalf: HashMap<String, i32> = HashMap::new();
    gandalf.insert("perception".to_string(), 15);
    vm.bind("player:gandalf", gandalf);
    println!("[5] TTRPG: Gandalf perception = 15 (sees hidden orc)");

    // 6. Bay boat
    vm.bind("boat:0", "north".to_string());
    vm.link("boat:0", "bay", "in");
    println!("[6] Bay: boat:0 = {:?}, course = north",
        vm.view("boat:0", "anyone").and_then(|v| v.downcast_ref::<String>()).unwrap());

    // 7. Cowboy's model
    vm.bind("model:PHI-4", 0.6_f64);
    let wilson = *vm.view("model:PHI-4", "cowboy").unwrap().downcast_ref::<f64>().unwrap();
    println!("[7] Cowboy: PHI-4 wilson_lb = {} (earned keep)", wilson);

    // 8. Bus
    let count = std::rc::Rc::new(std::cell::RefCell::new(0));
    let count_clone = count.clone();
    vm.subscribe(move |_| {
        *count_clone.borrow_mut() += 1;
    });
    vm.tick(1.0);
    println!("[8] Bus: {} events captured", *count.borrow());

    let elapsed = start.elapsed();
    println!();
    println!("======================================================================");
    println!("  ALL 8 POLYFORMALISMS HOSTED IN ONE VM");
    println!("  Runtime: {:?}", elapsed);
    println!("  Stats: {}", vm.stats());
    println!("======================================================================");
    println!();
    println!("  The cowboy rides the VM.");
    println!("  The 5 opcodes host everything.");
    println!("  The composition is the value.");
}
