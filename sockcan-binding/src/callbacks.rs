/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * $RP_BEGIN_LICENSE$
 * Commercial License Usage
 *  Licensees holding valid commercial IoT.bzh licenses may use this file in
 *  accordance with the commercial license agreement provided with the
 *  Software or, alternatively, in accordance with the terms contained in
 *  a written agreement between you and The IoT.bzh Company. For licensing terms
 *  and conditions see https://www.iot.bzh/terms-conditions. For further
 *  information use the contact form at https://www.iot.bzh/contact.
 *
 * GNU General Public License Usage
 *  Alternatively, this file may be used under the terms of the GNU General
 *  Public license version 3. This license is as published by the Free Software
 *  Foundation and appearing in the file LICENSE.GPLv3 included in the packaging
 *  of this file. Please review the following information to ensure the GNU
 *  General Public License requirements will be met
 *  https://www.gnu.org/licenses/gpl-3.0.html.
 * $RP_END_LICENSE$
 */

use crate::context::{AfbClientData, CanEvtCtx, CheckCtx, SessionCtx, SubVerbCtx};
use afbv4::prelude::*;

use sockcan::prelude::{
    CanBcmFlag, CanBcmOpCode, CanError, CanTimeStamp, SockBcmCmd, SockBcmMsg, SockCanBmc,
    SockCanHandle,
};
use sockdata::types::{CanBmcData, CanBmcError, SubscribeParam, UnSubscribeParam};
use std::sync::Arc;

// Handler appelé quand le fd CAN a des données
pub(crate) fn async_can_cb(
    _evtfd: &AfbEvtFd,
    revent: u32,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let ctx: &CanEvtCtx = ctx.get_ref::<CanEvtCtx>()?;
    let data = |msg: SockBcmMsg| -> Result<CanBmcData, CanError> {
        Ok(CanBmcData {
            canid: msg.get_id()?,
            stamp: msg.get_stamp(),
            opcode: msg.get_opcode(),
            len: msg.get_len()?,
            data: msg.get_data()?.to_vec(),
        })
    };

    if revent == AfbEvtFdPoll::IN.bits() {
        let msg = ctx.client.sockfd.get_bcm_frame();
        let opcode = msg.get_opcode();
        let msgid = msg.get_id();
        let listener = match data(msg) {
            Err(error) => {
                ctx.client.event.push(CanBmcError::new(error.get_uid(), -1, error.get_info()))
            },
            Ok(data) => ctx.client.event.push(data),
        };

        if let CanBcmOpCode::RxTimeout = opcode {
            if let Ok(canid) = msgid {
                if let Err(_error) = SockBcmCmd::new(
                    CanBcmOpCode::RxSetup,
                    CanBcmFlag::RX_FILTER_ID
                        | CanBcmFlag::SET_TIMER
                        | CanBcmFlag::START_TIMER
                        | CanBcmFlag::RX_ANNOUNCE_RESUME,
                    canid,
                )
                .set_timers(ctx.client.rate, ctx.client.watchdog)
                .apply(&ctx.client.sockfd)
                {
                    afb_log_msg!(
                        Warning,
                        ctx.client.event,
                        "fail-sockbmc-filter canid={} rate={} watchdog={}",
                        canid,
                        ctx.client.rate,
                        ctx.client.watchdog
                    );
                    return Ok(());
                }
            }
        };

        // if no more listener, then close socket BMC + delete event
        if listener < 1 {
            afb_log_msg!(
                Debug,
                ctx.client.event,
                "closing-bmc-event uid:{} no more listener",
                ctx.client.uid
            );
            ctx.client.event.unref(); // delete associated event
            ctx.client.sockfd.close(); // close associated socket

            return Ok(());
        }
    }
    Ok(())
}

