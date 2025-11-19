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

#![doc(
    html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png",
    html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico"
)]

#[cfg(not(afbv4))]
extern crate afbv4;
extern crate sockcan;
extern crate sockdata;

const MSG_DFT_RATE: u64 = 500;
const MSG_DFT_WATCHDOG: u64 = 10000;

// libafb + sockcan/sockdata imports
use afbv4::prelude::*;

use sockcan::prelude::{
    CanDataStatus, CanDbcMessage, CanDbcPool, CanDbcSignal, CanMsgCtrl, CanMsgData, CanSigCtrl,
};

use sockdata::prelude::{
    sockdata_register, CanBmcData, DataBcmMsg, DataBmcSig, SubscribeFlag, SubscribeParam,
    UnSubscribeParam,
};

use std::cell::RefCell;
use std::rc::Rc;

#[allow(clippy::upper_case_acronyms)]
enum Action {
    SUBSCRIBE,
    UNSUBSCRIBE,
    READ,
    RESET,
}

/// Runtime info associated with a pool of signals/messages.
struct PoolInfoCtx {
    stamp: u64,
    rate: u64,
    watchdog: u64,
    listeners: i32,
    flag: SubscribeFlag,
}

/// Per-signal runtime data (throttle, watchdog, subscribers, event handle).
struct SigDataCtx {
    info: RefCell<PoolInfoCtx>,
    event: &'static AfbEvent,
}

/// Controller passed to the DBC signal to push notifications into AFB events.
struct SigPoolCtx {
    data: Rc<SigDataCtx>,
}

impl CanSigCtrl for SigPoolCtx {
    fn sig_notification(&self, sig: &dyn CanDbcSignal) -> i32 {
        // Try to borrow mutably the per-signal info; if already borrowed, report error.
        let mut info = match self.data.info.try_borrow_mut() {
            Err(_) => {
                afb_log_msg!(
                    Critical,
                    self.data.event,
                    "pool-sig-notification: failed to get event info"
                );
                return -1;
            },
            Ok(info) => info,
        };

        let signal = DataBmcSig {
            name: sig.get_name().to_owned(),
            status: sig.get_status(),
            stamp: sig.get_stamp(),
            value: sig.get_value(),
        };

        // Push event depending on update status and throttling policy, update listeners count.
        match sig.get_status() {
            CanDataStatus::Updated => {
                if (sig.get_stamp() - info.stamp) > info.rate * 1000 {
                    info.stamp = sig.get_stamp();
                    info.listeners = self.data.event.push(signal);
                };
                info.listeners
            },
            _ => {
                if (sig.get_stamp() - info.stamp) > info.watchdog * 1000
                    && info.flag == SubscribeFlag::ALL
                {
                    info.stamp = sig.get_stamp();
                    info.listeners = self.data.event.push(signal);
                };
                info.listeners
            },
        }
    }
}

/// Verb callback context for a single signal.
struct SigVerbCtx {
    sig_rfc: Rc<RefCell<Box<dyn CanDbcSignal>>>,
    msg_rfc: Rc<RefCell<Box<dyn CanDbcMessage>>>,
    msg_ctx: Rc<MessageDataCtx>,
    data: Rc<SigDataCtx>,
}

