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
extern crate dbcapi;

// include generated dbc message pool
include!("./__bms-dbcgen.rs");
use crate::DbcBms::*;

use libafb::prelude::*;
use sockdata::prelude::*;
use crate::dbcapi::*;

pub struct ApiUserData {
    pub uid: &'static str,
    pub canapi: &'static str,
    pub candev: &'static str,
}

impl AfbApiControls for ApiUserData {
    fn config(&mut self, api: &AfbApi, jconf: JsoncObj) -> Result<(),AfbError> {
        afb_log_msg!(Debug, api, "api={} config={}", api.get_uid(), jconf);
        Ok(())
    }

    // mandatory for downcasting back to custom api data object
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// Binding init callback started at binding load time before any API exist
// -----------------------------------------
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info,rootv4, "config:{}",jconf);
    let candev = if let Ok(value) = jconf.get::<String>("dev") {
        to_static_str(value)
    } else {
        "vcan0"
    };

    let dbc_uid = if let Ok(value) = jconf.get::<String>("uid") {
        to_static_str(value)
    } else {
        "sockcan"
    };

    let dbc_api = if let Ok(value) = jconf.get::<String>("dbc_api") {
        to_static_str(value)
    } else {
        dbc_uid.clone()
    };

    let bmc_api = if let Ok(value) = jconf.get::<String>("sock_api") {
        to_static_str(value)
    } else {
        "sockcan"
    };

    let info = if let Ok(value) = jconf.get::<String>("info") {
        to_static_str(value)
    } else {
        ""
    };

    let acls = if let Ok(value) = jconf.get::<String>("acls") {
        to_static_str(value)
    } else {
        "acl:sockcan"
    };

    let api_usrdata = ApiUserData {
        uid: dbc_uid,
        candev: candev,
        canapi: dbc_api.clone(),
    };

    // create a new api
    let can_api = AfbApi::new(dbc_uid)
        .set_name(dbc_api)
        .set_info(info)
        .set_permission(AfbPermission::new(acls))
        .set_callback(Box::new(api_usrdata))
        .require_api(bmc_api)
        .seal(false)
        .finalize() ?;

    // open dbc can message pool and create one verb per message/signal
    let pool = Box::new(CanMsgPool::new(dbc_uid));
    create_pool_verbs(can_api, jconf, pool) ?;

    // finalize api
    Ok(can_api)
}

// register binding within libafb
AfbBindingRegister!(binding_init);
