use crate::chat_protocol::{ChatCommand};
use std::collections::HashMap;
use async_trait::async_trait;

#[async_trait]
pub trait HandlerProtocolData {
    async fn handle(&self, a: &Vec<u8>);
}


pub struct HandleProtocolFactory {
    pub all_handler: HashMap<ChatCommand, Box<dyn HandlerProtocolData>>,
}

impl HandleProtocolFactory {
    pub fn new()->Self{
        HandleProtocolFactory{
            all_handler: HashMap::new()
        }
    }



    pub fn get_handler(&self, a: &ChatCommand) -> &Box<dyn HandlerProtocolData> {
        match self.all_handler.get(a) {
            None => {
                panic!("Not exist command:{:?}", a);
            }
            Some(t) => t,
        }
    }

    pub fn registry_handler(&mut self, a:ChatCommand , b: Box<dyn HandlerProtocolData> ){

        if self.all_handler.get(&a).is_some(){
            panic!("ChatCommand:{:?} already exist! ",a);
        }

        self.all_handler.insert(a,b);
    }

}
