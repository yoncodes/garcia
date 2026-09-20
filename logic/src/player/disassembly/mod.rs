use std::collections::{BTreeMap, HashSet};

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct DisassemblyOutcome {
    pub sold: Vec<DcNetDataSellRow>,
    pub reward: DcNetDataTakeRewardRes,
}

enum DisassemblyTarget {
    Equip(DcNetDataEquip),
    Partner(DcNetDataPartner),
}

impl Player {
    pub fn disassemble(
        &mut self,
        rows: &[DcNetDataSellRow],
        tables: &GameTables,
    ) -> Result<DisassemblyOutcome, DisassemblyError> {
        if rows.is_empty() {
            return Err(DisassemblyError::Empty);
        }

        let mut ids = HashSet::with_capacity(rows.len());
        let mut entities = Vec::with_capacity(rows.len());
        let mut equip_count = 0;
        let mut partner_count = 0;
        for row in rows {
            if !ids.insert(row.user_dat_id) {
                return Err(DisassemblyError::Duplicate(row.user_dat_id));
            }
            if let Some(equip) = self
                .equips
                .iter()
                .find(|equip| equip.user_equip_id == row.user_dat_id)
                .cloned()
            {
                if equip.equip_id != row.item_id {
                    return Err(DisassemblyError::ItemMismatch {
                        user_id: row.user_dat_id,
                        item_id: row.item_id,
                    });
                }
                if equip.locked != 0 {
                    return Err(DisassemblyError::Locked(row.user_dat_id));
                }
                if equip.equiped_role != 0 || equip.in_group != 0 {
                    return Err(DisassemblyError::Equipped(row.user_dat_id));
                }
                equip_count += 1;
                entities.push(DisassemblyTarget::Equip(equip));
                continue;
            }
            if let Some(partner) = self
                .partners
                .iter()
                .find(|partner| partner.id == row.user_dat_id)
                .copied()
            {
                if partner.partner_id != row.item_id {
                    return Err(DisassemblyError::ItemMismatch {
                        user_id: row.user_dat_id,
                        item_id: row.item_id,
                    });
                }
                if partner.locked != 0 {
                    return Err(DisassemblyError::Locked(row.user_dat_id));
                }
                if self.roles.iter().any(|role| {
                    role.role_basic_info
                        .as_ref()
                        .is_some_and(|role| role.user_partner_id == row.user_dat_id)
                }) {
                    return Err(DisassemblyError::Equipped(row.user_dat_id));
                }
                partner_count += 1;
                entities.push(DisassemblyTarget::Partner(partner));
                continue;
            }
            return Err(DisassemblyError::Unknown(row.user_dat_id));
        }
        if equip_count > tables.cultivation_constants.equipment_disassembly_limit {
            return Err(DisassemblyError::TooMany {
                kind: "equipment",
                limit: tables.cultivation_constants.equipment_disassembly_limit,
            });
        }
        if partner_count > tables.cultivation_constants.partner_disassembly_limit {
            return Err(DisassemblyError::TooMany {
                kind: "partners",
                limit: tables.cultivation_constants.partner_disassembly_limit,
            });
        }

        let mut rewards = BTreeMap::<i32, i32>::new();
        let mut equip_exp = 0;
        let mut partner_exp = 0;
        for entity in entities {
            match entity {
                DisassemblyTarget::Equip(equip) => {
                    let config = tables
                        .equipment
                        .get(equip.equip_id)
                        .ok_or(DisassemblyError::MissingConfig(equip.equip_id))?;
                    let para = tables
                        .equipment_parameters
                        .get(config.quality)
                        .ok_or(DisassemblyError::MissingConfig(equip.equip_id))?;
                    let sell = para
                        .sell
                        .first()
                        .ok_or(DisassemblyError::MissingConfig(equip.equip_id))?;
                    *rewards.entry(sell.key).or_default() += sell.value;
                    equip_exp += para.base_exp;
                    equip_exp += tables
                        .equipment_levels
                        .rows
                        .iter()
                        .filter(|level| level.level < equip.level)
                        .map(|level| {
                            scale_refund(
                                level.exp,
                                tables.cultivation_constants.equipment_exp_refund_rate,
                            )
                        })
                        .sum::<i32>();
                    equip_exp += scale_refund(
                        equip.exp,
                        tables.cultivation_constants.equipment_exp_refund_rate,
                    );
                }
                DisassemblyTarget::Partner(partner) => {
                    let config = tables
                        .partners
                        .get(partner.partner_id)
                        .ok_or(DisassemblyError::MissingConfig(partner.partner_id))?;
                    let para = tables
                        .partner_parameters
                        .get(config.quality)
                        .ok_or(DisassemblyError::MissingConfig(partner.partner_id))?;
                    *rewards.entry(para.sell.key).or_default() += para.sell.value;
                    partner_exp += para.base_exp;
                    partner_exp += tables
                        .partner_levels_by_quality
                        .get(config.quality)
                        .into_iter()
                        .flatten()
                        .filter(|level| level.level < partner.lv)
                        .map(|level| {
                            scale_refund(
                                level.required_exp,
                                tables.cultivation_constants.partner_exp_refund_rate,
                            )
                        })
                        .sum::<i32>();
                    partner_exp += scale_refund(
                        partner.exp,
                        tables.cultivation_constants.partner_exp_refund_rate,
                    );
                }
            }
        }
        add_exp_refund_items(
            &mut rewards,
            equip_exp,
            &tables.cultivation_constants.equipment_exp_items,
            tables,
        )?;
        add_exp_refund_items(
            &mut rewards,
            partner_exp,
            &tables.cultivation_constants.partner_exp_items,
            tables,
        )?;

        self.equips
            .retain(|equip| !ids.contains(&equip.user_equip_id));
        self.equipment_word_plans
            .retain(|equip_id, _| !ids.contains(equip_id));
        self.partners.retain(|partner| !ids.contains(&partner.id));
        let mut reward = DcNetDataTakeRewardRes::default();
        for (item_id, amount) in rewards {
            if amount > 0 {
                self.add_reward(item_id, amount, tables, 0, &mut reward);
            }
        }
        Ok(DisassemblyOutcome {
            sold: rows.to_vec(),
            reward,
        })
    }
}

fn scale_refund(value: i32, scale: f32) -> i32 {
    (value as f32 * scale) as i32
}

fn add_exp_refund_items(
    rewards: &mut BTreeMap<i32, i32>,
    mut exp: i32,
    item_ids: &[i32],
    tables: &GameTables,
) -> Result<(), DisassemblyError> {
    for &item_id in item_ids.iter().rev() {
        let value = tables
            .items
            .get(item_id)
            .and_then(|item| item.effect.as_ref())
            .map(|effect| effect.value)
            .filter(|&value| value > 0)
            .ok_or(DisassemblyError::MissingConfig(item_id))?;
        let amount = exp / value;
        exp %= value;
        if amount > 0 {
            *rewards.entry(item_id).or_default() += amount;
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests;
