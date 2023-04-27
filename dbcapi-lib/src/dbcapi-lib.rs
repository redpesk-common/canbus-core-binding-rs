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

extern crate libafb;
extern crate sockcan;
extern crate sockdata;

const MSG_DFT_RATE: u64 = 500;
const MSG_DFT_WATCHDOG: u64 = 10000;

// import libafb dependencies
use libafb::prelude::*;
use sockcan::prelude::*;
use sockdata::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

enum Action {
    SUBSCRIBE,
    UNSUBSCRIBE,
    READ,
    RESET,
}

struct PoolInfoCtx {
    stamp: u64,
    rate: u64,
    watchdog: u64,
    listeners: i32,
    flag: SubscribeFlag,
}

struct SigDataCtx {
    info: RefCell<PoolInfoCtx>,
    event: &'static AfbEvent,
}

struct SigPoolCtx {
    data: Rc<SigDataCtx>,
}

impl CanSigCtrl for SigPoolCtx {
    fn sig_notification(&self, sig: &dyn CanDbcSignal) -> i32 {
        let mut info = match self.data.info.try_borrow_mut() {
            Err(_) => {
                afb_log_msg!(
                    Critical,
                    self.data.event,
                    "pool-sig-notification: fail to get event info"
                );
                return -1;
            }
            Ok(info) => info,
        };

        let signal = DataBmcSig {
            name: sig.get_name().to_owned(),
            status: sig.get_status(),
            stamp: sig.get_stamp(),
            value: sig.get_value(),
        };

        // send event, count listener and update stamp
        let listeners = match sig.get_status() {
            CanDataStatus::Updated => {
                if (sig.get_stamp() - info.stamp) > info.rate * 1000 {
                    info.stamp = sig.get_stamp();
                    info.listeners = self.data.event.push(signal);
                };
                info.listeners
            }
            _ => {
                if (sig.get_stamp() - info.stamp) > info.watchdog * 1000
                    && info.flag == SubscribeFlag::ALL
                {
                    info.stamp = sig.get_stamp();
                    info.listeners = self.data.event.push(signal);
                };
                info.listeners
            }
        };
        listeners // return active listener to pool
    }
}

AfbVerbRegister!(SignalHandle, signal_vcb, SigVerbCtx);
struct SigVerbCtx {
    sig_rfc: Rc<RefCell<Box<dyn CanDbcSignal>>>,
    msg_rfc: Rc<RefCell<Box<dyn CanDbcMessage>>>,
    msg_ctx: Rc<MessageDataCtx>,
    data: Rc<SigDataCtx>,
}

fn signal_vcb(request: &AfbRequest, args: &AfbData, ctx: &mut SigVerbCtx) -> Result<(), AfbError> {
    let jquery = args.get::<JsoncObj>(0)?;
    let jaction = jquery.get::<String>("action")?;
    let action = match jaction.to_uppercase().as_str() {
        "SUBSCRIBE" => Action::SUBSCRIBE,
        "UNSUBSCRIBE" => Action::UNSUBSCRIBE,
        "READ" => Action::READ,
        "RESET" => Action::RESET,
        _ => {
            let error = AfbError::new("invalid-action", "expect: SUBSCRIBE|UNSUBSCRIBE|READ|RESET");
            return Err(error);
        }
    };

    // extract Signal from can pool ref cell
    let mut sig = match ctx.sig_rfc.try_borrow_mut() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-sig",
                "Internal pool error (sig rfc cell already used)",
            );
            return Err(afb_add_trace!(error));
        }
    };

    let msg = match ctx.msg_rfc.try_borrow() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-msg",
                "Internal pool error (msg rfc cell already used)",
            );
            return Err(afb_add_trace!(error));
        }
    };

    let mut msg_info = match ctx.msg_ctx.info.try_borrow_mut() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-info",
                "Internal pool error (msg info cell already used)",
            );
            return Err(afb_add_trace!(error));
        }
    };

    let mut sig_info = match ctx.data.info.try_borrow_mut() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-info",
                "Internal pool error (sig info cell already used)",
            );
            return Err(afb_add_trace!(error));
        }
    };

    match action {
        Action::SUBSCRIBE => {
            ctx.data.event.subscribe(request)?;
            let rate = match jquery.get::<u64>("rate") {
                Ok(value) => value,
                Err(_error) => msg_info.rate,
            };
            let watchdog = match jquery.get::<u64>("watchdog") {
                Ok(value) => value,
                Err(_error) => msg_info.watchdog,
            };
            let flag = match jquery.get::<String>("flag") {
                Ok(value) => match value.to_uppercase().as_str() {
                    "NEW" => SubscribeFlag::NEW,
                    "ALL" => SubscribeFlag::ALL,
                    _ => SubscribeFlag::NEW,
                },
                Err(_error) => msg_info.flag.clone(),
            };

            // if needed update signal subscription info
            if rate < sig_info.rate {
                sig_info.rate = rate
            }
            if watchdog < sig_info.watchdog {
                sig_info.watchdog = watchdog
            }

            // if needed update message subscription
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
            sig_info.listeners += 1; // we have at least one listener
            request.reply(
                format!(
                    "Subscribe (canid:{}) sig:{} OK",
                    msg.get_id(),
                    sig.get_name(),
                ),
                0,
            );
        }

        Action::UNSUBSCRIBE => {
            ctx.data.event.unsubscribe(request)?;
            request.reply(
                format!(
                    "UnSubscribe (canid:{}) sig:{} OK",
                    msg.get_id(),
                    sig.get_name(),
                ),
                0,
            );
        }

        Action::READ => {
            let sig_data = DataBmcSig {
                name: sig.get_name().to_owned(),
                stamp: sig.get_stamp(),
                status: sig.get_status(),
                value: sig.get_value(),
            };
            let mut params = AfbParams::new();
            params.push(sig_data)?;
            request.reply(params, 0);
        }

        Action::RESET => {
            sig.reset();
            request.reply(
                format!(
                    "Reset (canid:{}) sig:{} OK",
                    msg.get_id(),
                    sig.get_name(),
                ),
                0,
            );
        }
    };
    Ok(())
}

