use protocol::{
    cs::{
        DcNetWorkingNotifyPortsInfo, DcNetWorkingNotifyProfileNew,
        DcNetWorkingNotifyRedeemGiftCode, DcNetWorkingNotifyWorldLevel, DcNetWorkingParamLocalSet,
    },
    pbcommon::{
        DcNetDataLocal, DcNetDataPartnerReward, DcNetDataRoleReward, DcNetDataTakeRewardRes,
        Reddot, TaskStatus,
    },
    proids::NotifyId,
};
use serde::Serialize;

use super::GatewaySession;
use crate::net::{
    context::HandlerContext,
    error::{NetworkError, NetworkResult},
    gm_command::{GmCommand, GmOutcome},
};

#[derive(Serialize)]
struct CollectionCatalog {
    heroes: Vec<CollectionEntry>,
    pets: Vec<CollectionEntry>,
    relics: Vec<RelicCatalogEntry>,
    items: Vec<ItemCatalogEntry>,
    progression: ProgressionCatalog,
}

#[derive(Serialize)]
struct ProgressionCatalog {
    player_level: i32,
    player_levels: Vec<i32>,
    world_level: i32,
    world_levels: Vec<i32>,
    story_stages: Vec<StoryStageEntry>,
}

#[derive(Serialize)]
struct StoryStageEntry {
    gameplay_id: i32,
    task_group: i32,
    sort: i32,
    completed: bool,
}

#[derive(Serialize)]
struct CollectionEntry {
    id: i32,
    name: String,
    owned: bool,
}

#[derive(Serialize)]
struct ItemCatalogEntry {
    id: i32,
    name: String,
    amount: i32,
}

#[derive(Serialize)]
struct RelicCatalogEntry {
    id: i32,
    name: String,
    amount: i32,
    main_stats: Vec<RelicStatEntry>,
    substats: Vec<RelicStatEntry>,
    substat_count: i32,
}

#[derive(Serialize)]
struct RelicStatEntry {
    id: i32,
    effect_id: i32,
    name: &'static str,
    value: f32,
    limit: i32,
}

#[derive(Serialize)]
struct TeleportCatalog {
    current: Option<CurrentLocation>,
    missions: Vec<MissionLocation>,
    gates: Vec<TeleportLocation>,
}

#[derive(Serialize)]
struct CurrentLocation {
    location_id: String,
    city_id: Option<i32>,
    region: i32,
    kind: &'static str,
    name: String,
}

#[derive(Clone, Serialize)]
struct TeleportLocation {
    anchor_id: String,
    city_id: i32,
    region: i32,
    name: String,
    visible: bool,
    unlocked: bool,
}

#[derive(Serialize)]
struct MissionLocation {
    task_id: i32,
    task_group: i32,
    city_id: i32,
    region: i32,
    target_kind: &'static str,
    target: String,
    current: bool,
    teleportable: bool,
}

fn teleport_name(anchor_id: &str) -> String {
    anchor_id
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(chars).collect()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn relic_stat_name(id: i32) -> &'static str {
    match id {
        1 => "HP",
        2 => "Attack",
        3 => "Defense",
        4 => "Critical Rate",
        5 => "Critical Damage",
        6 => "HP %",
        7 => "Attack %",
        8 => "Defense %",
        9 => "Wind Damage",
        10 => "Thunder Damage",
        11 => "Ice Damage",
        12 => "Fire Damage",
        13 => "Dark Damage",
        18 => "Healing",
        25 => "Ultimate Charge",
        40 => "Elemental Resonance",
        _ => "Unknown attribute",
    }
}

fn push_reward_notification(
    ctx: &mut HandlerContext,
    reward: DcNetDataTakeRewardRes,
) -> NetworkResult<()> {
    ctx.push(
        NotifyId::DcNetWorkingNotifyRedeemGiftCode as u16,
        DcNetWorkingNotifyRedeemGiftCode {
            reward: Some(reward),
        },
    )
}