/// Verb for signal operations: subscribe/unsubscribe/read/reset.
fn signal_vcb(request: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<SigVerbCtx>()?;
    let jquery = args.get::<JsoncObj>(0)?;
    let jaction = jquery.get::<String>("action")?;
    let action = match jaction.to_uppercase().as_str() {
        "SUBSCRIBE" => Action::SUBSCRIBE,
        "UNSUBSCRIBE" => Action::UNSUBSCRIBE,
        "READ" => Action::READ,
        "RESET" => Action::RESET,
        _ => {
            let error =
                AfbError::new("invalid-action", 0, "expect: SUBSCRIBE|UNSUBSCRIBE|READ|RESET");
            return Err(error);
        },
    };

    // Borrow underlying signal/message instances and pool infos.
    let mut sig = match ctx.sig_rfc.try_borrow_mut() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-sig",
                0,
                "internal pool error (sig rfc cell already used)",
            );
            return Err(afb_add_trace!(error));
        },
    };

    let msg = match ctx.msg_rfc.try_borrow() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-msg",
                0,
                "internal pool error (msg rfc cell already used)",
            );
            return Err(afb_add_trace!(error));
        },
    };

    let mut msg_info = match ctx.msg_ctx.info.try_borrow_mut() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-info",
                0,
                "internal pool error (msg info cell already used)",
            );
            return Err(afb_add_trace!(error));
        },
    };

    let mut sig_info = match ctx.data.info.try_borrow_mut() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-info",
                0,
                "internal pool error (sig info cell already used)",
            );
            return Err(afb_add_trace!(error));
        },
    };

    match action {
        Action::SUBSCRIBE => {
            // Subscribe the requester to the signal's event stream.
            ctx.data.event.subscribe(request)?;
            let rate = jquery.get::<u64>("rate").unwrap_or(msg_info.rate);
            let watchdog = jquery.get::<u64>("watchdog").unwrap_or(msg_info.watchdog);
            let flag = jquery
                .get::<String>("flag")
                .ok()
                .and_then(|v| match v.to_uppercase().as_str() {
                    "NEW" => Some(SubscribeFlag::NEW),
                    "ALL" => Some(SubscribeFlag::ALL),
                    _ => None,
                })
                .unwrap_or_else(|| msg_info.flag.clone());

            // Update signal throttling if tighter.
            if rate < sig_info.rate {
                sig_info.rate = rate
            }
            if watchdog < sig_info.watchdog {
                sig_info.watchdog = watchdog
            }

            // If message-level subscription parameters need tightening, propagate to sockcan.
            if msg_info.stamp == 0
                || watchdog < msg_info.watchdog
                || rate < msg_info.rate
                || msg_info.flag != sig_info.flag
            {
                if flag == SubscribeFlag::ALL {
                    msg_info.flag = SubscribeFlag::ALL;
                }
                if rate < msg_info.rate {
                    msg_info.rate = rate
                }
                if watchdog < msg_info.watchdog {
                    msg_info.watchdog = watchdog
                }
                msg_info.stamp = 1;

                AfbSubCall::call_sync(
                    request,
                    ctx.msg_ctx.bmc,
                    "subscribe",
                    SubscribeParam::new(
                        vec![msg.get_id()],
                        msg_info.watchdog,
                        msg_info.rate,
                        msg_info.flag.clone(),
                    ),
                )?;
            }
            sig_info.listeners += 1; // we have at least one listener now
            request
                .reply(format!("Subscribe (canid:{}) sig:{} OK", msg.get_id(), sig.get_name(),), 0);
        },

        Action::UNSUBSCRIBE => {
            ctx.data.event.unsubscribe(request)?;
            request.reply(
                format!("UnSubscribe (canid:{}) sig:{} OK", msg.get_id(), sig.get_name(),),
                0,
            );
        },

        Action::READ => {
            // Return current signal snapshot.
            let sig_data = DataBmcSig {
                name: sig.get_name().to_owned(),
                stamp: sig.get_stamp(),
                status: sig.get_status(),
                value: sig.get_value(),
            };
            let mut params = AfbParams::new();
            params.push(sig_data)?;
            request.reply(params, 0);
        },

        Action::RESET => {
            sig.reset();
            request.reply(format!("Reset (canid:{}) sig:{} OK", msg.get_id(), sig.get_name(),), 0);
        },
    };
    Ok(())
}

