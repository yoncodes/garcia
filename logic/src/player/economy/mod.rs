use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct ShopPurchaseOutcome {
    pub reward: DcNetDataTakeRewardRes,
    pub remains: Vec<DcNetDataItem>,
    pub bought: i32,
}

impl Player {
    pub fn buy_recharge(
        &mut self,
        recharge_id: i32,
        tables: &GameTables,
        now: i32,
    ) -> Result<RechargePurchaseOutcome, RechargePurchaseError> {
        let product = tables
            .recharges
            .get(recharge_id)
            .cloned()
            .ok_or(RechargePurchaseError::Unknown(recharge_id))?;
        if !matches!(product.kind, 1 | 2) {
            return Err(RechargePurchaseError::UnsupportedType {
                id: recharge_id,
                kind: product.kind,
            });
        }

        let existing = self
            .recharge_purchases
            .iter()
            .find(|purchase| purchase.recharge_id == recharge_id)
            .copied();
        let purchase_count =
            existing.map_or(1, |purchase| purchase.purchase_count.saturating_add(1));
        let expires_at = if product.kind == 2 {
            let day = 86_400;
            let extension = tables
                .cultivation_constants
                .monthly_card_days
                .saturating_mul(day);
            let maximum = now.saturating_add(
                tables
                    .cultivation_constants
                    .monthly_card_max_days
                    .saturating_mul(day),
            );
            let current = existing.map_or(now, |purchase| purchase.expires_at.max(now));
            if current >= maximum {
                return Err(RechargePurchaseError::DurationCap);
            }
            current.saturating_add(extension).min(maximum)
        } else {
            0
        };

        let mut reward = DcNetDataTakeRewardRes::default();
        if let Some(goods) = product.goods {
            self.add_reward(goods.key, goods.value, tables, now, &mut reward);
        }
        if product.kind == 1 {
            let bonus = if existing.is_none() {
                product.first_goods
            } else {
                product.other_goods
            };
            if let Some(bonus) = bonus {
                self.add_reward(bonus.key, bonus.value, tables, now, &mut reward);
            }
        }

        let updated = RechargePurchaseState {
            recharge_id,
            purchase_count,
            last_bought_at: now,
            expires_at,
        };
        if let Some(purchase) = self
            .recharge_purchases
            .iter_mut()
            .find(|purchase| purchase.recharge_id == recharge_id)
        {
            *purchase = updated;
        } else {
            self.recharge_purchases.push(updated);
        }
        Ok(RechargePurchaseOutcome {
            reward,
            purchase_count,
        })
    }

