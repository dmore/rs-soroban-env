use crate::{
    events::{DebugEvent, HostEvent},
    xdr::{
        ContractEvent, ContractEventBody, ContractEventType, ContractEventV0, ExtensionPoint, Hash,
        ScMap, ScMapEntry, ScObject::Map, ScVal,
    },
    ContractFunctionSet, Env, EnvBase, Host, HostError, RawVal, Symbol,
};
use expect_test::expect;
use std::rc::Rc;

pub struct ContractWithSingleEvent;

impl ContractFunctionSet for ContractWithSingleEvent {
    fn call(&self, _func: &Symbol, host: &Host, _args: &[RawVal]) -> Option<RawVal> {
        // Add a contract event
        let mut data = host.map_new().unwrap();
        data = host.map_put(data, 1_u32.into(), 2_u32.into()).unwrap();
        let mut topics = host.vec_new(().into()).unwrap();
        topics = host.vec_push_back(topics, 0u32.into()).unwrap();
        topics = host.vec_push_back(topics, 1u32.into()).unwrap();
        Some(host.contract_event(topics, data.to_raw()).unwrap())
    }
}

#[test]
fn contract_event() -> Result<(), HostError> {
    let host = Host::default();
    let dummy_id = [0; 32];
    let id = host.bytes_new_from_slice(&dummy_id)?;
    let test_contract = Rc::new(ContractWithSingleEvent {});
    let sym = Symbol::from_str("add");
    let args = host.test_vec_obj::<i32>(&[1, 2])?;
    host.register_test_contract(id, test_contract)?;
    assert_eq!(
        host.call(id, sym.into(), args.into())?.get_payload(),
        RawVal::from_void().get_payload()
    );

    let event_ref = ContractEvent {
        ext: ExtensionPoint::V0,
        contract_id: Some(Hash(dummy_id)),
        type_: ContractEventType::Contract,
        body: ContractEventBody::V0(ContractEventV0 {
            topics: host.map_err(vec![ScVal::U32(0), ScVal::U32(1)].try_into())?,
            data: ScVal::Object(Some(Map(host.map_err(ScMap::try_from(vec![
                ScMapEntry {
                    key: ScVal::U32(1),
                    val: ScVal::U32(2),
                },
            ]))?))),
        }),
    };

    // Fish out the last contract event and check that it is
    // correct, and formats as expected.
    let events = host.get_events()?;
    match events.0.last() {
        Some(HostEvent::Contract(ce)) => {
            assert_eq!(*ce, event_ref)
        }
        _ => {
            panic!("missing contract event")
        }
    };
    Ok(())
}

pub struct ContractWithMultipleEvents;

impl ContractFunctionSet for ContractWithMultipleEvents {
    fn call(&self, _func: &Symbol, host: &Host, _args: &[RawVal]) -> Option<RawVal> {
        let topics = host.test_vec_obj(&[0, 1]).unwrap();
        let data = RawVal::from(0u32);
        host.record_contract_event(ContractEventType::Contract, topics, data)
            .unwrap();
        host.record_debug_event(DebugEvent::new().msg("debug event 0"))
            .unwrap();
        host.record_contract_event(ContractEventType::System, topics, data)
            .unwrap();
        Some(().into())
    }
}

#[test]
fn test_event_rollback() -> Result<(), HostError> {
    let host = Host::default();
    let dummy_id = [0; 32];
    let id = host.bytes_new_from_slice(&dummy_id)?;
    let test_contract = Rc::new(ContractWithMultipleEvents {});
    let sym = Symbol::from_str("add");
    let args = host.test_vec_obj::<i32>(&[1, 2])?;
    host.register_test_contract(id, test_contract)?;
    assert_eq!(
        host.call(id, sym.into(), args.into())?.get_payload(),
        RawVal::from_void().get_payload()
    );
    host.0.events.borrow_mut().rollback(1, &host)?;

    let expected = expect![[
        r#"Events([Contract(ContractEvent { ext: V0, contract_id: Some(Hash(0000000000000000000000000000000000000000000000000000000000000000)), type_: Contract, body: V0(ContractEventV0 { topics: ScVec(VecM([I32(0), I32(1)])), data: U32(0) }) }), Debug(DebugEvent { msg: Some("debug event 0"), args: [] }), Debug(DebugEvent { msg: Some("rolled-back contract event: type {}, id {}, topics {}, data {}"), args: [Val(I32(0)), Val(Object(Bytes(#4))), Val(Object(Vec(#2))), Val(U32(0))] }), Debug(DebugEvent { msg: Some("{} contract events rolled back. Rollback start pos = {}"), args: [Val(U32(1)), Val(U32(1))] })])"#
    ]];
    let actual = format!("{:?}", host.0.events.borrow().externalize(&host)?);
    expected.assert_eq(&actual);
    Ok(())
}