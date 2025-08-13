use bevy::prelude::*;
use ts_rs::TS;

use crate::view::layout_3d::tool::ToolState;



#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Event, BufferedEvent, TS)]
#[serde(tag = "type", content = "data")]
pub enum BildInEvent {
    ChangeTool(ToolState),
    ExitApp,
}




impl BildInEvent {
    pub fn handle(mut event_rdr: EventReader<BildInEvent>, mut next_tool_state: ResMut<NextState<ToolState>>, mut app_exit: EventWriter<AppExit>) {
        for ev in event_rdr.read(){
            info!("event: {:?}", ev);
            match ev {
                BildInEvent::ChangeTool(tool_label) => {
                    next_tool_state.set( *tool_label);
                }
                BildInEvent::ExitApp => {
                    info!("AppExit event received:");
                    app_exit.write(AppExit::Success);
                }
               
            }
        }
    }
}






