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

// Import libafb dependencies and serde for JSON (de)serialization.
use afbv4::prelude::*;
use serde::{Deserialize, Serialize};

use sockcan::prelude::{CanBcmOpCode, CanDataStatus, CanDbcType};

// Automatically generate JSON encoder/decoder and AFB registration glue
// for the following data types via the `AfbDataConverter!` macro.
AfbDataConverter!(bmc_error, CanBmcError);

/// High-level error wrapper for BCM-related failures exposed to AFB clients.
///
/// Fields:
/// - `uid`: string identifier for the error source/category,
/// - `status`: numeric status code,
/// - `info`: human-readable error description.
///
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CanBmcError {
    uid: String,
    status: i32,
    info: String,
}

impl CanBmcError {
    /// Construct a new `CanBmcError` with the given identifier, status code and message.
    pub fn new(uid: String, status: i32, info: String) -> Self {
        CanBmcError { uid, status, info }
    }

    /// Return the error identifier.
    pub fn get_uid(&self) -> String {
        self.uid.clone()
    }

    /// Return the numeric status code of this error.
    pub fn status(&self) -> i32 {
        self.status
    }

    /// Return the human-readable error description.
    pub fn info(&self) -> String {
        self.info.clone()
    }
}

AfbDataConverter!(bmc_data, CanBmcData);

/// High-level representation of a BCM CAN frame that travels through the AFB API.
///
/// This structure is used as the payload of BCM-related events and verb replies.
#[derive(Serialize, Deserialize, Debug)]
pub struct CanBmcData {
    pub canid: u32,
    pub len: u8,
    pub stamp: u64,
    pub opcode: CanBcmOpCode,
    pub data: Vec<u8>,
}

AfbDataConverter!(bmc_msg, DataBcmMsg);

/// Short BCM message metadata used for certain notifications and logging.
///
#[derive(Serialize, Deserialize, Debug)]
pub struct DataBcmMsg {
    pub canid: u32,
    pub stamp: u64,
    pub status: CanBcmOpCode,
}

AfbDataConverter!(bmc_sig, DataBmcSig);

/// Snapshot of a decoded DBC signal used as event payload.
///
/// Fields:
/// - `name`: DBC signal name,
/// - `stamp`: timestamp of the last update,
/// - `status`: data status (updated, timeout, invalid, etc.),
/// - `value`: decoded value with the correct DBC type.
#[derive(Serialize, Deserialize, Debug)]
pub struct DataBmcSig {
    pub name: String,
    pub stamp: u64,
    pub status: CanDataStatus,
    pub value: CanDbcType,
}

impl CanBmcData {
    /// Construct a new BCM data record from low-level CAN parameters.
    pub fn new(canid: u32, opcode: CanBcmOpCode, stamp: u64, data: Vec<u8>, len: u8) -> Self {
        CanBmcData { canid, len, stamp, opcode, data }
    }

    /// Return the DLC (data length) of the CAN frame.
    pub fn get_len(&self) -> u8 {
        self.len
    }

    /// Return the timestamp at which this frame was captured.
    pub fn get_stamp(&self) -> u64 {
        self.stamp
    }

    /// Return the CAN identifier of this frame.
    pub fn get_id(&self) -> u32 {
        self.canid
    }

    /// Return the BCM opcode associated with this frame.
    pub fn get_opcode(&self) -> CanBcmOpCode {
        self.opcode
    }

    /// Return a reference to the raw CAN payload bytes.
    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}

AfbDataConverter!(subscribe_flag, SubscribeFlag);

/// Subscription mode for BCM CAN notifications.
///
/// `NEW` – forward only new data updates.
///
/// `ALL` – forward every notification, including periodic/watchdog events.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SubscribeFlag {
    NEW,
    ALL,
}

AfbDataConverter!(subscribe_param, SubscribeParam);

/// Parameters used when subscribing to BCM CAN IDs.
///
/// Fields:
/// - `rate`: minimum interval between notifications (time unit is binding-specific),
/// - `watchdog`: maximum allowed idle time before a timeout is reported,
/// - `canids`: list of CAN IDs to subscribe to,
/// - `flag`: controls which updates are delivered (new-only vs all).
///
#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeParam {
    rate: u64,
    watchdog: u64,
    canids: Vec<u32>,
    flag: SubscribeFlag,
}
impl SubscribeParam {
    /// Create a new subscription parameter set for the given CAN IDs and timer configuration.
    pub fn new(canids: Vec<u32>, watchdog: u64, rate: u64, flag: SubscribeFlag) -> Self {
        SubscribeParam { rate, watchdog, canids, flag }
    }

