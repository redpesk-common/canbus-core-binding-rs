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

//!  CAN DBC binding for libafb/redpesk.
//!
//! This binding:
//! - loads configuration from JSON (sockcan/bus parameters, ACLs, API name),
//! - instantiates a DBC-generated message pool for car CAN network,
//! - exposes verbs/events for CAN messages and signals via `create_pool_verbs`,
//! - registers a `binding_init` entry point used by libafb at load time.
#![doc(
    html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png",
    html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico"
)]

use afbv4::prelude::*;
// Import helper that creates verbs/events from a DBC pool.
use dbcapi::create_pool_verbs;
// Import parser for the JSON configuration describing sockcan and API parameters.
use sockdata::types::parse_sockcan_config;

// Include generated DBC message pool for the Tesla Model 3.
// include generated dbc message pool
include!("./__bms-dbcgen.rs");
use crate::DbcBms::CanMsgPool;

/// Binding entry point.
/// Runs when the shared object is loaded; create and register the API here.
///
/// Order matters:
/// 1) Build the API descriptor and declare dependencies
/// 2) **Finalize** to obtain a valid apiv4 handle (non-NULL)
/// 3) Register verbs/events using the finalized handle
/// 4) Return the finalized API handle to libafb
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    // Log raw configuration for traceability; be careful with potential sensitive data
    // (ACLs, bus names, credentials) and ensure log level is appropriate.
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    // Parse and validate JSON configuration into a strongly-typed structure.
    let config = parse_sockcan_config(&jconf);

    // create a new api
    // Create and configure the public API:
    // - set API identifier and human-readable info,
    // - attach permission/ACL information,
    // - keep the API unsealed while verbs/events are being added.
    let can_api = AfbApi::new(config.api_uid)
        .set_info(config.info)
        .set_permission(AfbPermission::new(to_static_str(config.acls.to_owned())))
        .seal(false)
        .require_api(config.sock_api);

    // Instantiate the DBC message pool for car CAN network,
    // and register verbs/events for each message/signal defined in the DBC.
    let pool = Box::new(CanMsgPool::new(config.api_uid));

    // Create verbs and events from the DBC pool and register them on the API.
    create_pool_verbs(rootv4, can_api, jconf, pool)?;
    // Finalize the API so it becomes visible/usable by clients.
    // After this call the API descriptor is no longer mutable.
    can_api.finalize()
}

// Register the binding entry point with libafb.
// libafb will call `binding_init` during module load to initialize the API.
// register binding within libafb
AfbBindingRegister!(binding_init);