    pub fn buy_shop_goods(
        &mut self,
        goods_id: i32,
        amount: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<ShopPurchaseOutcome, ShopPurchaseError> {
        let goods = tables
            .free_shop_goods
            .iter()
            .find(|goods| goods.id == goods_id)
            .cloned()
            .ok_or(ShopPurchaseError::Unknown(goods_id))?;
        let (_, period) = shop_goods_runtime(tables, goods_id, now, zone_offset)
            .ok_or(ShopPurchaseError::Inactive(goods_id))?;
        if amount <= 0 {
            return Err(ShopPurchaseError::InvalidAmount(amount));
        }
        let bought = self.mall_bought(goods_id, period);
        let new_bought = bought.saturating_add(amount);
        if goods.buy_limit > 0 && new_bought > goods.buy_limit {
            return Err(ShopPurchaseError::BuyLimit {
                id: goods_id,
                limit: goods.buy_limit,
            });
        }
        let reward_known = tables.items.get(goods.goods.key).is_some()
            || tables.maids.get(goods.goods.key).is_some()
            || tables.partners.get(goods.goods.key).is_some()
            || tables.equipment.get(goods.goods.key).is_some()
            || tables.collections.get(goods.goods.key).is_some()
            || tables.maid_skins.get(goods.goods.key).is_some()
            || tables.profile_avatars.get(goods.goods.key).is_some()
            || tables.profile_frames.get(goods.goods.key).is_some()
            || tables.profile_cards.get(goods.goods.key).is_some()
            || tables.profile_titles.get(goods.goods.key).is_some();
        if !reward_known {
            return Err(ShopPurchaseError::UnsupportedReward {
                goods_id,
                reward_id: goods.goods.key,
            });
        }

        let mut costs = BTreeMap::new();
        costs.insert(
            goods.price.key,
            goods
                .price
                .value
                .checked_mul(amount)
                .ok_or(ShopPurchaseError::InvalidAmount(amount))?,
        );
        let remains = consume_item_costs(&mut self.items, &costs, |item_id, needed, available| {
            ShopPurchaseError::InsufficientCost {
                item_id,
                needed,
                available,
            }
        })?;
        if let Some(purchase) = self
            .mall_purchases
            .iter_mut()
            .find(|purchase| purchase.goods_id == goods_id)
        {
            purchase.bought = new_bought;
            purchase.period = period;
        } else {
            self.mall_purchases.push(MallPurchaseState {
                goods_id,
                bought: new_bought,
                period,
            });
        }
        let mut reward = DcNetDataTakeRewardRes::default();
        self.add_reward(
            goods.goods.key,
            goods.goods.value.saturating_mul(amount),
            tables,
            now,
            &mut reward,
        );
        Ok(ShopPurchaseOutcome {
            reward,
            remains,
            bought: new_bought,
        })
    }

    pub fn claim_reward_box(
        &mut self,
        id: &str,
        tables: &GameTables,
        now: i32,
    ) -> Result<DcNetDataTakeRewardRes, RewardBoxError> {
        let drop_id = tables
            .reward_boxes
            .get(id)
            .map(|reward_box| reward_box.drop_id)
            .or_else(|| {
                tables
                    .breakable_objects
                    .get(id)
                    .map(|object| object.drop_id)
            })
            .ok_or_else(|| RewardBoxError::Unknown(id.to_owned()))?;
        if self
            .reward_boxes
            .iter()
            .any(|reward_box| reward_box.id == id && reward_box.status == 1)
        {
            return Err(RewardBoxError::AlreadyClaimed(id.to_owned()));
        }
        let rewards = tables
            .rewards
            .get(drop_id)
            .ok_or_else(|| RewardBoxError::MissingReward(id.to_owned()))?
            .reward
            .clone();

        self.reward_boxes.push(DcNetDataTBoxInfo {
            id: id.to_owned(),
            status: 1,
        });
        let mut result = DcNetDataTakeRewardRes::default();
        for reward in rewards {
            self.add_reward(reward.key, reward.value, tables, now, &mut result);
        }
        Ok(result)
    }

    pub fn mall_bought(&self, goods_id: i32, period: i32) -> i32 {
        self.mall_purchases
            .iter()
            .find(|purchase| purchase.goods_id == goods_id && purchase.period == period)
            .map_or(0, |purchase| purchase.bought)
    }

    pub fn buy_mall_goods(
        &mut self,
        goods_id: i32,
        amount: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
    ) -> Result<MallPurchaseOutcome, MallPurchaseError> {
        self.buy_mall_goods_with_override(goods_id, amount, tables, now, zone_offset, None)
    }

    pub fn buy_mall_goods_with_override(
        &mut self,
        goods_id: i32,
        amount: i32,
        tables: &GameTables,
        now: i32,
        zone_offset: i32,
        override_expires_at: Option<i32>,
    ) -> Result<MallPurchaseOutcome, MallPurchaseError> {
        let goods = tables
            .mall_goods
            .iter()
            .find(|goods| goods.id == goods_id)
            .cloned()
            .ok_or(MallPurchaseError::Unknown(goods_id))?;
        let (cd_time, period) = mall_goods_runtime_with_override(
            tables,
            goods_id,
            now,
            zone_offset,
            override_expires_at,
        )
        .ok_or(MallPurchaseError::Inactive(goods_id))?;
        if amount <= 0 {
            return Err(MallPurchaseError::InvalidAmount);
        }
        if goods.buy_single_limit > 0 && amount > goods.buy_single_limit {
            return Err(MallPurchaseError::SingleLimit {
                id: goods_id,
                limit: goods.buy_single_limit,
            });
        }
        let bought = self.mall_bought(goods_id, period);
        if goods.buy_limit > 0 && bought.saturating_add(amount) > goods.buy_limit {
            return Err(MallPurchaseError::BuyLimit {
                id: goods_id,
                limit: goods.buy_limit,
            });
        }
        let rewards = if goods.price_type == 2 {
            tables
                .items
                .get(goods.goods.key)
                .and_then(|item| item.effect.as_ref())
                .and_then(|effect| tables.rewards.get(effect.key))
                .map(|bundle| {
                    bundle
                        .reward
                        .iter()
                        .map(|reward| {
                            (
                                reward.key,
                                reward
                                    .value
                                    .saturating_mul(goods.goods.value)
                                    .saturating_mul(amount),
                            )
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| {
                    vec![(goods.goods.key, goods.goods.value.saturating_mul(amount))]
                })
        } else {
            (0..amount)
                .map(|_| {
                    let reward_id = tables
                        .items
                        .get(goods.goods.key)
                        .and_then(|item| item.effect.as_ref())
                        .filter(|effect| tables.rewards.get(effect.key).is_some())
                        .and_then(|effect| select_reward(effect.key, tables))
                        .unwrap_or(goods.goods.key);
                    (reward_id, goods.goods.value)
                })
                .collect()
        };
        if rewards.is_empty() {
            return Err(MallPurchaseError::MissingReward(goods_id));
        }

        let mut remains = Vec::new();
        if goods.price_type == 1 {
            let price = goods
                .price
                .as_ref()
                .ok_or(MallPurchaseError::MissingReward(goods_id))?;
            let needed = price.value.saturating_mul(amount);
            let available = self
                .items
                .iter()
                .find(|item| item.item_id == price.key)
                .map_or(0, |item| item.amount);
            if available < needed {
                return Err(MallPurchaseError::InsufficientCost {
                    item_id: price.key,
                    needed,
                    available,
                });
            }
            let item = self
                .items
                .iter_mut()
                .find(|item| item.item_id == price.key)
                .expect("mall currency was checked above");
            item.amount -= needed;
            remains.push(*item);
        }

        let new_bought = bought.saturating_add(amount);
        if let Some(purchase) = self
            .mall_purchases
            .iter_mut()
            .find(|purchase| purchase.goods_id == goods_id)
        {
            purchase.bought = new_bought;
            purchase.period = period;
        } else {
            self.mall_purchases.push(MallPurchaseState {
                goods_id,
                bought: new_bought,
                period,
            });
        }
        let mut reward = DcNetDataTakeRewardRes::default();
        for (reward_id, reward_amount) in rewards {
            self.add_reward(reward_id, reward_amount, tables, now, &mut reward);
        }
        Ok(MallPurchaseOutcome {
            reward,
            remains,
            goods: DcNetDataMallGoods {
                id: goods_id,
                cd_time,
                bought: new_bought,
            },
        })
    }
}

pub fn shop_runtime(tables: &GameTables, shop_id: i32, now: i32, zone_offset: i32) -> Option<i32> {
    let shop = tables.shops.get(shop_id)?;
    let end = shop_end(shop, zone_offset);
    if shop.refresh_type != 0 && end.is_none_or(|end| i64::from(now) >= end) {
        return None;
    }
    Some(end.map_or(0, |end| common::time::seconds_until(end, i64::from(now))))
}

pub fn shop_goods_runtime(
    tables: &GameTables,
    goods_id: i32,
    now: i32,
    zone_offset: i32,
) -> Option<(i32, i32)> {
    let goods = tables
        .free_shop_goods
        .iter()
        .find(|goods| goods.id == goods_id)?;
    shop_runtime(tables, goods.belong_shop, now, zone_offset)?;
    let remain_sec = if goods.refresh_type == 0 {
        0
    } else {
        common::time::next_daily_refresh_seconds(
            now,
            zone_offset,
            tables.cultivation_constants.daily_reset_hour,
        )
    };
    Some((remain_sec, expiry_period(now, remain_sec)))
}

pub fn active_table_window(open: &str, close: &str, now: i32, zone_offset: i32) -> bool {
    if !open.is_empty()
        && common::time::table_time_utc(open, zone_offset)
            .is_none_or(|start| i64::from(now) < start)
    {
        return false;
    }
    close.is_empty()
        || common::time::table_time_utc(close, zone_offset).is_some_and(|end| i64::from(now) < end)
}

pub fn mall_group_runtime(
    tables: &GameTables,
    group_id: i32,
    now: i32,
    zone_offset: i32,
) -> Option<(i32, i32)> {
    mall_group_runtime_with_override(tables, group_id, now, zone_offset, None)
}

pub fn mall_group_runtime_with_override(
    tables: &GameTables,
    group_id: i32,
    now: i32,
    zone_offset: i32,
    override_expires_at: Option<i32>,
) -> Option<(i32, i32)> {
    let group = tables
        .mall_goods_groups
        .iter()
        .find(|group| group.goods_group_id == group_id)?;
    let mall = tables.malls.get(group.mall_id)?;
    if let Some(expires_at) = override_expires_at.filter(|expires_at| *expires_at > now) {
        let cd_time = expires_at - now;
        return Some((cd_time, expires_at));
    }
    if !active_table_window(&mall.mall_open, &mall.mall_close, now, zone_offset)
        || !active_table_window(
            &group.goods_group_open,
            &group.goods_group_close,
            now,
            zone_offset,
        )
    {
        return None;
    }
    let refresh_hour = tables.cultivation_constants.daily_reset_hour;
    let cd_time = if group.goods_group_close.is_empty() {
        match group.refresh_type {
            2 | 3 => common::time::next_daily_refresh_seconds(now, zone_offset, refresh_hour),
            4 => common::time::next_month_refresh_seconds(now, zone_offset, refresh_hour),
            _ => 0,
        }
    } else {
        common::time::table_time_utc(&group.goods_group_close, zone_offset)
            .map_or(0, |end| common::time::seconds_until(end, i64::from(now)))
    };
    Some((cd_time, expiry_period(now, cd_time)))
}

pub fn mall_goods_runtime(
    tables: &GameTables,
    goods_id: i32,
    now: i32,
    zone_offset: i32,
) -> Option<(i32, i32)> {
    mall_goods_runtime_with_override(tables, goods_id, now, zone_offset, None)
}

pub fn mall_goods_runtime_with_override(
    tables: &GameTables,
    goods_id: i32,
    now: i32,
    zone_offset: i32,
    override_expires_at: Option<i32>,
) -> Option<(i32, i32)> {
    let goods = tables
        .mall_goods
        .iter()
        .find(|goods| goods.id == goods_id)?;
    mall_group_runtime_with_override(
        tables,
        goods.goods_group_id,
        now,
        zone_offset,
        override_expires_at,
    )
}

fn shop_end(shop: &configs::tables::Shop, zone_offset: i32) -> Option<i64> {
    let (start, duration) = shop.refresh_time.split_once(',')?;
    common::time::table_time_utc(start, zone_offset)?.checked_add(duration.parse::<i64>().ok()?)
}

fn expiry_period(now: i32, remaining: i32) -> i32 {
    if remaining == 0 {
        0
    } else {
        now.saturating_add(remaining)
    }
}
#[cfg(test)]
mod tests;