    /// Create a new subscription parameter set for the given CAN IDs and timer configuration.
    pub fn get_rate(&self) -> u64 {
        self.rate
    }

    /// Return the configured watchdog timeout.
    pub fn get_watchdog(&self) -> u64 {
        self.watchdog
    }

    /// Return the list of CAN IDs to subscribe to.
    pub fn get_canids(&self) -> &Vec<u32> {
        &self.canids
    }
}

AfbDataConverter!(unsubscribe_param, UnSubscribeParam);

/// Parameters used when unsubscribing from BCM CAN IDs.
///
/// Field:
/// - `canids`: list of CAN IDs to remove from the subscription.
#[derive(Serialize, Deserialize, Debug)]
pub struct UnSubscribeParam {
    canids: Vec<u32>,
}
impl UnSubscribeParam {
    /// Create a new unsubscription parameter set for the given CAN IDs.
    pub fn new(canids: Vec<u32>) -> Self {
        UnSubscribeParam { canids }
    }

    /// Return the list of CAN IDs to unsubscribe from.
    pub fn get_canids(&self) -> &Vec<u32> {
        &self.canids
    }
}

// Register custom data types within the AFB binder.
//
// This function must be called during binding initialization so the framework
// knows how to (de)serialize these types in requests, replies and events.
pub fn sockdata_register(_root: AfbApiV4) -> Result<(), AfbError> {
    // Custom types should be registered at binding startup time.
    bmc_error::register()?;
    bmc_data::register()?;
    bmc_sig::register()?;
    bmc_msg::register()?;
    subscribe_param::register()?;
    subscribe_flag::register()?;
    unsubscribe_param::register()?;
    Ok(())
}

/// Static configuration for the sockcan binding, parsed once from the JSON binding config.
///
/// All fields are `'static` string slices, typically created using `to_static_str`,
/// and are expected to live for the entire process lifetime.
///
/// Fields:
/// - `api_uid`: public API identifier for this binding,
/// - `event_uid`: event name used to emit BCM notifications,
/// - `can_device`: CAN interface name (e.g. "can0", "vcan0"),
/// - `sock_api`: name of the underlying sockcan service API,
/// - `info`: human-readable API description,
/// - `acls`: ACL expression required to access the API.
///
pub struct SockcanBindingConfig {
    pub api_uid: &'static str,
    pub event_uid: &'static str,
    pub can_device: &'static str,
    pub sock_api: &'static str,
    pub info: &'static str,
    pub acls: &'static str,
}

/// Parse the JSON configuration object into a `SockcanBindingConfig`.
///
/// Supported JSON keys and defaults:
/// - `"dev"`       → `can_device`, default: `"vcan0"`
/// - `"uid"`       → `api_uid`, default: `"sockcan"`
/// - `"sock_api"`  → `sock_api`, default: `api_uid`
/// - `"info"`      → `info`, default: `""`
/// - `"event_uid"` → `event_uid`, default: `"sockbmc"`
/// - `"acls"`      → `acls`, default: `"acl:sockcan"`
///
/// All string values are converted to `'static` with `to_static_str`.
///
pub fn parse_sockcan_config(jconf: &JsoncObj) -> SockcanBindingConfig {
    let can_device =
        if let Ok(value) = jconf.get::<String>("dev") { to_static_str(value) } else { "vcan0" };

    let api_uid =
        if let Ok(value) = jconf.get::<String>("uid") { to_static_str(value) } else { "sockcan" };

    let sock_api = if let Ok(value) = jconf.get::<String>("sock_api") {
        to_static_str(value)
    } else {
        api_uid
    };

    let info = if let Ok(value) = jconf.get::<String>("info") { to_static_str(value) } else { "" };

    let event_uid = if let Ok(value) = jconf.get::<String>("event_uid") {
        to_static_str(value)
    } else {
        "sockbmc"
    };

    let acls = if let Ok(value) = jconf.get::<String>("acls") {
        to_static_str(value)
    } else {
        "acl:sockcan"
    };

    SockcanBindingConfig { api_uid, event_uid, can_device, sock_api, info, acls }
}