// ============ Subscribe Canids ===============
pub(crate) fn subscribe_cb(
    request: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<SubVerbCtx>()?;
    // parse api query
    let param = args.get::<&SubscribeParam>(0)?;

    if param.get_canids().is_empty() {
        let error = AfbError::new("fail-empty-canids", 0, "pool canids list is empty");
        afb_log_msg!(Warning, request, &error);
        return Err(error);
    }

    // check if we already have a running session
    let session = match SessionCtx::get_from(request) {
        Ok(session) => session,
        Err(_) => {
            // open socketcan
            let sockfd = match SockCanHandle::open_bcm(ctx.candev, CanTimeStamp::CLASSIC) {
                Ok(handle) => handle,
                Err(bmcerr) => {
                    let error = AfbError::new("fail-sockbmc-open", 0, bmcerr.to_string());
                    afb_log_msg!(Warning, request, &error);
                    return Err(error);
                },
            };

            let event = AfbEvent::new(ctx.sockevt);
            if event.register(request.get_api().get_apiv4()) < 0 {
                let error = AfbError::new(
                    "evt-fail-registration",
                    0,
                    format!("evt-fail-registration uid:{}", ctx.uid),
                );
                afb_log_msg!(Warning, request, &error);
                return Err(error);
            } else {
                event.finalize()?;
            }
            #[allow(clippy::arc_with_non_send_sync)]
            let client_data = Arc::new(AfbClientData {
                uid: ctx.uid,
                sockfd,
                event,
                rate: param.get_rate(),
                watchdog: param.get_watchdog(),
            });

            // subscribe to newly created event
            client_data.event.subscribe(request)?;

            // Subscribe sockfd handler within AFB mainloop
            AfbEvtFd::new(ctx.uid)
                .set_fd(client_data.sockfd.as_rawfd())
                .set_events(AfbEvtFdPoll::IN)
                .set_callback(async_can_cb)
                .set_context(CanEvtCtx { client: Arc::clone(&client_data) })
                .start()?;

            SessionCtx::set_for(request, SessionCtx { client: Arc::clone(&client_data) })?
        },
    };

    // Subscribe to bmc can event
    let mut can_error: Vec<u32> = Vec::new();
    for canid in param.get_canids() {
        let mut filter = SockBcmCmd::new(
            CanBcmOpCode::RxSetup,
            CanBcmFlag::RX_FILTER_ID
                | CanBcmFlag::SET_TIMER
                | CanBcmFlag::START_TIMER
                | CanBcmFlag::RX_ANNOUNCE_RESUME,
            *canid,
        );

        if param.get_rate() > 0 || param.get_watchdog() > 0 {
            filter.set_timers(param.get_rate(), param.get_watchdog());
        }

        match filter.apply(&session.client.sockfd) {
            Ok(()) => {},
            Err(_error) => can_error.push(*canid),
        }
    }

    // subscription to canid fail.
    if !can_error.is_empty() {
        let error = AfbError::new(
            "fail-canid-Subscribe",
            0,
            format!("Fail to Subscribe canids={:?}", can_error),
        );
        afb_log_msg!(Warning, request, &error);
        return Err(error);
    }

    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

// ============ Unsubscribe Canids ===============
pub(crate) fn unsubscribe_cb(
    request: &AfbRequest,
    args: &AfbRqtData,
    _ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let session = SessionCtx::get_from(request)?;
    afb_log_msg!(Notice, request, "unsubscribe from session uid:{}", session.client.uid);

    let param = args.get::<&UnSubscribeParam>(0)?;

    if param.get_canids().is_empty() {
        let error = AfbError::new("fail-empty-canids", 0, "canids list is empty");
        afb_log_msg!(Warning, request, &error);
        return Err(error);
    }

    // Subscribe to bmc can event
    let mut can_error: Vec<u32> = Vec::new();
    for canid in param.get_canids() {
        //TODO: Make comment
        let mut filter = SockBcmCmd::new(CanBcmOpCode::RxDelete, CanBcmFlag::NONE, *canid);

        match filter.apply(&session.client.sockfd) {
            Ok(()) => {},
            Err(_error) => can_error.push(*canid),
        }
    }

    // subscription to canid fail.
    if !can_error.is_empty() {
        let error = AfbError::new(
            "fail-canid-Subscribe",
            0,
            format!("Fail to UnSubscribe canids={:?}", can_error),
        );
        afb_log_msg!(Warning, request, &error);
        return Err(error);
    }

    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

// ============ Close SockBmc ===============
pub(crate) fn close_cb(
    request: &AfbRequest,
    _args: &AfbRqtData,
    _ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let session = SessionCtx::get_from(request)?;
    afb_log_msg!(Notice, request, "closing subscription uid:{}", session.client.uid);
    session.client.event.unref();
    session.client.sockfd.close();
    let _ = SessionCtx::unref_from(request);
    Ok(())
}

// =========== Check SockBmc ===============
pub(crate) fn check_cb(
    request: &AfbRequest,
    _args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let vbdata: &mut CheckCtx = ctx.get_mut::<CheckCtx>()?;
    // open/close socketcan
    match SockCanHandle::open_bcm(vbdata.candev, CanTimeStamp::CLASSIC) {
        Ok(sock) => sock.close(),
        Err(bmcerr) => {
            let error = AfbError::new("fail-sockbmc-open", 0, bmcerr.to_string());
            afb_log_msg!(Warning, request, &error);
            return Err(error);
        },
    };

    request.reply(AFB_NO_DATA, 0);
    Ok(())
}