fn register_signal(
    _api: &mut AfbApi,
    _config: &SockBmcConfig,
    msg_ctx: &Rc<MessageDataCtx>,
    msg_rfc: &Rc<RefCell<Box<dyn CanDbcMessage>>>,
    sig_rfc: &Rc<RefCell<Box<dyn CanDbcSignal>>>,
) -> Result<(&'static AfbVerb, &'static AfbEvent), AfbError> {
    let mut sig_ref = match sig_rfc.try_borrow_mut() {
        Err(_) => return Err(AfbError::new("register-sig-fail", "internal pool error")),
        Ok(sig) => sig,
    };

    let sig_event = AfbEvent::new(sig_ref.get_name()).finalize()?;

    let info = PoolInfoCtx {
        rate: MSG_DFT_RATE,
        watchdog: MSG_DFT_WATCHDOG,
        stamp: 0,
        listeners: 0,
        flag: SubscribeFlag::NEW,
    };
    let sigdata = Rc::new(SigDataCtx {
        event: sig_event,
        info: RefCell::new(info),
    });

    sig_ref.set_callback(Box::new(SigPoolCtx {
        data: sigdata.clone(),
    }));

    let sig_verb = AfbVerb::new(sig_ref.get_name())
        .set_action("['reset','read','subscribe','unsubscribe']")?
        .set_sample("{'action':'subscribe','rate':250,'watchdog':5000,'flag':'all'}")?
        .set_callback(Box::new(SigVerbCtx {
            data: sigdata.clone(),
            sig_rfc: sig_rfc.clone(),
            msg_rfc: msg_rfc.clone(),
            msg_ctx: msg_ctx.clone(),
        }))
        .finalize()?;

    Ok((sig_verb, sig_event))
}

struct MessageDataCtx {
    info: RefCell<PoolInfoCtx>,
    event: &'static AfbEvent,
    bmc: &'static str,
}

struct MessageVerbCtx {
    msg_rfc: Rc<RefCell<Box<dyn CanDbcMessage>>>,
    data: Rc<MessageDataCtx>,
}
struct MessagePoolCtx {
    data: Rc<MessageDataCtx>,
}

// notification is called from msg pool when a message is updated
impl CanMsgCtrl for MessagePoolCtx {
    fn msg_notification(&self, msg: &dyn CanDbcMessage) {
        let msg_value = DataBcmMsg {
            canid: msg.get_id(),
            stamp: msg.get_stamp(),
            status: msg.get_status(),
        };

        let info = match self.data.info.try_borrow() {
            Err(_) => {
                afb_log_msg!(
                    Critical,
                    self.data.event,
                    "pool-msg-notification: fail to get event info"
                );
                return;
            }
            Ok(info) => info,
        };

        let params = |msg: &dyn CanDbcMessage| -> Result<AfbParams, AfbError> {
            let mut args = AfbParams::new();
            args.push(msg_value)?;

            for sig_rfc in msg.get_signals() {
                // extract Signal from can pool ref cell
                let sig = match sig_rfc.try_borrow() {
                    Ok(value) => value,
                    Err(_) => {
                        let error = AfbError::new(
                            "fail-borrow-sig",
                            "Internal pool error (sig rfc cell already used)",
                        );
                        return Err(error);
                    }
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
                    }
                    SubscribeFlag::ALL => {
                        args.push(sig_value)?;
                    }
                }
            }

            Ok(args)
        };

