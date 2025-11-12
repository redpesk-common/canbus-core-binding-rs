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

use std::any::Any; // needed for AfbApiControls::as_any

// bring in the generated DBC message pool for this example
include!("./__bms-dbcgen.rs");
use crate::DbcBms::*;

use crate::dbcapi::*;
use afbv4::prelude::*;

/// Per-API user data carried through lifecycle hooks (optional).
pub struct ApiUserData {
    pub uid: &'static str,
    pub canapi: &'static str,
    pub candev: &'static str,
}

impl AfbApiControls for ApiUserData {
    /// Called when the binder applies configuration to the API.
    /// `jconf` is the JSON fragment passed at load time (or its `args` part depending on setup).
    fn config(&mut self, api: &AfbApi, jconf: JsoncObj) -> Result<(), AfbError> {
        afb_log_msg!(Debug, api, "api={} config={}", api.get_uid(), jconf);
        Ok(())
    }

    /// Required so callers can downcast back to this concrete type.
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// Binding entry point.
/// Runs when the shared object is loaded; create and register the API here.
///
/// Order matters:
/// 1) Build the API descriptor and declare dependencies
/// 2) **Finalize** to obtain a valid apiv4 handle (non-NULL)
/// 3) Register verbs/events using the finalized handle
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    // Optional CAN device name (defaults to vcan0 for development).
    let candev = if let Ok(value) = jconf.get::<String>("dev") {
        to_static_str(value)
    } else {
        "vcan0"
    };

    // Public API uid (fallback: "sockcan").
    let dbc_uid = if let Ok(value) = jconf.get::<String>("uid") {
        to_static_str(value)
    } else {
        "sockcan"
    };

    // Name of the DBC-facing API (default to same as uid).
    let dbc_api = if let Ok(value) = jconf.get::<String>("dbc_api") {
        to_static_str(value)
    } else {
        dbc_uid
    };

    // Lower-level CAN service dependency (the sockcan binding).
    let bmc_api = if let Ok(value) = jconf.get::<String>("sock_api") {
        to_static_str(value)
    } else {
        "sockcan"
    };

    // Optional human-readable info text.
    let info = if let Ok(value) = jconf.get::<String>("info") {
        to_static_str(value)
    } else {
        ""
    };

    // Optional ACL/permission string.
    let acls = if let Ok(value) = jconf.get::<String>("acls") {
        to_static_str(value)
    } else {
        "acl:sockcan"
    };

    // Example userdata (attach if you need it in hooks).
    let _api_usrdata = ApiUserData {
        uid: dbc_uid,
        candev,
        canapi: dbc_api,
    };

    // Build the API descriptor:
    // - set public name
    // - attach info and permissions
    // - keep unsealed so we can still register things
    // - declare dependency on the sockcan service
    let can_api = AfbApi::new(dbc_uid)
        .set_info(info)
        .set_permission(AfbPermission::new(to_static_str(acls.to_owned())))
        .seal(false)
        .require_api(bmc_api);

    // 1) finalize to obtain a valid apiv4 handle
    let api = can_api.finalize()?;

    // 2) create/register verbs & events (requires finalized API)
    let pool = Box::new(CanMsgPool::new(dbc_uid));
    create_pool_verbs(api, jconf, pool)?;

    // 3) return the finalized API to the binder
    Ok(api)
}

// Register the binding entry point with libafb.
AfbBindingRegister!(binding_init);
