use crate::base::RchatCommand;
use std::collections::HashMap;
use std::net::SocketAddr;

pub trait HandlerProtocolData {
    fn handle(&mut self, address: SocketAddr, data: &Vec<u8>) -> Option<Vec<u8>>;
}

pub struct HandleProtocolFactory {
    pub all_handler: HashMap<RchatCommand, Box<dyn HandlerProtocolData + 'static + Send>>,
}

impl HandleProtocolFactory {
    pub fn new() -> Self {
        HandleProtocolFactory {
            all_handler: HashMap::new(),
        }
    }

    pub fn get_handler(
        &mut self,
        a: &RchatCommand,
    ) -> &mut Box<dyn HandlerProtocolData + 'static + Send> {
        match self.all_handler.get_mut(a) {
            None => {
                panic!("Not exist command:{:?}", a);
            }
            Some(t) => t,
        }
    }

    pub fn registry_handler(
        &mut self,
        a: RchatCommand,
        b: Box<dyn HandlerProtocolData + 'static + Send>,
    ) {
        if self.all_handler.get(&a).is_some() {
            panic!("RchatCommand:{:?} already exist! ", a);
        }

        self.all_handler.insert(a, b);
    }
}