/// Register a signal verb and its event; returns (&verb, &event) for grouping.
fn register_signal(
    _config: &SockBmcConfig,
    msg_ctx: &Rc<MessageDataCtx>,
    msg_rfc: &Rc<RefCell<Box<dyn CanDbcMessage>>>,
    sig_rfc: &Rc<RefCell<Box<dyn CanDbcSignal>>>,
) -> Result<(&'static AfbVerb, &'static AfbEvent), AfbError> {
    let mut sig_ref = match sig_rfc.try_borrow_mut() {
        Err(_) => return Err(AfbError::new("register-sig-fail", 0, "internal pool error")),
        Ok(sig) => sig,
    };

    // Create an event per signal.
    let sig_event = AfbEvent::new(sig_ref.get_name()).finalize()?;

    // Initialize per-signal runtime context.
    let info = PoolInfoCtx {
        rate: MSG_DFT_RATE,
        watchdog: MSG_DFT_WATCHDOG,
        stamp: 0,
        listeners: 0,
        flag: SubscribeFlag::NEW,
    };
    let sigdata = Rc::new(SigDataCtx { event: sig_event, info: RefCell::new(info) });

    // Attach controller to push updates into the event.
    sig_ref.set_callback(Box::new(SigPoolCtx { data: sigdata.clone() }));

    // Build and finalize the verb for this signal.
    let sig_verb = AfbVerb::new(sig_ref.get_name())
        .set_actions("['reset','read','subscribe','unsubscribe']")?
        .add_sample("{'action':'subscribe','rate':250,'watchdog':5000,'flag':'all'}")?
        .set_callback(signal_vcb)
        .set_context(SigVerbCtx {
            data: sigdata.clone(),
            sig_rfc: sig_rfc.clone(),
            msg_rfc: msg_rfc.clone(),
            msg_ctx: msg_ctx.clone(),
        })
        .finalize()?;

    Ok((sig_verb, sig_event))
}

/// Per-message runtime data (throttle/flag) + event + backend API name.
struct MessageDataCtx {
    info: RefCell<PoolInfoCtx>,
    event: &'static AfbEvent,
    bmc: &'static str,
}

/// Verb callback context for a message.
struct MessageVerbCtx {
    msg_rfc: Rc<RefCell<Box<dyn CanDbcMessage>>>,
    data: Rc<MessageDataCtx>,
}

/// Controller attached to the message pool to push message updates as events.
struct MessagePoolCtx {
    data: Rc<MessageDataCtx>,
}

// Called by the CAN pool when a message gets updated.
impl CanMsgCtrl for MessagePoolCtx {
    fn msg_notification(&self, msg: &dyn CanDbcMessage) {
        let msg_value =
            DataBcmMsg { canid: msg.get_id(), stamp: msg.get_stamp(), status: msg.get_status() };

        let info = match self.data.info.try_borrow() {
            Err(_) => {
                afb_log_msg!(
                    Critical,
                    self.data.event,
                    "pool-msg-notification: failed to get event info"
                );
                return;
            },
            Ok(info) => info,
        };

        // Build a parameter pack containing the message snapshot + selected signals.
        let params = |msg: &dyn CanDbcMessage| -> Result<AfbParams, AfbError> {
            let mut args = AfbParams::new();
            args.push(msg_value)?;

            for sig_rfc in msg.get_signals() {
                let sig = match sig_rfc.try_borrow() {
                    Ok(value) => value,
                    Err(_) => {
                        let error = AfbError::new(
                            "fail-borrow-sig",
                            0,
                            "internal pool error (sig rfc cell already used)",
                        );
                        return Err(error);
                    },
                };
                let sig_value = DataBmcSig {
                    name: sig.get_name().to_string(),
                    status: sig.get_status(),
                    stamp: sig.get_stamp(),
                    value: sig.get_value(),
                };

                match info.flag {
                    SubscribeFlag::NEW => {
                        if sig.get_status() == CanDataStatus::Updated {
                            args.push(sig_value)?;
                        }
                    },
                    SubscribeFlag::ALL => {
                        args.push(sig_value)?;
                    },
                }
            }

            Ok(args)
        };

        let args = match params(msg) {
            Err(_) => {
                afb_log_msg!(
                    Critical,
                    self.data.event,
                    "pool-msg-notification: failed to build event params"
                );
                return;
            },
            Ok(value) => value,
        };

        // Push event; if no more listeners, clear backend subscription.
        let listener = self.data.event.push(args);
        if listener + msg.get_listeners() < 1 {
            afb_log_msg!(
                Notice,
                self.data.event,
                format!("msg-empty-listener: clearing canid={} subscription", msg.get_id())
            );

            // No active listener: unsubscribe from backend.
            let _status = AfbSubCall::call_sync(
                self.data.event.get_apiv4(),
                self.data.bmc,
                "unsubscribe",
                UnSubscribeParam::new(vec![msg.get_id()]),
            );

            // Reset per-message throttling so the next subscribe recomputes it.
            match self.data.info.try_borrow_mut() {
                Err(_) => {},
                Ok(mut info) => {
                    info.stamp = 0;
                    info.rate = 0;
                    info.watchdog = 0;
                },
            }
        }
    }
}

