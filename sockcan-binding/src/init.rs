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
 *  and conditsockdata_registerions see https://www.iot.bzh/terms-conditions. For further
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

use crate::*;
use afbv4::prelude::*;
use sockdata::types::parse_sockcan_config;
use sockdata::types::sockdata_register;

/// Binding initialization callback invoked when the shared object is loaded by libafb.
///
/// Responsibilities:
/// - log and parse the JSON configuration object,
/// - register data converters (sockdata) with the AFB root API,
/// - create the CAN-related API (name, permissions, metadata),
/// - register verbs and events using the parsed configuration,
/// - finalize and return a `'static` reference to the API.
///
/// The `rootv4` handle represents the root AFB API context used to register
/// new APIs and global resources.
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    // Log the raw configuration for traceability and debugging.
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    // Parse all configuration fields from JSON into a strongly-typed configuration structure.
    let config = parse_sockcan_config(&jconf);

    // Register data converters (sockdata) with the AFB root context so that CAN-related
    // payloads can be automatically mapped between wire representation and Rust structs.
    sockdata_register(rootv4)?;

    // Create a new AFB API instance for this binding:
    // - `api_uid` controls the public API name,
    // - `info` is a human-readable description,
    // - `acls` defines permission requirements for calling this API.
    //
    // `seal(false)` keeps the API mutable so verbs and events can still be registered
    // before finalization.
    let api = AfbApi::new(config.api_uid)
        .set_info(config.info)
        .set_permission(AfbPermission::new(to_static_str(config.acls.to_owned())))
        .seal(false);

    // Register all verbs and events associated with this binding using the parsed configuration.
    // The `verbs::register` helper is responsible for:
    // - creating verb handlers for subscription, control, diagnostics, etc.,
    // - wiring CAN and DBC-related events,
    // - attaching any necessary per-verb context.
    // It receives the API instance and the configuration structure.
    verbs::register(api, &config)?;

    // Finalize the API so it becomes visible/usable to clients.
    // After this call, the API descriptor is no longer mutable.
    api.finalize()
}

// Register the binding entry point with libafb.
//
// This macro exposes `binding_init` as the symbol that libafb looks up and calls
// when loading this shared object as a binding plugin.
AfbBindingRegister!(binding_init);
