use protocol::{cs::DcNetWorkingNotifyAchi, pbcommon::DcNetDataAchi, proids::NotifyId};

use crate::net::{context::HandlerContext, error::NetworkResult};

pub(crate) fn push_updates(
    ctx: &mut HandlerContext,
    achievements: &[DcNetDataAchi],
) -> NetworkResult<()> {
    for achievement in achievements {
        ctx.push(
            NotifyId::DcNetWorkingNotifyAchi as u16,
            DcNetWorkingNotifyAchi {
                achi: Some(*achievement),
            },
        )?;
    }
    Ok(())
}
