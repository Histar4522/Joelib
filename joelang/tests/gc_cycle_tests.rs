use dumpster::unsync::Gc;
use joelang::value::{JoeInstance, JoeValue};
use std::cell::RefCell;

#[test]
fn test_dumpster_cycle_collection() {
    let alice_inst = Gc::new(RefCell::new(JoeInstance::new("Person".to_string())));
    let bob_inst = Gc::new(RefCell::new(JoeInstance::new("Person".to_string())));

    let alice_val = JoeValue::Person(alice_inst.clone());
    let bob_val = JoeValue::Person(bob_inst.clone());

    // Create cyclic reference in Joe's world: Alice <-> Bob
    alice_inst.borrow_mut().fields.insert("friend".to_string(), bob_val);
    bob_inst.borrow_mut().fields.insert("friend".to_string(), alice_val);

    // Verify cyclic references exist
    assert!(matches!(alice_inst.borrow().fields.get("friend"), Some(JoeValue::Person(_))));
    assert!(matches!(bob_inst.borrow().fields.get("friend"), Some(JoeValue::Person(_))));

    // Dropping these handles will trigger dumpster's automatic cycle detection and cleanup!
    drop(alice_inst);
    drop(bob_inst);
}
