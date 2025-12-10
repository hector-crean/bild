use bevy::prelude::*;
use ts_rs::TS;

use crate::view::layout_3d::tool::ToolState;



#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Event, Message, TS)]
#[serde(tag = "type", content = "data")]
pub enum BildInMsg {
    ChangeTool(ToolState),
    ExitApp,
}




impl BildInMsg {
    pub fn handle(mut event_rdr: MessageReader<BildInMsg>, mut next_tool_state: ResMut<NextState<ToolState>>, mut app_exit: MessageWriter<AppExit>) {
        for ev in event_rdr.read(){
            info!("event: {:?}", ev);
            match ev {
                BildInMsg::ChangeTool(tool_label) => {
                    next_tool_state.set( *tool_label);
                }
                BildInMsg::ExitApp => {
                    info!("AppExit event received:");
                    app_exit.write(AppExit::Success);
                }
               
            }
        }
    }
}