/// Verb for message operations: subscribe/unsubscribe/read/reset.
fn message_vcb(request: &AfbRequest, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<MessageVerbCtx>()?;
    let jquery = args.get::<JsoncObj>(0)?;
    let jaction = jquery.get::<String>("action")?;
    let action = match jaction.to_uppercase().as_str() {
        "SUBSCRIBE" => Action::SUBSCRIBE,
        "UNSUBSCRIBE" => Action::UNSUBSCRIBE,
        "READ" => Action::READ,
        "RESET" => Action::RESET,
        _ => {
            let error =
                AfbError::new("invalid-action", 0, "expect: SUBSCRIBE|UNSUBSCRIBE|READ|RESET");
            return Err(error);
        },
    };

    // Borrow the message and its runtime info.
    let mut msg = match ctx.msg_rfc.try_borrow_mut() {
        Ok(value) => value,
        Err(_) => {
            let error =
                AfbError::new("fail-borrow-msg", 0, "internal pool error (msg cell already used)");
            return Err(afb_add_trace!(error));
        },
    };

    let mut msg_info = match ctx.data.info.try_borrow_mut() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-info",
                0,
                "internal pool error (info cell already used)",
            );
            return Err(afb_add_trace!(error));
        },
    };

    match action {
        Action::SUBSCRIBE => {
            ctx.data.event.subscribe(request)?;

            let rate = jquery.get::<u64>("rate").unwrap_or(msg_info.rate);
            let watchdog = jquery.get::<u64>("watchdog").unwrap_or(msg_info.watchdog);
            let flag = jquery
                .get::<String>("flag")
                .ok()
                .and_then(|v| match v.to_uppercase().as_str() {
                    "NEW" => Some(SubscribeFlag::NEW),
                    "ALL" => Some(SubscribeFlag::ALL),
                    _ => None,
                })
                .unwrap_or_else(|| msg_info.flag.clone());

            // If we need to tighten backend subscription, do it now.
            if msg_info.stamp == 0
                || watchdog < msg_info.watchdog
                || rate < msg_info.rate
                || msg_info.flag != flag
            {
                if flag == SubscribeFlag::ALL {
                    msg_info.flag = SubscribeFlag::ALL;
                }
                if rate < msg_info.rate {
                    msg_info.rate = rate
                }
                if watchdog < msg_info.watchdog {
                    msg_info.watchdog = watchdog
                }
                msg_info.stamp = 1;

                AfbSubCall::call_sync(
                    request,
                    ctx.data.bmc,
                    "subscribe",
                    SubscribeParam::new(
                        vec![msg.get_id()],
                        msg_info.watchdog,
                        msg_info.rate,
                        msg_info.flag.clone(),
                    ),
                )?;
            }
            request
                .reply(format!("Subscribe (canid:{}) msg:{} OK", msg.get_id(), msg.get_name(),), 0);
        },

        Action::UNSUBSCRIBE => {
            ctx.data.event.unsubscribe(request)?;
            request.reply(
                format!("UnSubscribe (canid:{}) msg:{} OK", msg.get_id(), msg.get_name(),),
                0,
            );
        },

        Action::READ => {
            // Return current message snapshot.
            let msg_data = DataBcmMsg {
                canid: msg.get_id(),
                stamp: msg.get_stamp(),
                status: msg.get_status(),
            };
            let mut params = AfbParams::new();
            params.push(msg_data)?;
            request.reply(params, 0);
        },

        Action::RESET => match msg.reset() {
            Err(_) => {
                return Err(AfbError::new(
                    "reset-msg-fail",
                    0,
                    "internal pool (fail to get borrow mut)",
                ))
            },
            Ok(()) => {
                request
                    .reply(format!("Reset (canid:{}) msg:{} OK", msg.get_id(), msg.get_name(),), 0);
            },
        },
    };
    Ok(())
}

