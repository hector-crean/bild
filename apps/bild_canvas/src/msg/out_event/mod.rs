use bevy::prelude::*;
use ts_rs::TS;

use crate::view::layout_3d::tool::ToolState;



#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Event, Message, TS)]
#[serde(tag = "type", content = "data")]
pub enum BildOutMsg {
    ToolChanged(ToolState),
}

impl BildOutMsg {
    pub fn handle(mut event_rdr: MessageReader<BildOutMsg>) {
        for ev in event_rdr.read(){
            info!("event: {:?}", ev);
            match ev {
                BildOutMsg::ToolChanged(tool_state) => {
                    // let _  = GLOBAL_EVENT_CHANNEL.send(ev.clone());
                }
            }
        }
    }
}



