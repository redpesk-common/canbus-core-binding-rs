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

// import libafb dependencies
use afbv4::prelude::*;
use serde::{Deserialize, Serialize};
use sockcan::prelude::*;

// automatically generate json encoder/decoder for MySimpleData
AfbDataConverter!(bmc_error, CanBmcError);
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CanBmcError {
    uid: String,
    status: i32,
    info: String,
}

impl CanBmcError {
    pub fn new(uid: String, status: i32, info: String) -> Self {
        CanBmcError { uid, status, info }
    }
    pub fn get_uid(&self) -> String {
        self.uid.clone()
    }
    pub fn status(&self) -> i32 {
        self.status
    }
    pub fn info(&self) -> String {
        self.info.clone()
    }
}

AfbDataConverter!(bmc_data, CanBmcData);
#[derive(Serialize, Deserialize, Debug)]
pub struct CanBmcData {
    pub canid: u32,
    pub len: u8,
    pub stamp: u64,
    pub opcode: CanBcmOpCode,
    pub data: Vec<u8>,
}

AfbDataConverter!(bmc_msg, DataBcmMsg);
#[derive(Serialize, Deserialize, Debug)]
pub struct DataBcmMsg {
    pub canid: u32,
    pub stamp: u64,
    pub status: CanBcmOpCode,
}

AfbDataConverter!(bmc_sig, DataBmcSig);
#[derive(Serialize, Deserialize, Debug)]
pub struct DataBmcSig {
    pub name: String,
    pub stamp: u64,
    pub status: CanDataStatus,
    pub value: CanDbcType,
}

impl CanBmcData {
    pub fn new(canid: u32, opcode: CanBcmOpCode, stamp: u64, data: Vec<u8>, len: u8) -> Self {
        CanBmcData { canid, len, stamp, opcode, data }
    }
    pub fn get_len(&self) -> u8 {
        self.len
    }
    pub fn get_stamp(&self) -> u64 {
        self.stamp
    }
    pub fn get_id(&self) -> u32 {
        self.canid
    }
    pub fn get_opcode(&self) -> CanBcmOpCode {
        self.opcode
    }
    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}

AfbDataConverter!(subscribe_flag, SubscribeFlag);
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum SubscribeFlag {
    NEW,
    ALL,
}

AfbDataConverter!(subscribe_param, SubscribeParam);
#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribeParam {
    rate: u64,
    watchdog: u64,
    canids: Vec<u32>,
    flag: SubscribeFlag,
}
impl SubscribeParam {
    pub fn new(canids: Vec<u32>, watchdog: u64, rate: u64, flag: SubscribeFlag) -> Self {
        SubscribeParam { rate, watchdog, canids, flag }
    }
    pub fn get_rate(&self) -> u64 {
        self.rate
    }
    pub fn get_watchdog(&self) -> u64 {
        self.watchdog
    }
    pub fn get_canids(&self) -> &Vec<u32> {
        &self.canids
    }
}

AfbDataConverter!(unsubscribe_param, UnSubscribeParam);
#[derive(Serialize, Deserialize, Debug)]
pub struct UnSubscribeParam {
    canids: Vec<u32>,
}
impl UnSubscribeParam {
    pub fn new(canids: Vec<u32>) -> Self {
        UnSubscribeParam { canids }
    }
    pub fn get_canids(&self) -> &Vec<u32> {
        &self.canids
    }
}

// register data type within afb_binder
pub fn sockdata_register(_root: AfbApiV4) -> Result<(), AfbError> {
    // Custom type should be registered at binding startup time
    bmc_error::register()?;
    bmc_data::register()?;
    bmc_sig::register()?;
    bmc_msg::register()?;
    subscribe_param::register()?;
    subscribe_flag::register()?;
    unsubscribe_param::register()?;
    Ok(())
}