/// Static configuration given to registration helpers.
struct SockBmcConfig {
    _uid: &'static str,
    bmc: &'static str,
    _evt: &'static str,
    jconf: JsoncObj,
    api: *const AfbApi,
}

/// Create a verb for a message, its event, and a group for its signals.
fn register_msg(
    config: &SockBmcConfig,
    msg_rfc: &Rc<RefCell<Box<dyn CanDbcMessage>>>,
) -> Result<(), AfbError> {
    let msg_ref = msg_rfc.clone();
    let mut msg = match msg_ref.try_borrow_mut() {
        Err(_) => {
            return Err(AfbError::new(
                "register-msg-fail",
                0,
                "internal pool (fail to get borrow mut)",
            ))
        },
        Ok(msg) => msg,
    };

    let msg_name = msg.get_name();

    let mut msg_acls = AFB_NO_AUTH;
    let api = unsafe { &mut *(config.api as *const _ as *mut AfbApi) };

    // Create a verb named after the CAN message.
    let msg_verb = AfbVerb::new(msg_name);
    let mut info = PoolInfoCtx {
        rate: MSG_DFT_RATE,
        watchdog: MSG_DFT_WATCHDOG,
        stamp: 0,
        listeners: 0,
        flag: SubscribeFlag::NEW,
    };

    // Optional extra verb parameters from JSON (rate/watchdog/acls/info).
    if let Ok(jverb) = config.jconf.get::<JsoncObj>(msg_name) {
        if let Ok(value) = jverb.get::<String>("info") {
            msg_verb.set_info(to_static_str(value));
        } else {
            msg_verb.set_info(to_static_str(format!("(canid:{})", msg.get_id())));
        }
        if let Ok(acls) = jverb.get::<String>("acls") {
            msg_acls =
                AfbPermisionV4::new(AfbPermission::new(to_static_str(acls)), AFB_AUTH_DFLT_V4);
        }
        if let Ok(rate) = jverb.get::<u64>("rate") {
            info.rate = rate
        }
        if let Ok(watchdog) = jverb.get::<u64>("watchdog") {
            info.watchdog = watchdog
        }
    } else {
        msg_verb.set_info(to_static_str(format!("(canid:{})", msg.get_id())));
    }

    // Create a message-wide event and its runtime context.
    let event = AfbEvent::new(msg_name).finalize()?;
    let vcbdata = Rc::new(MessageDataCtx { bmc: config.bmc, event, info: RefCell::new(info) });

    // Attach controller so pool updates push to this event.
    msg.set_callback(Box::new(MessagePoolCtx { data: vcbdata.clone() }));

    // Finalize and register the message verb.
    unsafe {
        msg_verb
            .set_actions("['reset','read','subscribe','unsubscribe']")?
            .add_sample("{'action':'subscribe','rate':250,'watchdog':5000,'flag':'new'}")?
            .set_callback(message_vcb)
            .set_context(MessageVerbCtx { msg_rfc: msg_rfc.clone(), data: vcbdata.clone() })
            .register(api.get_apiv4(), msg_acls);
    }

    // Build a group containing the message event and all signal verbs.
    let mut group = AfbGroup::new(msg_name)
        .add_event(event)
        .set_info(to_static_str(format!("(canid:{})", msg.get_id())));

    for sig_rfc in msg.get_signals() {
        let (verb, event) = register_signal(config, &vcbdata, msg_rfc, sig_rfc)?;
        group = group.add_verb(verb);
        group = group.add_event(event);
    }
    unsafe {
        group.register(api.get_apiv4(), msg_acls);
    }

    Ok(())
}

