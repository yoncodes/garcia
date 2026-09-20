use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeatExchangeOutcome {
    pub cost_remain: DcNetDataItem,
    pub heat_remain: DcNetDataItem,
    pub exchange_count: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeatInfoOutcome {
    pub item: DcNetDataItem,
    pub cd_time: i32,
}

impl Player {
    pub fn refresh_heat(
        &mut self,
        tables: &GameTables,
        now: i32,
    ) -> Result<HeatInfoOutcome, HeatExchangeError> {
        let interval = tables.cultivation_constants.heat_regeneration_interval;
        let maximum = tables
            .player_levels
            .get(self.level)
            .map(|level| level.max_stamina)
            .filter(|maximum| *maximum > 0)
            .ok_or(HeatExchangeError::MissingConfig)?;
        if interval <= 0 {
            return Err(HeatExchangeError::MissingConfig);
        }
        let index = self
            .items
            .iter()
            .position(|item| item.item_id == tables.cultivation_constants.heat_item_id)
            .ok_or(HeatExchangeError::MissingConfig)?;

        if self.items[index].amount < maximum {
            let elapsed = now.saturating_sub(self.heat_updated_at);
            let restored = elapsed / interval;
            if restored > 0 {
                self.items[index].amount = self.items[index]
                    .amount
                    .saturating_add(restored)
                    .min(maximum);
                self.heat_updated_at = self
                    .heat_updated_at
                    .saturating_add(restored.saturating_mul(interval));
            }
        }
        let cd_time = if self.items[index].amount >= maximum {
            0
        } else {
            interval
                - now
                    .saturating_sub(self.heat_updated_at)
                    .clamp(0, interval - 1)
        };
        Ok(HeatInfoOutcome {
            item: self.items[index],
            cd_time,
        })
    }

    pub fn exchange_diamonds_for_stamps(
        &mut self,
        amount: i32,
        tables: &GameTables,
    ) -> Result<Vec<DcNetDataItem>, CurrencyExchangeError> {
        if amount <= 0 {
            return Err(CurrencyExchangeError::InvalidAmount);
        }
        let price = tables.cultivation_constants.diamonds_per_stamp;
        let needed = price
            .checked_mul(amount)
            .filter(|needed| *needed > 0)
            .ok_or(CurrencyExchangeError::MissingConfig)?;
        let costs = [(tables.cultivation_constants.diamond_item_id, needed)]
            .into_iter()
            .collect();
        let mut changed = consume_item_costs(&mut self.items, &costs, |_, needed, available| {
            CurrencyExchangeError::InsufficientDiamonds { needed, available }
        })?;
        let stamp = self
            .add_item(tables.cultivation_constants.stamp_item_id, amount, tables)
            .ok_or(CurrencyExchangeError::MissingConfig)?;
        changed.push(stamp);
        Ok(changed)
    }

    pub fn heat_exchange_count(&self, now: i32) -> i32 {
        if self.heat_exchange_day == common::time::day_index(now) {
            self.heat_exchange_count
        } else {
            0
        }
    }

    pub fn exchange_heat(
        &mut self,
        amount: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<HeatExchangeOutcome, HeatExchangeError> {
        if amount != 1 {
            return Err(HeatExchangeError::InvalidAmount(amount));
        }
        let count = self.heat_exchange_count(now);
        if count >= tables.cultivation_constants.daily_heat_exchange_limit {
            return Err(HeatExchangeError::DailyLimit);
        }
        let point = tables
            .cultivation_constants
            .heat_restore_amounts
            .first()
            .map(|point| point.value)
            .filter(|point| *point > 0)
            .ok_or(HeatExchangeError::MissingConfig)?;
        let current_heat = self
            .items
            .iter()
            .find(|item| item.item_id == tables.cultivation_constants.heat_item_id)
            .map_or(0, |item| item.amount);
        if current_heat.saturating_add(point) > tables.cultivation_constants.heat_limit {
            return Err(HeatExchangeError::HeatFull);
        }
        let prices = &tables.cultivation_constants.heat_restore_costs;
        let price = prices
            .iter()
            .take(prices.len().saturating_sub(1))
            .find(|price| price.key >= count)
            .or_else(|| prices.last())
            .map(|price| price.value)
            .filter(|price| *price > 0)
            .ok_or(HeatExchangeError::MissingConfig)?;
        let costs = [(tables.cultivation_constants.diamond_item_id, price)]
            .into_iter()
            .collect();
        let cost_remain =
            consume_item_costs(&mut self.items, &costs, |item_id, needed, available| {
                HeatExchangeError::InsufficientCost {
                    item_id,
                    needed,
                    available,
                }
            })?
            .into_iter()
            .next()
            .ok_or(HeatExchangeError::MissingConfig)?;
        let heat_remain = self
            .add_item(tables.cultivation_constants.heat_item_id, point, tables)
            .ok_or(HeatExchangeError::MissingConfig)?;
        self.heat_exchange_day = common::time::day_index(now);
        self.heat_exchange_count = count + 1;
        Ok(HeatExchangeOutcome {
            cost_remain,
            heat_remain,
            exchange_count: count + 1,
        })
    }
}
#[cfg(test)]
mod tests;
