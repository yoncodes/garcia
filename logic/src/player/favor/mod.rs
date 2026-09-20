use super::*;

// TableManager.InitIdDefine uses this subtype to build the client's favor-gift list.
const FAVOR_GIFT_SUBTYPE: i32 = 22;

impl Player {
    pub fn favor_touches(&self, tables: &GameTables, now: i32, zone_offset: i32) -> i32 {
        let day = common::time::daily_refresh_day(
            now,
            zone_offset,
            tables.cultivation_constants.daily_reset_hour,
        );
        if self.favor_day == day {
            self.favor_touches
        } else {
            0
        }
    }

    pub fn touch_favor(
        &mut self,
        character_id: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<(i32, DcNetDataFavor), FavorError> {
        validate_character(character_id, tables)?;
        let day = common::time::daily_refresh_day(
            now,
            zone_offset,
            tables.cultivation_constants.daily_reset_hour,
        );
        if self.favor_day != day {
            self.favor_day = day;
            self.favor_touches = 0;
        }
        if self.favor_touches >= tables.cultivation_constants.daily_favor_touch_limit {
            return Err(FavorError::DailyLimit);
        }

        self.favor_touches += 1;
        let info = apply_favor_exp(
            &mut self.favors,
            character_id,
            tables.cultivation_constants.favor_per_touch,
            tables,
        );
        Ok((self.favor_touches, info))
    }

    pub fn gift_favor(
        &mut self,
        character_id: i32,
        materials: &[DcNetDataUseItem],
        tables: &GameTables,
    ) -> Result<(DcNetDataFavor, Vec<DcNetDataItem>), FavorError> {
        validate_character(character_id, tables)?;
        let max_level = max_favor_level(tables);
        if self
            .favors
            .iter()
            .any(|favor| favor.id == character_id && favor.lv >= max_level)
        {
            return Err(FavorError::MaxLevel(character_id));
        }

        let mut costs = BTreeMap::new();
        let mut gained_exp = 0_i32;
        for material in materials {
            let effect = tables
                .items
                .get(material.item_id)
                .filter(|item| item.sub_type == FAVOR_GIFT_SUBTYPE)
                .and_then(|item| item.effect.as_ref())
                .ok_or(FavorError::InvalidMaterial(material.item_id))?;
            if material.amount <= 0 {
                return Err(FavorError::InvalidMaterial(material.item_id));
            }
            *costs.entry(material.item_id).or_default() += material.amount;
            gained_exp = gained_exp.saturating_add(effect.value.saturating_mul(material.amount));
        }
        if costs.is_empty() {
            return Err(FavorError::InvalidMaterial(0));
        }

        let remains = consume_item_costs(&mut self.items, &costs, |item_id, needed, available| {
            FavorError::InsufficientItem {
                item_id,
                needed,
                available,
            }
        })?;
        let info = apply_favor_exp(&mut self.favors, character_id, gained_exp, tables);
        Ok((info, remains))
    }
}

fn validate_character(character_id: i32, tables: &GameTables) -> Result<(), FavorError> {
    if tables
        .character_favor_by_character
        .get(character_id)
        .is_some()
    {
        Ok(())
    } else {
        Err(FavorError::UnknownCharacter(character_id))
    }
}

fn max_favor_level(tables: &GameTables) -> i32 {
    tables
        .character_favor_parameters
        .rows
        .last()
        .map_or(1, |param| param.id)
}

fn apply_favor_exp(
    favors: &mut Vec<DcNetDataFavor>,
    character_id: i32,
    gained_exp: i32,
    tables: &GameTables,
) -> DcNetDataFavor {
    let index = favors
        .iter()
        .position(|favor| favor.id == character_id)
        .unwrap_or_else(|| {
            favors.push(DcNetDataFavor {
                id: character_id,
                exp: 0,
                lv: 1,
            });
            favors.len() - 1
        });
    let favor = &mut favors[index];
    let maximum = max_favor_level(tables);
    favor.exp = favor.exp.saturating_add(gained_exp);
    while favor.lv < maximum {
        let Some(required) = tables
            .character_favor_parameters
            .get(favor.lv)
            .map(|param| param.exp)
        else {
            break;
        };
        if required <= 0 || favor.exp < required {
            break;
        }
        favor.exp -= required;
        favor.lv += 1;
    }
    if favor.lv >= maximum {
        favor.lv = maximum;
        favor.exp = 0;
    }
    *favor
}
#[cfg(test)]
mod tests;
