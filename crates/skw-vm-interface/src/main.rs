use skw_vm_interface::call::Caller;

use skw_vm_store::create_store;
use skw_vm_primitives::contract_runtime::{CryptoHash, AccountId};
use std::convert::TryInto;
use rocket::form::Form;

static mut CALLER: Option<Caller> = None;

#[derive(rocket::FromForm)]
struct Init<'v> {
    state_file_path: &'v str,
    state_root: &'v str,
}

#[derive(rocket::FromForm)]
struct Payload<'v> {
    payload: &'v str,
}


#[rocket::post("/init", data = "<init>")]
fn init<'v>(init: Form<Init<'_>>) {
    unsafe {
        if CALLER.is_some() {
            return
        }
    }

    let store = create_store();
    store.load_state_from_file(init.state_file_path).unwrap();
    let state_root: CryptoHash = hex::decode(init.state_root).unwrap().try_into().expect("state_root should have length 32");

    unsafe {
        CALLER = Some(Caller::new(store.clone(), state_root, AccountId::test(), None));
    }
}

#[rocket::post("/call", data = "<payload>")]
fn call<'r>(
    payload: Form<Payload<'_>>
) -> String {
    let pld = hex::decode(payload.payload).expect("invalid payload");
    unsafe {
        match &mut CALLER {
            Some(c) => hex::encode( c.call_payload(&pld[..]) ),
            None => "init first".to_string()
        }
    }
}


#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes![init, call])
}
