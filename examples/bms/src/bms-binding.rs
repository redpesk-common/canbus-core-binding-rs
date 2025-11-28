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

use dbcapi::create_pool_verbs;
use sockdata::types::parse_sockcan_config;

// bring in the generated DBC message pool for this example
include!("./__bms-dbcgen.rs");
use crate::DbcBms::CanMsgPool;

use afbv4::prelude::*;

/// Binding entry point.
/// Runs when the shared object is loaded; create and register the API here.
///
/// Order matters:
/// 1) Build the API descriptor and declare dependencies
/// 2) **Finalize** to obtain a valid apiv4 handle (non-NULL)
/// 3) Register verbs/events using the finalized handle
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    // parse all config fields from JSON into a single struct
    let config = parse_sockcan_config(&jconf);

    // Build the API descriptor:
    // - set public name
    // - attach info and permissions
    // - keep unsealed so we can still register things
    // - declare dependency on the sockcan service
    let can_api = AfbApi::new(config.api_uid)
        .set_info(config.info)
        .set_permission(AfbPermission::new(to_static_str(config.acls.to_owned())))
        .seal(false)
        .require_api(config.sock_api);

    let pool = Box::new(CanMsgPool::new(config.api_uid));
    create_pool_verbs(rootv4, can_api, jconf, pool)?;
    can_api.finalize()
}

// Register the binding entry point with libafb.
AfbBindingRegister!(binding_init);