impl GatewaySession {
    pub async fn handle_gm_command(
        &mut self,
        command: GmCommand,
    ) -> NetworkResult<(GmOutcome, Vec<Vec<u8>>)> {
        let state = self.context.state.clone();
        let tables = &state.tables;
        let now = common::time::ServerTime::now_seconds_i32();
        let outcome = match command {
            GmCommand::Collection => {
                let player = self.context.player()?;
                let mut heroes = tables
                    .maids
                    .rows
                    .iter()
                    .filter(|hero| tables.is_playable_maid(hero.id))
                    .map(|hero| CollectionEntry {
                        id: hero.id,
                        name: if hero.english_name.is_empty() {
                            hero.name.clone()
                        } else {
                            hero.english_name.clone()
                        },
                        owned: player.owns_role(hero.id),
                    })
                    .collect::<Vec<_>>();
                let mut pets = tables
                    .partners
                    .rows
                    .iter()
                    .map(|pet| CollectionEntry {
                        id: pet.id,
                        name: if pet.english_name.is_empty() {
                            pet.name.clone()
                        } else {
                            pet.english_name.clone()
                        },
                        owned: player.owns_partner(pet.id),
                    })
                    .collect::<Vec<_>>();
                let mut items = tables
                    .items
                    .rows
                    .iter()
                    .map(|item| ItemCatalogEntry {
                        id: item.id,
                        name: if item.english_name.is_empty() {
                            format!("Item {}", item.id)
                        } else {
                            item.english_name.clone()
                        },
                        amount: player
                            .items
                            .iter()
                            .find(|owned| owned.item_id == item.id)
                            .map_or(0, |owned| owned.amount),
                    })
                    .collect::<Vec<_>>();
                let mut relics = tables
                    .equipment
                    .rows
                    .iter()
                    // Unlocalized relic rows are unfinished client data: they have no
                    // obtainable reward and their suit bonuses use placeholder talents.
                    .filter(|relic| !relic.english_name.is_empty())
                    .map(|relic| {
                        let main_stats = tables
                            .main_equipment_words_by_group
                            .get(relic.group_id)
                            .into_iter()
                            .flatten()
                            .map(|word| RelicStatEntry {
                                id: word.id,
                                effect_id: word.effect.key,
                                name: relic_stat_name(word.effect.key),
                                value: word.effect.value,
                                limit: 1,
                            })
                            .collect();
                        let substats = tables
                            .extra_equipment_words_by_slot
                            .get(relic.slot)
                            .into_iter()
                            .flatten()
                            .map(|word| RelicStatEntry {
                                id: word.id,
                                effect_id: word.effect.key,
                                name: relic_stat_name(word.effect.key),
                                value: word.effect.value,
                                limit: word.duplicate_limit,
                            })
                            .collect();
                        RelicCatalogEntry {
                            id: relic.id,
                            name: format!(
                                "{} · {} · Slot {}",
                                if relic.english_name.is_empty() {
                                    format!("Relic {}", relic.id)
                                } else {
                                    relic.english_name.clone()
                                },
                                match relic.quality {
                                    4 => "Blue",
                                    5 => "Purple",
                                    6 => "Gold",
                                    _ => "Unknown rarity",
                                },
                                relic.slot
                            ),
                            amount: player
                                .equips
                                .iter()
                                .filter(|owned| owned.equip_id == relic.id)
                                .count() as i32,
                            main_stats,
                            substats,
                            substat_count: tables
                                .equipment_parameters
                                .get(relic.quality)
                                .map_or(0, |parameters| parameters.max_word_rolls()),
                        }
                    })
                    .collect::<Vec<_>>();
                heroes.sort_unstable_by(|left, right| {
                    left.name.cmp(&right.name).then(left.id.cmp(&right.id))
                });
                pets.sort_unstable_by(|left, right| {
                    left.name.cmp(&right.name).then(left.id.cmp(&right.id))
                });
                items.sort_unstable_by_key(|item| item.id);
                relics.sort_unstable_by_key(|relic| relic.id);
                let mut story_stages = tables
                    .tasks
                    .rows
                    .iter()
                    .filter(|task| task.task_type == 1)
                    .filter_map(|task| {
                        let gameplay_id = task
                            .done_key
                            .iter()
                            .find(|condition| condition.key == 39)?
                            .value
                            .split('|')
                            .next()?
                            .parse::<i32>()
                            .ok()?;
                        Some(StoryStageEntry {
                            gameplay_id,
                            task_group: task.task_group,
                            sort: task.sort,
                            completed: player
                                .ports
                                .iter()
                                .any(|port| port.id == gameplay_id && port.pass_cnt > 0),
                        })
                    })
                    .collect::<Vec<_>>();
                story_stages.sort_unstable_by_key(|stage| (stage.task_group, stage.sort));
                GmOutcome {
                    message: "player collection loaded".into(),
                    data: Some(
                        serde_json::to_value(CollectionCatalog {
                            heroes,
                            pets,
                            relics,
                            items,
                            progression: ProgressionCatalog {
                                player_level: player.level,
                                player_levels: tables
                                    .player_levels
                                    .rows
                                    .iter()
                                    .map(|row| row.level)
                                    .collect(),
                                world_level: player.world_level(tables),
                                world_levels: tables
                                    .world_levels
                                    .iter()
                                    .map(|row| row.level)
                                    .collect(),
                                story_stages,
                            },
                        })
                        .expect("collection catalog is serializable"),
                    ),
                    ..Default::default()
                }
            }
            GmCommand::TeleportLocations => {
                let player = self.context.player()?;
                let mut gates = tables
                    .teleportation_anchors
                    .rows
                    .iter()
                    .filter_map(|anchor| {
                        let stage = tables.city_stages.get(anchor.city_id)?;
                        Some(TeleportLocation {
                            anchor_id: anchor.id.clone(),
                            city_id: anchor.city_id,
                            region: stage.region,
                            name: teleport_name(&anchor.id),
                            visible: anchor.is_visible != 0,
                            unlocked: anchor.is_touch == 0
                                || player.interact_objs.iter().any(|object| {
                                    object.object_id == anchor.id && object.status > 0
                                }),
                        })
                    })
                    .collect::<Vec<_>>();
                gates.sort_unstable_by(|left, right| {
                    (left.region, left.city_id, &left.name).cmp(&(
                        right.region,
                        right.city_id,
                        &right.name,
                    ))
                });
                let current_location_id = player
                    .locals
                    .iter()
                    .find(|local| local.region == player.region)
                    .map(|local| {
                        if local.local2.is_empty() {
                            local.local.as_str()
                        } else {
                            local.local2.as_str()
                        }
                    });
                let current = current_location_id.map(|location_id| {
                    if let Some(gate) = gates
                        .iter()
                        .find(|location| location.anchor_id == location_id)
                    {
                        CurrentLocation {
                            location_id: location_id.to_owned(),
                            city_id: Some(gate.city_id),
                            region: gate.region,
                            kind: "teleport gate",
                            name: gate.name.clone(),
                        }
                    } else if let Some(task) = tables.tasks.rows.iter().find(|task| {
                        task.position
                            .as_ref()
                            .is_some_and(|position| position.value == location_id)
                    }) {
                        CurrentLocation {
                            location_id: location_id.to_owned(),
                            city_id: Some(task.city_id),
                            region: player.region,
                            kind: "mission point",
                            name: teleport_name(location_id),
                        }
                    } else {
                        CurrentLocation {
                            location_id: location_id.to_owned(),
                            city_id: None,
                            region: player.region,
                            kind: "scene point",
                            name: teleport_name(location_id),
                        }
                    }
                });
                let current_tasks = player.current_traced_task_ids(tables);
                let mut missions = player
                    .tasks
                    .iter()
                    .filter(|task| task.status == TaskStatus::Picked as i32)
                    .filter_map(|state| {
                        let task = tables.tasks.get(state.id)?;
                        let position = task.position.as_ref()?;
                        let stage = tables.city_stages.get(task.city_id)?;
                        Some(MissionLocation {
                            task_id: task.id,
                            task_group: task.task_group,
                            city_id: task.city_id,
                            region: stage.region,
                            target_kind: match position.key {
                                2 => "coordinates",
                                3 => "city",
                                4 => "scene point",
                                _ => "unknown",
                            },
                            target: position.value.clone(),
                            current: current_tasks.contains(&task.id),
                            teleportable: position.key == 4,
                        })
                    })
                    .collect::<Vec<_>>();
                missions.sort_unstable_by_key(|mission| (mission.task_group, mission.task_id));
                GmOutcome {
                    message: format!(
                        "loaded {} teleport locations and {} active mission destinations",
                        gates.len(),
                        missions.len()
                    ),
                    data: Some(
                        serde_json::to_value(TeleportCatalog {
                            current,
                            missions,
                            gates,
                        })
                        .expect("teleport catalog is serializable"),
                    ),
                    ..Default::default()
                }
            }
            GmCommand::Teleport(anchor_id) => {
                let anchor = tables
                    .teleportation_anchors
                    .get(&anchor_id)
                    .ok_or_else(|| {
                        NetworkError::InvalidFeature(format!(
                            "unknown teleport location {anchor_id}"
                        ))
                    })?;
                let stage = tables.city_stages.get(anchor.city_id).ok_or_else(|| {
                    NetworkError::InvalidFeature(format!(
                        "teleport location {anchor_id} has no city stage"
                    ))
                })?;
                let changed = self.context.update_player()?.set_local(DcNetDataLocal {
                    region: stage.region,
                    local: anchor_id.clone(),
                    local2: String::new(),
                });
                self.context.push(
                    60_000,
                    DcNetWorkingParamLocalSet {
                        region: anchor.city_id,
                        local: anchor_id.clone(),
                        local2: String::new(),
                    },
                )?;
                GmOutcome {
                    message: format!(
                        "teleporting to {} ({})",
                        teleport_name(&anchor_id),
                        anchor.city_id
                    ),
                    changed: usize::from(changed),
                    ..Default::default()
                }
            }
            GmCommand::TeleportMission(task_id) => {
                let player = self.context.player()?;
                if !player
                    .tasks
                    .iter()
                    .any(|task| task.id == task_id && task.status == TaskStatus::Picked as i32)
                {
                    return Err(NetworkError::InvalidFeature(format!(
                        "mission {task_id} is not active"
                    )));
                }
                let task = tables.tasks.get(task_id).ok_or_else(|| {
                    NetworkError::InvalidFeature(format!("unknown mission {task_id}"))
                })?;
                let position = task
                    .position
                    .as_ref()
                    .filter(|position| position.key == 4)
                    .ok_or_else(|| {
                        NetworkError::InvalidFeature(format!(
                            "mission {task_id} has no exact scene teleport point"
                        ))
                    })?;
                let stage = tables.city_stages.get(task.city_id).ok_or_else(|| {
                    NetworkError::InvalidFeature(format!("mission {task_id} has no city stage"))
                })?;
                let target = position.value.clone();
                let changed = self.context.update_player()?.set_local(DcNetDataLocal {
                    region: stage.region,
                    local: target.clone(),
                    local2: String::new(),
                });
                self.context.push(
                    60_000,
                    DcNetWorkingParamLocalSet {
                        region: task.city_id,
                        local: target.clone(),
                        local2: String::new(),
                    },
                )?;
                GmOutcome {
                    message: format!("teleporting to mission {task_id} at {target}"),
                    changed: usize::from(changed),
                    ..Default::default()
                }
            }
            GmCommand::UnlockTeleportGates => {
                let updates = self
                    .context
                    .update_player()?
                    .unlock_all_teleport_gates(tables);
                for object in &updates {
                    self.context.push(
                        NotifyId::DcNetWorkingNotifyInteractObjs as u16,
                        protocol::cs::DcNetWorkingNotifyInteractObjs {
                            obj: Some(object.clone()),
                        },
                    )?;
                }
                GmOutcome {
                    message: format!("unlocked {} teleport gates", updates.len()),
                    changed: updates.len(),
                    ..Default::default()
                }
            }
            GmCommand::CompleteTutorials => {
                let changed = self.context.update_player()?.complete_all_tutorials(tables);
                if changed > 0 {
                    crate::handlers::guide::push_list(&mut self.context)?;
                }
                GmOutcome {
                    message: format!("completed {changed} tutorial groups"),
                    changed,
                    reconnect_required: false,
                    ..Default::default()
                }
            }
            GmCommand::ResetTutorials => {
                let changed = self.context.update_player()?.reset_tutorials(tables);
                if changed > 0 {
                    crate::handlers::guide::push_list(&mut self.context)?;
                }
                GmOutcome {
                    message: format!("reset {changed} tutorial groups"),
                    changed,
                    reconnect_required: false,
                    ..Default::default()
                }
            }
            GmCommand::UnlockFeatures => {
                let outcome = self
                    .context
                    .update_player()?
                    .unlock_all_features(tables, now)
                    .map_err(|error| NetworkError::InvalidFeature(error.to_string()))?;
                push_progression_tasks(
                    &mut self.context,
                    &outcome.requirements,
                    &outcome.completions,
                    now,
                )?;
                push_progression_features(
                    &mut self.context,
                    &outcome.features,
                    &outcome.completions,
                )?;
                if !outcome.ports.is_empty() {
                    self.context.push(
                        NotifyId::DcNetWorkingNotifyPortsInfo as u16,
                        DcNetWorkingNotifyPortsInfo {
                            ports: outcome.ports.clone(),
                        },
                    )?;
                }
                if let Some(update) = outcome.level_up_data.clone() {
                    self.context
                        .push(NotifyId::DcNetDataAttrUpData as u16, update)?;
                }
                let changed = outcome.requirements.len()
                    + outcome.ports.len()
                    + outcome.features.len()
                    + usize::from(outcome.level_up_data.is_some());
                GmOutcome {
                    message: format!(
                        "completed {} feature requirements and unlocked {} features",
                        outcome.requirements.len(),
                        outcome.features.len()
                    ),
                    changed,
                    reconnect_required: false,
                    ..Default::default()
                }
            }
            GmCommand::GrantHeroes => {
                let rewards = self.context.update_player()?.grant_all_roles(tables, now);
                let changed = rewards.role_rewards.len();
                if changed > 0 {
                    push_reward_notification(&mut self.context, rewards)?;
                }
                GmOutcome {
                    message: format!("granted {changed} heroes"),
                    changed,
                    reconnect_required: false,
                    ..Default::default()
                }
            }
            GmCommand::GrantPets => {
                let rewards = self
                    .context
                    .update_player()?
                    .grant_all_partners(tables, now);
                let changed = rewards.partner_rewards.len();
                if changed > 0 {
                    push_reward_notification(&mut self.context, rewards)?;
                }
                GmOutcome {
                    message: format!("granted {changed} pets"),
                    changed,
                    reconnect_required: false,
                    ..Default::default()
                }
            }
            GmCommand::GrantProfileFrames => {
                let rewards = self
                    .context
                    .update_player()?
                    .grant_all_profile_frames(tables, now);
                let changed = rewards.profileframe_rewards.len();
                if changed > 0 {
                    push_reward_notification(&mut self.context, rewards)?;
                }
                GmOutcome {
                    message: format!("unlocked {changed} avatar frames"),
                    changed,
                    ..Default::default()
                }
            }
            GmCommand::GrantProfileTitles => {
                let rewards = self
                    .context
                    .update_player()?
                    .grant_all_profile_titles(tables, now);
                let changed = rewards.profiletitle_rewards.len();
                if changed > 0 {
                    push_reward_notification(&mut self.context, rewards)?;
                }
                GmOutcome {
                    message: format!("unlocked {changed} titles"),
                    changed,
                    ..Default::default()
                }
            }
            GmCommand::GrantHero(id) => {
                let known = tables.is_playable_maid(id);
                let already_owned = self.context.player()?.owns_role(id);
                let rewards = self.context.update_player()?.grant_role(id, tables, now);
                let changed = rewards.role_rewards.len();
                if changed > 0 {
                    push_reward_notification(&mut self.context, rewards)?;
                }
                GmOutcome {
                    message: if !known {
                        format!("unknown hero {id}")
                    } else if already_owned {
                        format!("granted duplicate hero {id}")
                    } else {
                        format!("granted hero {id}")
                    },
                    changed,
                    ..Default::default()
                }
            }
            GmCommand::GrantPet(id, amount) => {
                let known = tables.partners.get(id).is_some();
                let already_owned = self.context.player()?.owns_partner(id);
                let rewards = self
                    .context
                    .update_player()?
                    .grant_partner(id, amount, tables, now);
                let changed = rewards.partner_rewards.len();
                if changed > 0 {
                    push_reward_notification(&mut self.context, rewards)?;
                }
                GmOutcome {
                    message: if !known {
                        format!("unknown pet {id}")
                    } else if already_owned {
                        format!("granted {amount} duplicate copies of pet {id}")
                    } else {
                        format!("granted {amount} copies of pet {id}")
                    },
                    changed,
                    ..Default::default()
                }
            }
            GmCommand::MaxHero(id) => {
                let outcome = self
                    .context
                    .update_player()?
                    .max_role(id, tables)
                    .map_err(|error| NetworkError::InvalidRoleMutation(error.to_string()))?;
                push_reward_notification(
                    &mut self.context,
                    DcNetDataTakeRewardRes {
                        role_rewards: vec![DcNetDataRoleReward {
                            role_info: Some(outcome.role),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                )?;
                if let Some(card) = outcome.namecard {
                    crate::handlers::red_dot::push_one(&mut self.context, Reddot::Card, card.id)?;
                    self.context.push(
                        NotifyId::DcNetWorkingNotifyProfileNew as u16,
                        DcNetWorkingNotifyProfileNew {
                            avatar: None,
                            card: Some(card),
                        },
                    )?;
                }
                GmOutcome {
                    message: format!("maxed hero {id}"),
                    changed: usize::from(outcome.changed),
                    ..Default::default()
                }
            }
            GmCommand::MaxPet(id) => {
                let (partner, changed) = self
                    .context
                    .update_player()?
                    .max_partner(id, tables)
                    .ok_or_else(|| {
                        NetworkError::InvalidPartnerProgression(format!(
                            "familiar {id} is not owned"
                        ))
                    })?;
                push_reward_notification(
                    &mut self.context,
                    DcNetDataTakeRewardRes {
                        partner_rewards: vec![DcNetDataPartnerReward {
                            partner: Some(partner),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                )?;
                GmOutcome {
                    message: format!("maxed familiar {id}"),
                    changed: usize::from(changed),
                    ..Default::default()
                }
            }
            GmCommand::GrantItem(id, amount) => {
                let relic = tables.equipment.get(id);
                if relic.is_some_and(|relic| relic.english_name.is_empty()) {
                    return Err(NetworkError::InvalidEquipment(
                        logic::EquipmentError::UnavailableRelic(id).to_string(),
                    ));
                }
                let is_relic = relic.is_some();
                let rewards = self
                    .context
                    .update_player()?
                    .grant_reward(id, amount, tables, now)
                    .map_err(|error| NetworkError::InvalidInventory(error.to_string()))?;
                let changed = if is_relic { rewards.equips.len() } else { 1 };
                push_reward_notification(&mut self.context, rewards)?;
                GmOutcome {
                    message: if is_relic {
                        format!("generated {amount} relics of {id}")
                    } else {
                        format!("granted {amount} of item {id}")
                    },
                    changed,
                    ..Default::default()
                }
            }
            GmCommand::GenerateRelic {
                id,
                amount,
                main_stat,
                substats,
            } => {
                let relics = self
                    .context
                    .update_player()?
                    .generate_custom_relics(id, amount, main_stat, &substats, tables, now)
                    .map_err(|error| NetworkError::InvalidEquipment(error.to_string()))?;
                let changed = relics.len();
                push_reward_notification(
                    &mut self.context,
                    DcNetDataTakeRewardRes {
                        equips: relics,
                        ..Default::default()
                    },
                )?;
                GmOutcome {
                    message: format!("generated {changed} custom relics of {id}"),
                    changed,
                    ..Default::default()
                }
            }
            GmCommand::IncreasePlayerLevel(target) => {
                let outcome = self
                    .context
                    .update_player()?
                    .increase_player_level(target, tables, now)
                    .map_err(|error| NetworkError::InvalidTask(error.to_string()))?;
                let task_progress = self
                    .context
                    .update_player()?
                    .advance_level_tasks(tables, now)
                    .map_err(|error| NetworkError::InvalidTask(error.to_string()))?;
                if let Some(update) = outcome.level_update.clone() {
                    self.context
                        .push(NotifyId::DcNetDataAttrUpData as u16, update)?;
                }
                crate::handlers::tasks::push_missions(&mut self.context, &outcome.mission_updates)?;
                crate::handlers::tasks::push_progress(&mut self.context, task_progress, now)?;
                crate::handlers::tasks::push_features(&mut self.context, &outcome.features)?;
                GmOutcome {
                    message: format!("increased player level to {target}"),
                    changed: usize::from(outcome.level_update.is_some())
                        + outcome.mission_updates.len()
                        + outcome.features.len(),
                    ..Default::default()
                }
            }
            GmCommand::IncreaseWorldLevel(target) => {
                let mut outcome = self
                    .context
                    .update_player()?
                    .increase_world_level(target, tables, now)
                    .map_err(|error| NetworkError::InvalidTask(error.to_string()))?;
                let world_level = self.context.player()?.world_level(tables);
                let changed = progression_change_count(&outcome);
                if outcome.level_update.is_none() {
                    outcome.level_update = Some(
                        self.context
                            .player()?
                            .player_level_snapshot(tables, changed == 0),
                    );
                }
                // Always resynchronize this value. A connected client may still hold an
                // older world level even when the requested progression is already stored.
                push_progression(&mut self.context, &outcome, Some(world_level))?;
                GmOutcome {
                    message: format!("increased world level to {target}"),
                    changed,
                    ..Default::default()
                }
            }
            GmCommand::CompleteStoryStage(gameplay_id) => {
                let previous_world_level = self.context.player()?.world_level(tables);
                let outcome = self
                    .context
                    .update_player()?
                    .complete_story_stage(gameplay_id, tables, now)
                    .map_err(|error| NetworkError::InvalidTask(error.to_string()))?;
                let world_level = self.context.player()?.world_level(tables);
                push_progression(
                    &mut self.context,
                    &outcome,
                    (world_level > previous_world_level).then_some(world_level),
                )?;
                GmOutcome {
                    message: format!("completed story stage {gameplay_id}"),
                    changed: progression_change_count(&outcome),
                    ..Default::default()
                }
            }
        };
        if outcome.changed > 0 {
            self.context.save_player().await?;
        }
        let mut packets = Vec::new();
        for reply in self.context.take_outbound() {
            let mut packet = self.encode_reply(reply)?;
            self.crypto
                .as_mut()
                .ok_or(NetworkError::MissingKeyExchange)?
                .encrypt(&mut packet)?;
            packets.push(packet);
        }
        Ok((outcome, packets))
    }
}

fn push_progression(
    ctx: &mut HandlerContext,
    outcome: &logic::ProgressionOutcome,
    world_level: Option<i32>,
) -> NetworkResult<()> {
    push_progression_tasks(
        ctx,
        &outcome.tasks,
        &outcome.completions,
        common::time::ServerTime::now_seconds_i32(),
    )?;
    push_progression_features(ctx, &outcome.features, &outcome.completions)?;
    crate::handlers::tasks::push_missions(ctx, &outcome.mission_updates)?;
    if !outcome.ports.is_empty() {
        ctx.push(
            NotifyId::DcNetWorkingNotifyPortsInfo as u16,
            DcNetWorkingNotifyPortsInfo {
                ports: outcome.ports.clone(),
            },
        )?;
    }
    if let Some(world_level) = world_level {
        ctx.push(
            NotifyId::DcNetWorkingNotifyWorldLevel as u16,
            DcNetWorkingNotifyWorldLevel {
                world_level,
                // The client dereferences this message without a null check.
                updata: Some(outcome.level_update.clone().unwrap_or_default()),
            },
        )?;
    } else if let Some(update) = outcome.level_update.clone() {
        ctx.push(NotifyId::DcNetDataAttrUpData as u16, update)?;
    }
    Ok(())
}

fn push_progression_tasks(
    ctx: &mut HandlerContext,
    tasks: &[protocol::pbcommon::DcNetDataTaskStatus],
    completions: &[logic::TaskCompleteOutcome],
    now: i32,
) -> NetworkResult<()> {
    for completion in completions {
        crate::handlers::tasks::push_completion(ctx, completion, now)?;
    }
    let activated = tasks
        .iter()
        .filter(|task| {
            !completions.iter().any(|completion| {
                completion.current.id == task.id
                    || completion
                        .changed
                        .iter()
                        .any(|changed| changed.id == task.id)
            })
        })
        .copied()
        .collect::<Vec<_>>();
    crate::handlers::tasks::push_task_changes(ctx, &activated)
}

fn push_progression_features(
    ctx: &mut HandlerContext,
    features: &[protocol::pbcommon::DcNetDataFeat],
    completions: &[logic::TaskCompleteOutcome],
) -> NetworkResult<()> {
    let remaining = features
        .iter()
        .filter(|feature| {
            !completions.iter().any(|completion| {
                completion
                    .feat_updates
                    .iter()
                    .any(|updated| updated.id == feature.id)
            })
        })
        .copied()
        .collect::<Vec<_>>();
    crate::handlers::tasks::push_features(ctx, &remaining)
}

fn progression_change_count(outcome: &logic::ProgressionOutcome) -> usize {
    outcome.tasks.len()
        + outcome.ports.len()
        + outcome.features.len()
        + outcome.mission_updates.len()
        + usize::from(outcome.level_update.is_some())
}
