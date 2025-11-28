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

pub fn register(api: &mut AfbApi, config: &SockcanBindingConfig) -> Result<(), AfbError> {
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

    let unsubscribe = AfbVerb::new("unsubscribe")
        .set_callback(unsubscribe_cb)
        .set_info("Unsubscribe socket BMC cannids from session")
        .set_usage("{'canids':[x,y,...,z]}")
        .add_sample("{'canids':[266,257,599]}")?
        .finalize()?;
    api.add_verb(unsubscribe);

    let check = AfbVerb::new("check")
        .set_callback(check_cb)
        .set_context(CheckCtx { candev: config.can_device })
        .set_info("Check socket BMC is available")
        .set_usage("no-input")
        .finalize()?;
    api.add_verb(check);

    let close = AfbVerb::new("close")
        .set_callback(close_cb)
        .set_info("Close socket BMC session")
        .set_usage("no-input")
        .finalize()?;
    api.add_verb(close);

    Ok(())
}
