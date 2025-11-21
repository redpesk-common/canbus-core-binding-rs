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
extern crate dbcapi;
extern crate sockcan;

// include generated dbc message pool
include!("./__model3-dbcgen.rs");
use crate::DbcModel3::*;

use crate::dbcapi::create_pool_verbs;
use afbv4::prelude::*;

pub struct ApiUserData {
    pub uid: &'static str,
    pub canapi: &'static str,
    pub candev: &'static str,
}

impl AfbApiControls for ApiUserData {
    fn config(&mut self, api: &AfbApi, jconf: JsoncObj) -> Result<(), AfbError> {
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
    afb_log_msg!(Info, rootv4, "config:{}", jconf);
    let candev =
        if let Ok(value) = jconf.get::<String>("dev") { to_static_str(value) } else { "vcan0" };

    let dbc_uid =
        if let Ok(value) = jconf.get::<String>("uid") { to_static_str(value) } else { "sockcan" };

    let dbc_api =
        if let Ok(value) = jconf.get::<String>("dbc_api") { to_static_str(value) } else { dbc_uid };

    let _bmc_api = if let Ok(value) = jconf.get::<String>("sock_api") {
        to_static_str(value)
    } else {
        "sockcan"
    };

    let info = if let Ok(value) = jconf.get::<String>("info") { to_static_str(value) } else { "" };

    let acls = if let Ok(value) = jconf.get::<String>("acls") {
        to_static_str(value)
    } else {
        "acl:sockcan"
    };

    let _api_usrdata = ApiUserData { uid: dbc_uid, candev, canapi: dbc_api };

    // create a new api
    let can_api = AfbApi::new(dbc_uid)
        .set_info(info)
        .set_permission(AfbPermission::new(to_static_str(acls.to_owned())))
        .seal(false);

    // open dbc can message pool and create one verb per message/signal
    let pool = Box::new(CanMsgPool::new(dbc_uid));
    create_pool_verbs(rootv4, can_api, jconf, pool).expect("create_pool_verbs failed");

    can_api.finalize()
}

// register binding within libafb
AfbBindingRegister!(binding_init);
