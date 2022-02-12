use std::{rc::Rc};
use std::cell::RefCell;
use std::convert::TryFrom;
use skw_vm_interface::{
    to_yocto, runtime::init_runtime, UserAccount,
};
use skw_vm_store::{create_store};
use skw_vm_primitives::contract_runtime::AccountId;

#[test]
fn test_dump_state_from_file() {

    let state_root = {
        let store = create_store();

        let (runtime, root_signer) = init_runtime(
            &"root", 
            None,
            Some(&store),
            None,
        );

        let root_account = UserAccount::new(
            &Rc::new(RefCell::new(runtime)),
            AccountId::try_from("root".to_string()).unwrap(), 
            root_signer
        );

        let _ = root_account
            .deploy(
                include_bytes!("../../skw-contract-sdk/examples/status-message-collections/res/status_message_collections.wasm")
                    .as_ref()
                    .into(),
                AccountId::try_from("status".to_string()).unwrap(),
                to_yocto("1"),
            );
        
        let _ = root_account.create_user(
            AccountId::try_from("alice".to_string()).unwrap(),
            to_yocto("100")
        );

        let status_account = root_account.borrow_runtime().view_account(&"status");
        let alice_account = root_account.borrow_runtime().view_account(&"alice");

        assert!(status_account.is_some());
        assert!(alice_account.is_some());

        store.save_state_to_file("./mock/new").unwrap();
        root_account.state_root()
    };

    {

        let store = create_store();
        store.load_state_from_file("./mock/new").unwrap();

        let (runtime, root_signer) = init_runtime(
            &"root", 
            None,
            Some(&store),
            Some(state_root),
        );

        let root_account = UserAccount::new(
            &Rc::new(RefCell::new(runtime)),
            AccountId::try_from("root".to_string()).unwrap(), 
            root_signer
        );

        let _ = root_account
            .deploy(
                include_bytes!("../../skw-contract-sdk/examples/status-message/res/status_message.wasm")
                    .as_ref()
                    .into(),
                AccountId::try_from("status_new".to_string()).unwrap(),
                to_yocto("1"),
            );
        
        let _ = root_account.create_user(
            AccountId::try_from("alice_new".to_string()).unwrap(),
            to_yocto("100")
        );

        // existing accounts in the state store
        let status_account = root_account.borrow_runtime().view_account(&"status");
        let alice_account = root_account.borrow_runtime().view_account(&"alice");

        assert!(status_account.is_some());
        assert!(alice_account.is_some());

        // newly created accounts in the state store
        let status_account = root_account.borrow_runtime().view_account(&"status_new");
        let alice_account = root_account.borrow_runtime().view_account(&"alice_new");

        assert!(status_account.is_some());
        assert!(alice_account.is_some());
    };
}
