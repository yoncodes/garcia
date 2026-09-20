use tokio::sync::oneshot;

#[derive(Debug, Clone)]
pub(crate) enum GmCommand {
    Collection,
    TeleportLocations,
    Teleport(String),
    TeleportMission(i32),
    UnlockTeleportGates,
    CompleteTutorials,
    ResetTutorials,
    UnlockFeatures,
    GrantHeroes,
    GrantPets,
    GrantProfileFrames,
    GrantProfileTitles,
    GrantHero(i32),
    GrantPet(i32, i32),
    MaxHero(i32),
    MaxPet(i32),
    GenerateRelic {
        id: i32,
        amount: i32,
        main_stat: i32,
        substats: Vec<i32>,
    },
    GrantItem(i32, i32),
    IncreasePlayerLevel(i32),
    IncreaseWorldLevel(i32),
    CompleteStoryStage(i32),
}

pub(crate) struct GmCommandEnvelope {
    pub command: GmCommand,
    pub response: oneshot::Sender<Result<GmOutcome, String>>,
}

#[derive(Debug, Default)]
pub(crate) struct GmOutcome {
    pub message: String,
    pub changed: usize,
    pub reconnect_required: bool,
    pub data: Option<serde_json::Value>,
}
