use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RiftBuff {
    pub id: i32,
    pub group: i32,
    pub unlock_limit: i32,
    #[serde(default)]
    pub buff_cost1: i32,
    #[serde(default)]
    pub buff_cost2: i32,
    #[serde(default)]
    pub buff_cost3: i32,
}

impl RiftBuff {
    pub fn buff(&self, index: i32) -> Option<i32> {
        match index {
            1 if self.buff_cost1 != 0 => Some(self.buff_cost1),
            2 if self.buff_cost2 != 0 => Some(self.buff_cost2),
            3 if self.buff_cost3 != 0 => Some(self.buff_cost3),
            _ => None,
        }
    }
}
