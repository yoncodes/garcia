use super::super::super::*;

pub(super) fn write(
    player: &Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    record.checked_red_dots = player.checked_red_dots.clone();
    record.mails = player
        .mails
        .iter()
        .map(|state| database::models::game::player_state::MailRecord {
            email_id: state.mail.email_id,
            is_read: state.mail.is_read,
            taken: state.mail.taken,
            sent_at: state.mail.sent_at,
            sender: state.mail.sender,
            title: state.mail.title.clone(),
            content: state.mail.content.clone(),
            gifts: state
                .mail
                .gift_list
                .iter()
                .map(
                    |gift| database::models::game::player_state::MailGiftRecord {
                        reward: gift.reward,
                        reward_type: gift.reward_type,
                        amount: gift.amount,
                    },
                )
                .collect(),
            expires_at: state.expires_at,
            sys_mail_id: state.mail.sys_mail_id,
            parameter: state.mail.paramter.clone(),
        })
        .collect();
}
