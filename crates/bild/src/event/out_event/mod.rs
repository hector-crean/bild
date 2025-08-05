use bevy::prelude::*;
use ts_rs::TS;

use crate::tool::ToolState;


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Event, BufferedEvent, TS)]
#[serde(tag = "type", content = "data")]
pub enum BildOutEvent {
    ToolChanged(ToolState),
}

impl BildOutEvent {
    pub fn handle(mut event_rdr: EventReader<BildOutEvent>) {
        for ev in event_rdr.read(){
            info!("event: {:?}", ev);
            match ev {
                BildOutEvent::ToolChanged(tool_state) => {
                    // let _  = GLOBAL_EVENT_CHANNEL.send(ev.clone());
                }
            }
        }
    }
}