        let args = match params(msg) {
            Err(_) => {
                afb_log_msg!(
                    Critical,
                    self.data.event,
                    "pool-msg-notification: fail to build event params"
                );
                return;
            }
            Ok(value) => value,
        };

        // send event if no more active listener let's clear subscription
        let listener = self.data.event.push(args);
        if listener + msg.get_listeners() < 1 {
            afb_log_msg!(
                Notice,
                self.data.event,
                format!(
                    "msg-empty-listener: clearing canid={} subscription",
                    msg.get_id()
                )
            );

            // push event if no more listener clear canid subscription
            let _status = AfbSubCall::call_sync(
                self.data.event.get_apiv4(),
                self.data.bmc,
                "unsubscribe",
                UnSubscribeParam::new(vec![msg.get_id()]),
            );

            match self.data.info.try_borrow_mut() {
                Err(_) => {}
                Ok(mut info) => {
                    // reset message data context
                    info.stamp = 0;
                    info.rate = 0;
                    info.watchdog = 0;
                }
            }
        }
    }
}

AfbVerbRegister!(MessageHandle, message_vcb, MessageVerbCtx);
fn message_vcb(request: &AfbRequest, args: &AfbData, ctx: &MessageVerbCtx) -> Result<(), AfbError> {
    let jquery = args.get::<JsoncObj>(0)?;
    let jaction = jquery.get::<String>("action")?;
    let action = match jaction.to_uppercase().as_str() {
        "SUBSCRIBE" => Action::SUBSCRIBE,
        "UNSUBSCRIBE" => Action::UNSUBSCRIBE,
        "READ" => Action::READ,
        "RESET" => Action::RESET,
        _ => {
            let error = AfbError::new("invalid-action", "expect: SUBSCRIBE|UNSUBSCRIBE|READ|RESET");
            return Err(error);
        }
    };

    // extract message from can pool ref cell
    let mut msg = match ctx.msg_rfc.try_borrow_mut() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-msg",
                "Internal pool error (msg cell already used)",
            );
            return Err(afb_add_trace!(error));
        }
    };

    let mut msg_info = match ctx.data.info.try_borrow_mut() {
        Ok(value) => value,
        Err(_) => {
            let error = AfbError::new(
                "fail-borrow-info",
                "Internal pool error (info cell already used)",
            );
            return Err(afb_add_trace!(error));
        }
    };

    match action {
        Action::SUBSCRIBE => {
            ctx.data.event.subscribe(request)?;

            let rate = match jquery.get::<u64>("rate") {
                Ok(value) => value,
                Err(_error) => msg_info.rate,
            };
            let watchdog = match jquery.get::<u64>("watchdog") {
                Ok(value) => value,
                Err(_error) => msg_info.watchdog,
            };
            let flag = match jquery.get::<String>("flag") {
                Ok(value) => match value.to_uppercase().as_str() {
                    "NEW" => SubscribeFlag::NEW,
                    "ALL" => SubscribeFlag::ALL,
                    _ => SubscribeFlag::NEW,
                },
                Err(_error) => msg_info.flag.clone(),
            };

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
            request.reply(
                format!(
                    "Subscribe (canid:{}) msg:{} OK",
                    msg.get_id(),
                    msg.get_name(),
                ),
                0,
            );
        }

        Action::UNSUBSCRIBE => {
            ctx.data.event.unsubscribe(request)?;
            request.reply(
                format!(
                    "UnSubscribe (canid:{}) msg:{} OK",
                    msg.get_id(),
                    msg.get_name(),
                ),
                0,
            );
        }

        Action::READ => {
            let msg_data = DataBcmMsg {
                canid: msg.get_id(),
                stamp: msg.get_stamp(),
                status: msg.get_status(),
            };
            let mut params = AfbParams::new();
            params.push(msg_data)?;
            request.reply(params, 0);
        }

        Action::RESET => match msg.reset() {
            Err(_) => {
                return Err(AfbError::new(
                    "reset-msg-fail",
                    "internal pool (fail to get borrow mut)",
                ))
            }
            Ok(()) => {
                request.reply(
                    format!(
                    "Reset (canid:{}) msg:{} OK",
                    msg.get_id(),
                    msg.get_name(),
                    ),
                    0,
                );
            }
        },
    };
    Ok(())
}

struct SockBmcConfig {
    uid: &'static str,
    bmc: &'static str,
    evt: &'static str,
    jconf: JsoncObj,
    api: *const AfbApi,
}

