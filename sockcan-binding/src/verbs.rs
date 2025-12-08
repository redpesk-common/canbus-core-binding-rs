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

use crate::callbacks::{check_cb, close_cb, subscribe_cb, unsubscribe_cb};
use crate::context::{CheckCtx, SubVerbCtx};
use afbv4::prelude::*;
use sockdata::types::SockcanBindingConfig;

// ============ Register Canids ===============

/// Register all verbs exposed by this CAN binding on the given API.
///
/// This function wires the high-level verbs to their concrete callbacks and
/// attaches verb-specific context:
/// - `subscribe`: create/attach a BCM session and install RX filters for CAN IDs,
/// - `unsubscribe`: remove BCM filters for CAN IDs on the current session,
/// - `check`: health-check that BCM is available on the target CAN device,
/// - `close`: explicitly close the BCM session and release related resources.
///
/// The `config` parameter provides binding-level configuration:
/// - `api_uid`: logical API identifier,
/// - `event_uid`: event name used for BCM notifications,
/// - `can_device`: CAN interface name (e.g. "can0").
///
pub fn register(api: &mut AfbApi, config: &SockcanBindingConfig) -> Result<(), AfbError> {
    // Verb: subscribe
    //
    // Subscribes the caller to a set of CAN IDs via BCM, using optional
    // rate/watchdog/flag parameters to control timers and notification policy.
    //
    // Usage and samples are expressed as JSON-like strings; the underlying framework
    // uses them for introspection and API documentation.
    let subscribe = AfbVerb::new("subscribe")
        .set_callback(subscribe_cb)
        .set_context(SubVerbCtx {
            uid: config.api_uid,
            sockevt: config.event_uid,
            candev: config.can_device,
        })
        .set_info("Subscribe a canid array")
        .set_usage("{'canids':[x,y,...,z],['rate':xx_ms],['watchdog':xx_ms],['flag':'ALL|NEW']}")
        .add_sample("{'canids':[266,257,599],'rate':250,'watchdog':1000,'flag':'ALL'}")?
        .finalize()?;
    api.add_verb(subscribe);

    // Verb: unsubscribe
    //
    // Unsubscribes the caller from BCM notifications for the given CAN IDs
    // on the current session.
    let unsubscribe = AfbVerb::new("unsubscribe")
        .set_callback(unsubscribe_cb)
        .set_info("Unsubscribe socket BMC cannids from session")
        .set_usage("{'canids':[x,y,...,z]}")
        .add_sample("{'canids':[266,257,599]}")?
        .finalize()?;
    api.add_verb(unsubscribe);

    // Verb: check
    //
    // Performs a health check to ensure that a BCM socket can be opened on
    // the configured CAN device. It does not change any persistent state.
    let check = AfbVerb::new("check")
        .set_callback(check_cb)
        .set_context(CheckCtx { candev: config.can_device })
        .set_info("Check socket BMC is available")
        .set_usage("no-input")
        .finalize()?;
    api.add_verb(check);

    // Verb: close
    //
    // Explicitly closes the BCM session associated with the current request/session,
    // unreferences the AFB event and closes the underlying socket.
    let close = AfbVerb::new("close")
        .set_callback(close_cb)
        .set_info("Close socket BMC session")
        .set_usage("no-input")
        .finalize()?;
    api.add_verb(close);

    Ok(())
}
