use super::super::super::*;

pub(super) fn restore(
    player: &mut Player,
    record: &mut database::models::game::player_state::PlayerRecord,
) {
    player.checked_red_dots = std::mem::take(&mut record.checked_red_dots);
    player.mails = std::mem::take(&mut record.mails)
        .into_iter()
        .map(|mail| MailState {
            expires_at: mail.expires_at,
            mail: DcNetDataEmail {
                email_id: mail.email_id,
                is_read: mail.is_read,
                taken: mail.taken,
                sent_at: mail.sent_at,
                sender: mail.sender,
                title: mail.title,
                content: mail.content,
                gift_list: mail
                    .gifts
                    .into_iter()
                    .map(|gift| protocol::pbcommon::DcNetDataReward {
                        reward: gift.reward,
                        reward_type: gift.reward_type,
                        amount: gift.amount,
                    })
                    .collect(),
                expire: 0,
                sys_mail_id: mail.sys_mail_id,
                paramter: mail.parameter,
            },
        })
        .collect();
}