fn register_msg(
    config: &SockBmcConfig,
    msg_rfc: &Rc<RefCell<Box<dyn CanDbcMessage>>>,
) -> Result<(), AfbError> {
    let msg_ref = msg_rfc.clone();
    let mut msg = match msg_ref.try_borrow_mut() {
        Err(_) => {
            return Err(AfbError::new(
                "register-msg-fail",
                "internal pool (fail to get borrow mut)",
            ))
        }
        Ok(msg) => msg,
    };

    let msg_name = msg.get_name();

    let mut msg_acls = AFB_NO_AUTH;
    let api = unsafe { &mut *(config.api as *const _ as *mut AfbApi) };

    // create a verb match message name
    let msg_verb = AfbVerb::new(msg_name);
    let mut info = PoolInfoCtx {
        rate: MSG_DFT_RATE,
        watchdog: MSG_DFT_WATCHDOG,
        stamp: 0,
        listeners: 0,
        flag: SubscribeFlag::NEW,
    };

    // search for complementary verb parameters
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

    // create message event and build verb userdata
    let event = AfbEvent::new(msg_name).finalize()?;
    let vcbdata = Rc::new(MessageDataCtx {
        bmc: config.bmc,
        event: event,
        info: RefCell::new(info),
    });

    msg.set_callback(Box::new(MessagePoolCtx {
        data: vcbdata.clone(),
    }));

    msg_verb
        .set_action("['reset','read','subscribe','unsubscribe']")?
        .set_sample("{'action':'subscribe','rate':250,'watchdog':5000,'flag':'new'}")?
        .set_callback(Box::new(MessageVerbCtx {
            msg_rfc: msg_rfc.clone(),
            data: vcbdata.clone(),
        }))
        .register(api.get_apiv4(), msg_acls);
    api.add_verb(msg_verb.finalize()?);

    // create signals verbs and group
    let mut group = AfbGroup::new(msg_name)
        .add_event(event)
        .set_info(to_static_str(format!("(canid:{})", msg.get_id())));

    for sig_rfc in msg.get_signals() {
        let (verb, event) = register_signal(api, config, &vcbdata, msg_rfc, sig_rfc)?;
        // add_verb borrows group but returns it (Rust works to keep developer live simple !!!)
        group = group.add_verb(verb);
        group = group.add_event(event);
    }
    group.register(api.get_apiv4(), msg_acls);
    api.add_group(group);

    Ok(())
}

struct EvtUserData {
    pool: &'static mut dyn CanDbcPool,
}
AfbEventRegister!(EventGetCtrl, bmc_event_cb, EvtUserData);
fn bmc_event_cb(event: &AfbEventMsg, args: &AfbData, ctx: &mut EvtUserData) {
    // get event data directly as SockBcmMsg
    let bmc_frame = match args.get::<&CanBmcData>(0) {
        Err(_) => {
            let error = AfbError::new(
                "event-bmc-invalid",
                "Internal error event not not SockBmcMsg type",
            );
            afb_log_msg!(Critical, event, &error);
            return;
        }
        Ok(value) => value,
    };

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
                format!("Fail to update message pool canid:{}", bmc_frame.get_id()),
            );
            afb_log_msg!(Critical, event, &error);
            return;
        }
        Ok(_msg) => {}
    };
}

pub fn create_pool_verbs(
    api: &AfbApi,
    jconf: JsoncObj,
    pool_box: Box<dyn CanDbcPool>,
) -> Result<(), AfbError> {
    // register data converter
    sockdata_register(api.get_apiv4())?;

    // open check sockbmc binding is alive
    let uid = to_static_str(jconf.get::<String>("uid")?);
    let bmc = to_static_str(jconf.get::<String>("sock_api")?);
    let evt = to_static_str(jconf.get::<String>("sock_evt")?);
    AfbSubCall::call_sync(api, bmc, "check_sock", 0)?;

    // lock sockcan message pool in memory
    let pool = Box::leak(pool_box);

    let bmc_config = SockBmcConfig {
        uid: uid,
        api: api,
        bmc: bmc,
        evt: evt,
        jconf: jconf,
    };

    // loop on message pool and create verbs
    for msg in pool.get_messages() {
        register_msg(&bmc_config, msg)?
    }

    // add event handler to get messages
    let pattern = to_static_str(format!("{}/{}", bmc, evt));
    let evt_handler = AfbEvtHandler::new(uid)
        .set_info("Receive low level BMC data frame")
        .set_pattern(pattern)
        .set_callback(Box::new(EventGetCtrl { pool: pool }));
    evt_handler.register(api.get_apiv4());
    evt_handler.finalize()?;

    // force api update
    let api = unsafe { &mut *(api as *const _ as *mut AfbApi) };
    api.add_evt_handler(evt_handler);

    Ok(())
}