/// Context passed to the low-level event handler (backend → pool).
struct EvtUserData {
    pool: &'static mut dyn CanDbcPool,
}

/// Handler for raw BMC frames coming from the backend; updates the pool.
fn bmc_event_cb(event: &AfbEventMsg, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx: &EvtUserData = ctx.get_ref::<EvtUserData>()?;

    // Extract backend CAN frame as CanBmcData.
    let bmc_frame = match args.get::<&CanBmcData>(0) {
        Err(_) => {
            let error = AfbError::new(
                "event-bmc-invalid",
                0,
                "internal error: event is not SockBmcMsg type",
            );
            afb_log_msg!(Critical, event, &error);
            return Ok(());
        },
        Ok(value) => value,
    };

    // Convert to pool format and update the pool.
    let pool_frame = CanMsgData {
        canid: bmc_frame.get_id(),
        stamp: bmc_frame.get_stamp(),
        opcode: bmc_frame.get_opcode(),
        len: bmc_frame.get_len(),
        data: bmc_frame.get_data().as_slice(),
    };
    match ctx.pool.update(&pool_frame) {
        Err(_) => {
            let error = AfbError::new(
                "event-pool-update",
                0,
                format!("Fail to update message pool canid:{}", bmc_frame.get_id()),
            );
            afb_log_msg!(Critical, event, &error);
            return Ok(());
        },
        Ok(_msg) => {},
    };
    Ok(())
}

/// Create verbs/events/groups from the DBC pool and hook backend events.
pub fn create_pool_verbs(
    api: &AfbApi,
    jconf: JsoncObj,
    pool_box: Box<dyn CanDbcPool>,
) -> Result<(), AfbError> {
    // Register data converters for sockdata <-> afb types.
    sockdata_register(api.get_apiv4()).expect("sockdata_register failed");

    // Read from `args` subobject if present, otherwise fall back to the root object (compat).
    let conf = match jconf.get::<JsoncObj>("args") {
        Ok(a) => a,
        Err(_) => jconf.clone(),
    };

    // Basic runtime configuration.
    let uid = to_static_str(conf.get::<String>("uid")?);
    let bmc = to_static_str(conf.get::<String>("sock_api")?);
    let evt = to_static_str(conf.get::<String>("sock_evt")?);

    // Leak the pool to bind its lifetime to the API (intended design in this binding).
    let pool = Box::leak(pool_box);

    let bmc_config = SockBmcConfig { _uid: uid, api, bmc, _evt: evt, jconf };

    // Register message verbs + signal groups for each message in the pool.
    for msg in pool.get_messages() {
        register_msg(&bmc_config, msg)?
    }

    // Subscribe to backend raw frames (bmc/evt) and feed the DBC pool.
    let pattern = to_static_str(format!("{}/{}", bmc, evt));
    let evt_handler = AfbEvtHandler::new(uid)
        .set_info("Receive low-level BMC data frame")
        .set_pattern(pattern)
        .set_callback(bmc_event_cb)
        .set_context(EvtUserData { pool });
    unsafe {
        evt_handler.register(api.get_apiv4());
    }
    evt_handler.finalize()?;

    // No need to call api.add_evt_handler(evt_handler); C-side registration is already done.
    Ok(())
}
