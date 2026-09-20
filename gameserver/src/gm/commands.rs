use muipserver::GmResponse;
use tokio::sync::oneshot;

use crate::net::{
    app::AppState,
    gm_command::{GmCommand, GmCommandEnvelope},
};

pub(super) async fn execute(state: &AppState, player_uid: i64, input: &str) -> GmResponse {
    match parse(input) {
        Ok(command) => send(state, player_uid, command).await,
        Err(message) => GmResponse::error(400, message),
    }
}

pub(super) async fn send(state: &AppState, player_uid: i64, command: GmCommand) -> GmResponse {
    let Some(session) = state.session(player_uid) else {
        return GmResponse::error(404, format!("player {player_uid} is not online"));
    };
    let (response_tx, response_rx) = oneshot::channel();
    if session
        .send(GmCommandEnvelope {
            command,
            response: response_tx,
        })
        .is_err()
    {
        return GmResponse::error(503, "player session closed");
    }
    match tokio::time::timeout(std::time::Duration::from_secs(5), response_rx).await {
        Ok(Ok(Ok(outcome))) => GmResponse {
            message: outcome.message,
            changed: outcome.changed,
            reconnect_required: outcome.reconnect_required,
            data: outcome.data,
            ..GmResponse::default()
        },
        Ok(Ok(Err(error))) => GmResponse::error(400, error),
        Ok(Err(_)) => GmResponse::error(503, "player session closed"),
        Err(_) => GmResponse::error(504, "player session did not process the command"),
    }
}

fn parse(input: &str) -> Result<GmCommand, String> {
    let command = input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase();
    match command.as_str() {
        "tutorial complete all" => Ok(GmCommand::CompleteTutorials),
        "tutorial reset all" => Ok(GmCommand::ResetTutorials),
        "feature unlock all" => Ok(GmCommand::UnlockFeatures),
        "teleport unlock all" => Ok(GmCommand::UnlockTeleportGates),
        "hero grant all" => Ok(GmCommand::GrantHeroes),
        "pet grant all" => Ok(GmCommand::GrantPets),
        "profile frame unlock all" => Ok(GmCommand::GrantProfileFrames),
        "profile title unlock all" => Ok(GmCommand::GrantProfileTitles),
        _ => parse_grant(&command)
            .or_else(|| parse_max(&command))
            .or_else(|| parse_progression(&command))
            .ok_or_else(|| format!("unknown command `{input}`")),
    }
}

fn parse_max(command: &str) -> Option<GmCommand> {
    let mut words = command.split_whitespace();
    let kind = words.next()?;
    if words.next()? != "max" {
        return None;
    }
    let id = words.next()?.parse().ok()?;
    if words.next().is_some() {
        return None;
    }
    match kind {
        "hero" => Some(GmCommand::MaxHero(id)),
        "pet" | "familiar" => Some(GmCommand::MaxPet(id)),
        _ => None,
    }
}

fn parse_progression(command: &str) -> Option<GmCommand> {
    let mut words = command.split_whitespace();
    let kind = words.next()?;
    let action = words.next()?;
    let value = words.next()?.parse().ok()?;
    if words.next().is_some() {
        return None;
    }
    match (kind, action) {
        ("player", "level") => Some(GmCommand::IncreasePlayerLevel(value)),
        ("world", "level") => Some(GmCommand::IncreaseWorldLevel(value)),
        ("stage", "complete") => Some(GmCommand::CompleteStoryStage(value)),
        _ => None,
    }
}

fn parse_grant(command: &str) -> Option<GmCommand> {
    let mut words = command.split_whitespace();
    let kind = words.next()?;
    if words.next()? != "grant" {
        return None;
    }
    let id = words.next()?.parse().ok()?;
    match kind {
        "hero" if words.next().is_none() => Some(GmCommand::GrantHero(id)),
        "pet" => {
            let amount = words.next().map_or(Some(1), |value| value.parse().ok())?;
            (amount > 0 && words.next().is_none()).then_some(GmCommand::GrantPet(id, amount))
        }
        "relic" => {
            let amount = words.next()?.parse().ok()?;
            let main_stat = words.next()?.parse().ok()?;
            let substats = words.map(str::parse).collect::<Result<Vec<_>, _>>().ok()?;
            (amount > 0 && !substats.is_empty()).then_some(GmCommand::GenerateRelic {
                id,
                amount,
                main_stat,
                substats,
            })
        }
        "item" | "currency" => {
            let amount = words.next()?.parse().ok()?;
            words
                .next()
                .is_none()
                .then_some(GmCommand::GrantItem(id, amount))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_commands() {
        assert!(matches!(
            parse("  HERO   grant ALL "),
            Ok(GmCommand::GrantHeroes)
        ));
        assert!(matches!(
            parse("hero grant 110100001"),
            Ok(GmCommand::GrantHero(110100001))
        ));
        assert!(matches!(
            parse("pet grant 100400001"),
            Ok(GmCommand::GrantPet(100400001, 1))
        ));
        assert!(matches!(
            parse("pet grant 100400001 3"),
            Ok(GmCommand::GrantPet(100400001, 3))
        ));
        assert!(matches!(
            parse("hero max 110100001"),
            Ok(GmCommand::MaxHero(110100001))
        ));
        assert!(matches!(
            parse("familiar max 100400001"),
            Ok(GmCommand::MaxPet(100400001))
        ));
        assert!(matches!(
            parse("profile frame unlock all"),
            Ok(GmCommand::GrantProfileFrames)
        ));
        assert!(matches!(
            parse("profile title unlock all"),
            Ok(GmCommand::GrantProfileTitles)
        ));
        assert!(matches!(
            parse("teleport unlock all"),
            Ok(GmCommand::UnlockTeleportGates)
        ));
        assert!(matches!(
            parse("item grant 100100002 500"),
            Ok(GmCommand::GrantItem(100100002, 500))
        ));
        assert!(matches!(
            parse("relic grant 100500116 2 1 1 2 3 4 5"),
            Ok(GmCommand::GenerateRelic {
                id: 100500116,
                amount: 2,
                main_stat: 1,
                substats,
            }) if substats == [1, 2, 3, 4, 5]
        ));
        assert!(matches!(
            parse("player level 25"),
            Ok(GmCommand::IncreasePlayerLevel(25))
        ));
        assert!(matches!(
            parse("world level 2"),
            Ok(GmCommand::IncreaseWorldLevel(2))
        ));
        assert!(matches!(
            parse("stage complete 180101056"),
            Ok(GmCommand::CompleteStoryStage(180101056))
        ));
        assert!(parse("item grant 100100002").is_err());
        assert!(parse("pet grant 100400001 0").is_err());
        assert!(parse("hero grant 1 extra").is_err());
        assert!(parse("task complete all").is_err());
    }
}
