use protocol::{
    cs::{
        DcNetWorkingParamFriendGiftClaim, DcNetWorkingParamFriendGiftSend,
        DcNetWorkingParamFriendsApprove, DcNetWorkingParamFriendsInfo,
        DcNetWorkingParamFriendsRequest, DcNetWorkingParamFriendsSearch,
        DcNetWorkingResFriendGiftClaim, DcNetWorkingResFriendsApprove, DcNetWorkingResFriendsInfo,
        DcNetWorkingResFriendsSearch,
    },
    proids::Id,
    prost::Message,
};

use std::sync::Arc;

use super::*;
use crate::net::app::AppState;

fn packet(cmd: Id, message: impl Message) -> ClientPacket {
    ClientPacket {
        timestamp: 1,
        ack: 0,
        proto_id: cmd as u16,
        payload: message.encode_to_vec(),
    }
}

#[tokio::test]
async fn handlers_search_friend_approve_and_exchange_daily_gift() {
    let config = common::load_config().unwrap();
    common::init_config(config.clone());
    let tables = configs::GameTables::load(&config.paths.game_tables).unwrap();
    let db = database::connect_memory().await.unwrap();
    let state = Arc::new(AppState::new(db.clone(), tables));

    let mut alice = Player::new(10, &state.tables, &config.account_defaults);
    alice.nickname = "Alice".into();
    let mut bob = Player::new(20, &state.tables, &config.account_defaults);
    bob.nickname = "Bob".into();
    database::db::player_state::save(&db, &alice.to_record())
        .await
        .unwrap();
    database::db::player_state::save(&db, &bob.to_record())
        .await
        .unwrap();

    let mut alice_ctx = HandlerContext::new(state.clone());
    alice_ctx.player = Some(alice);
    on_search(
        &mut alice_ctx,
        packet(
            Id::DcNetWorkingParamFriendsSearch,
            DcNetWorkingParamFriendsSearch { fuid: 20 },
        ),
    )
    .await
    .unwrap();
    let search =
        DcNetWorkingResFriendsSearch::decode(alice_ctx.take_reply().unwrap().body.as_slice())
            .unwrap();
    assert_eq!(search.profile.unwrap().nickname, "Bob");

    on_request(
        &mut alice_ctx,
        packet(
            Id::DcNetWorkingParamFriendsRequest,
            DcNetWorkingParamFriendsRequest { fuid: 20 },
        ),
    )
    .await
    .unwrap();
    alice_ctx.take_reply().unwrap();

    let mut bob_ctx = HandlerContext::new(state.clone());
    bob_ctx.player = Some(bob);
    on_info(
        &mut bob_ctx,
        packet(
            Id::DcNetWorkingParamFriendsInfo,
            DcNetWorkingParamFriendsInfo {},
        ),
    )
    .await
    .unwrap();
    let info =
        DcNetWorkingResFriendsInfo::decode(bob_ctx.take_reply().unwrap().body.as_slice()).unwrap();
    assert_eq!(
        info.friends_info.unwrap().pending_approves[0].nickname,
        "Alice"
    );

    on_approve(
        &mut bob_ctx,
        packet(
            Id::DcNetWorkingParamFriendsApprove,
            DcNetWorkingParamFriendsApprove { fuid: 10 },
        ),
    )
    .await
    .unwrap();
    let approved =
        DcNetWorkingResFriendsApprove::decode(bob_ctx.take_reply().unwrap().body.as_slice())
            .unwrap();
    assert_eq!(approved.friends_info.unwrap().friends[0].nickname, "Alice");

    on_send_gift(
        &mut alice_ctx,
        packet(
            Id::DcNetWorkingParamFriendGiftSend,
            DcNetWorkingParamFriendGiftSend { fuid: 20 },
        ),
    )
    .await
    .unwrap();
    alice_ctx.take_reply().unwrap();

    on_claim_gift(
        &mut bob_ctx,
        packet(
            Id::DcNetWorkingParamFriendGiftClaim,
            DcNetWorkingParamFriendGiftClaim { fuid: 10 },
        ),
    )
    .await
    .unwrap();
    let claim =
        DcNetWorkingResFriendGiftClaim::decode(bob_ctx.take_reply().unwrap().body.as_slice())
            .unwrap();
    assert_eq!(claim.gift_num, 1);
    assert_eq!(claim.gifts[0].item_id, 100_100_008);
    assert_eq!(claim.gifts[0].amount, 241);
    assert_eq!(claim.friends[0].gift, 0);
}
