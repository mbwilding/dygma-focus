use super::Focus;
use anyhow::Result;

pub fn move_to(focus: &mut Focus, layer: u8) -> Result<()> {
    focus.command(&format!("layer.moveTo {}", layer - 1))
}
