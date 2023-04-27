    
    // -----------------------------------------------------------------------
    //              <- DBC file Rust mapping ->
    // -----------------------------------------------------------------------
    //  Do not exit this file it will be regenerated automatically by cargo.
    //  Check:
    //   - build.rs at project root for dynamically mapping
    //   - example/demo/dbc-log/??? for static values
    //  Reference: iot.bzh/Redpesk canbus-rs code generator
    // -----------------------------------------------------------------------
    
// --------------------------------------------------------------
//       WARNING: Manual modification will be destroyed
// --------------------------------------------------------------
// - code generated from ./dbc-log/model3can.dbc (Thu Apr 27 14:38:30 2023)
// - update only with [dbc-parser|build.rs::DbcParser]
// - source code: https://github.com/redpesk-labs/canbus-rs
// - (C)IoT.bzh(2023), Author: Fulup Ar Foll, http://redpesk.bzh
// - License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
// -------------------------------------------------------------
mod DbcModel3 {
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
extern crate serde;
extern crate bitvec;
use sockcan::prelude::*;
use std::cell::{RefCell,RefMut};
use std::rc::{Rc};

/// ID118DriveSystemStatus Message
/// - ID: 280 (0x118)
/// - Size: 8 bytes
/// - Transmitter: VehicleBus
pub mod Id118DriveSystemStatus { /// Message name space
    use sockcan::prelude::*;
    use bitvec::prelude::*;
    use std::any::Any;
    use std::cell::{RefCell};
    use std::rc::Rc;

    use std::fmt;

    use serde::{Deserialize, Serialize};
    pub enum DbcSignal {
        DiAccelPedalPos,
        DiBrakePedalState,
        DiDriveBlocked,
        DiEpbRequest,
        DiGear,
        DiImmobilizerState,
        DiKeepDrivePowerStateRequest,
        DiProximity,
        DiRegenLight,
        DiSystemState,
        DiSystemStatusChecksum,
        DiSystemStatusCounter,
        DiTrackModeState,
        DiTractionControlMode,
    }

    /// Id118DriveSystemStatus::DiAccelPedalPos
    ///
    /// Pedal Position
    /// - Min: 0
    /// - Max: 100
    /// - Unit: "%"
    /// - Receivers: Receiver
    /// - Start bit: 32
    /// - Signal size: 8 bits
    /// - Factor: 0.4
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiAccelPedalPos {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: f64,
    }

    // DBC definition for MsgID:280 Signal:DI_accelPedalPos
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiAccelPedalPos {
        Sna,
        _Other(f64),
    }

    impl From<DbcDiAccelPedalPos> for f64 {
        fn from (val: DbcDiAccelPedalPos) -> f64 {
            match val {
                DbcDiAccelPedalPos::Sna => panic! ("(Hoops) impossible conversion 255_f64 -> f64"),
                DbcDiAccelPedalPos::_Other(x) => x
            }
        }
    }

    impl DiAccelPedalPos  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiAccelPedalPos {
                status: CanDataStatus::Unset,
                name:"DiAccelPedalPos",
                value: 0_f64,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_f64;
        }

        pub fn get_as_def (&self) -> DbcDiAccelPedalPos {
                DbcDiAccelPedalPos::_Other(self.get_typed_value())
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiAccelPedalPos, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiAccelPedalPos::Sna => Err(CanError::new("not-in-range","(Sna) !!! 255(f64) not in [0..100] range")),
                DbcDiAccelPedalPos::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> f64 {
            self.value
        }

        fn set_typed_value(&mut self, value:f64, data:&mut [u8]) -> Result<(),CanError> {
            if value < 0_f64 || 100_f64 < value {
                return Err(CanError::new("invalid-signal-value",format!("value={} not in [0..100]",value)));
            }
            let factor = 0.4_f64;
            let offset = 0_f64;
            let value = ((value - offset) / factor) as u8;
            data.view_bits_mut::<Lsb0>()[32..40].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiAccelPedalPos impl end

    impl fmt::Display for DiAccelPedalPos {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiAccelPedalPos:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiAccelPedalPos {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiAccelPedalPos")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiAccelPedalPos public api (CanDbcSignal trait)
    impl CanDbcSignal for DiAccelPedalPos {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[32..40].load_le::<u8>();
                    let factor = 0.4_f64;
                    let offset = 0_f64;
                    let newval= (value as f64) * factor + offset;
                    if newval != self.value {
                        self.value= newval;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:f64= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::F64(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiAccelPedalPos public api

    /// Id118DriveSystemStatus::DiBrakePedalState
    ///
    /// Brake Pedal
    /// - Min: 0
    /// - Max: 2
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 19
    /// - Signal size: 2 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiBrakePedalState {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    // DBC definition for MsgID:280 Signal:DI_brakePedalState
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiBrakePedalState {
        Invalid,
        Off,
        On,
        _Other(u8),
    }

    impl From<DbcDiBrakePedalState> for u8 {
        fn from (val: DbcDiBrakePedalState) -> u8 {
            match val {
                DbcDiBrakePedalState::Invalid => 2_u8,
                DbcDiBrakePedalState::Off => 0_u8,
                DbcDiBrakePedalState::On => 1_u8,
                DbcDiBrakePedalState::_Other(x) => x
            }
        }
    }

    impl DiBrakePedalState  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiBrakePedalState {
                status: CanDataStatus::Unset,
                name:"DiBrakePedalState",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        pub fn get_as_def (&self) -> DbcDiBrakePedalState {
            match self.get_typed_value() {
                2_u8 => DbcDiBrakePedalState::Invalid,
                0_u8 => DbcDiBrakePedalState::Off,
                1_u8 => DbcDiBrakePedalState::On,
                _ => DbcDiBrakePedalState::_Other(self.get_typed_value()),
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiBrakePedalState, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiBrakePedalState::Invalid => self.set_typed_value(2_u8, data),
                DbcDiBrakePedalState::Off => self.set_typed_value(0_u8, data),
                DbcDiBrakePedalState::On => self.set_typed_value(1_u8, data),
                DbcDiBrakePedalState::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[19..21].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiBrakePedalState impl end

    impl fmt::Display for DiBrakePedalState {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiBrakePedalState:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiBrakePedalState {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiBrakePedalState")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiBrakePedalState public api (CanDbcSignal trait)
    impl CanDbcSignal for DiBrakePedalState {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[19..21].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiBrakePedalState public api

    /// Id118DriveSystemStatus::DiDriveBlocked
    /// - Min: 0
    /// - Max: 2
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 12
    /// - Signal size: 2 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiDriveBlocked {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    // DBC definition for MsgID:280 Signal:DI_driveBlocked
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiDriveBlocked {
        DriveBlockedFrunk,
        DriveBlockedNone,
        DriveBlockedProx,
        _Other(u8),
    }

    impl From<DbcDiDriveBlocked> for u8 {
        fn from (val: DbcDiDriveBlocked) -> u8 {
            match val {
                DbcDiDriveBlocked::DriveBlockedFrunk => 1_u8,
                DbcDiDriveBlocked::DriveBlockedNone => 0_u8,
                DbcDiDriveBlocked::DriveBlockedProx => 2_u8,
                DbcDiDriveBlocked::_Other(x) => x
            }
        }
    }

    impl DiDriveBlocked  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiDriveBlocked {
                status: CanDataStatus::Unset,
                name:"DiDriveBlocked",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        pub fn get_as_def (&self) -> DbcDiDriveBlocked {
            match self.get_typed_value() {
                1_u8 => DbcDiDriveBlocked::DriveBlockedFrunk,
                0_u8 => DbcDiDriveBlocked::DriveBlockedNone,
                2_u8 => DbcDiDriveBlocked::DriveBlockedProx,
                _ => DbcDiDriveBlocked::_Other(self.get_typed_value()),
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiDriveBlocked, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiDriveBlocked::DriveBlockedFrunk => self.set_typed_value(1_u8, data),
                DbcDiDriveBlocked::DriveBlockedNone => self.set_typed_value(0_u8, data),
                DbcDiDriveBlocked::DriveBlockedProx => self.set_typed_value(2_u8, data),
                DbcDiDriveBlocked::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[12..14].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiDriveBlocked impl end

    impl fmt::Display for DiDriveBlocked {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiDriveBlocked:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiDriveBlocked {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiDriveBlocked")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiDriveBlocked public api (CanDbcSignal trait)
    impl CanDbcSignal for DiDriveBlocked {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[12..14].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiDriveBlocked public api

    /// Id118DriveSystemStatus::DiEpbRequest
    /// - Min: 0
    /// - Max: 2
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 44
    /// - Signal size: 2 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiEpbRequest {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    // DBC definition for MsgID:280 Signal:DI_epbRequest
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiEpbRequest {
        DiEpbrequestNoRequest,
        DiEpbrequestPark,
        DiEpbrequestUnpark,
        _Other(u8),
    }

    impl From<DbcDiEpbRequest> for u8 {
        fn from (val: DbcDiEpbRequest) -> u8 {
            match val {
                DbcDiEpbRequest::DiEpbrequestNoRequest => 0_u8,
                DbcDiEpbRequest::DiEpbrequestPark => 1_u8,
                DbcDiEpbRequest::DiEpbrequestUnpark => 2_u8,
                DbcDiEpbRequest::_Other(x) => x
            }
        }
    }

    impl DiEpbRequest  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiEpbRequest {
                status: CanDataStatus::Unset,
                name:"DiEpbRequest",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        pub fn get_as_def (&self) -> DbcDiEpbRequest {
            match self.get_typed_value() {
                0_u8 => DbcDiEpbRequest::DiEpbrequestNoRequest,
                1_u8 => DbcDiEpbRequest::DiEpbrequestPark,
                2_u8 => DbcDiEpbRequest::DiEpbrequestUnpark,
                _ => DbcDiEpbRequest::_Other(self.get_typed_value()),
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiEpbRequest, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiEpbRequest::DiEpbrequestNoRequest => self.set_typed_value(0_u8, data),
                DbcDiEpbRequest::DiEpbrequestPark => self.set_typed_value(1_u8, data),
                DbcDiEpbRequest::DiEpbrequestUnpark => self.set_typed_value(2_u8, data),
                DbcDiEpbRequest::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[44..46].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiEpbRequest impl end

    impl fmt::Display for DiEpbRequest {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiEpbRequest:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiEpbRequest {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiEpbRequest")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiEpbRequest public api (CanDbcSignal trait)
    impl CanDbcSignal for DiEpbRequest {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[44..46].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiEpbRequest public api

    /// Id118DriveSystemStatus::DiGear
    ///
    /// Gear
    /// - Min: 0
    /// - Max: 7
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 21
    /// - Signal size: 3 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiGear {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    // DBC definition for MsgID:280 Signal:DI_gear
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiGear {
        DiGearD,
        DiGearInvalid,
        DiGearN,
        DiGearP,
        DiGearR,
        DiGearSna,
        _Other(u8),
    }

    impl From<DbcDiGear> for u8 {
        fn from (val: DbcDiGear) -> u8 {
            match val {
                DbcDiGear::DiGearD => 4_u8,
                DbcDiGear::DiGearInvalid => 0_u8,
                DbcDiGear::DiGearN => 3_u8,
                DbcDiGear::DiGearP => 1_u8,
                DbcDiGear::DiGearR => 2_u8,
                DbcDiGear::DiGearSna => 7_u8,
                DbcDiGear::_Other(x) => x
            }
        }
    }

    impl DiGear  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiGear {
                status: CanDataStatus::Unset,
                name:"DiGear",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        pub fn get_as_def (&self) -> DbcDiGear {
            match self.get_typed_value() {
                4_u8 => DbcDiGear::DiGearD,
                0_u8 => DbcDiGear::DiGearInvalid,
                3_u8 => DbcDiGear::DiGearN,
                1_u8 => DbcDiGear::DiGearP,
                2_u8 => DbcDiGear::DiGearR,
                7_u8 => DbcDiGear::DiGearSna,
                _ => DbcDiGear::_Other(self.get_typed_value()),
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiGear, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiGear::DiGearD => self.set_typed_value(4_u8, data),
                DbcDiGear::DiGearInvalid => self.set_typed_value(0_u8, data),
                DbcDiGear::DiGearN => self.set_typed_value(3_u8, data),
                DbcDiGear::DiGearP => self.set_typed_value(1_u8, data),
                DbcDiGear::DiGearR => self.set_typed_value(2_u8, data),
                DbcDiGear::DiGearSna => self.set_typed_value(7_u8, data),
                DbcDiGear::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[21..24].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiGear impl end

    impl fmt::Display for DiGear {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiGear:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiGear {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiGear")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiGear public api (CanDbcSignal trait)
    impl CanDbcSignal for DiGear {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[21..24].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiGear public api

    /// Id118DriveSystemStatus::DiImmobilizerState
    /// - Min: 0
    /// - Max: 6
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 27
    /// - Signal size: 3 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiImmobilizerState {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    // DBC definition for MsgID:280 Signal:DI_immobilizerState
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiImmobilizerState {
        DiImmStateAuthenticating,
        DiImmStateDisarmed,
        DiImmStateFault,
        DiImmStateIdle,
        DiImmStateInitSna,
        DiImmStateRequest,
        DiImmStateReset,
        _Other(u8),
    }

    impl From<DbcDiImmobilizerState> for u8 {
        fn from (val: DbcDiImmobilizerState) -> u8 {
            match val {
                DbcDiImmobilizerState::DiImmStateAuthenticating => 2_u8,
                DbcDiImmobilizerState::DiImmStateDisarmed => 3_u8,
                DbcDiImmobilizerState::DiImmStateFault => 6_u8,
                DbcDiImmobilizerState::DiImmStateIdle => 4_u8,
                DbcDiImmobilizerState::DiImmStateInitSna => 0_u8,
                DbcDiImmobilizerState::DiImmStateRequest => 1_u8,
                DbcDiImmobilizerState::DiImmStateReset => 5_u8,
                DbcDiImmobilizerState::_Other(x) => x
            }
        }
    }

    impl DiImmobilizerState  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiImmobilizerState {
                status: CanDataStatus::Unset,
                name:"DiImmobilizerState",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        pub fn get_as_def (&self) -> DbcDiImmobilizerState {
            match self.get_typed_value() {
                2_u8 => DbcDiImmobilizerState::DiImmStateAuthenticating,
                3_u8 => DbcDiImmobilizerState::DiImmStateDisarmed,
                6_u8 => DbcDiImmobilizerState::DiImmStateFault,
                4_u8 => DbcDiImmobilizerState::DiImmStateIdle,
                0_u8 => DbcDiImmobilizerState::DiImmStateInitSna,
                1_u8 => DbcDiImmobilizerState::DiImmStateRequest,
                5_u8 => DbcDiImmobilizerState::DiImmStateReset,
                _ => DbcDiImmobilizerState::_Other(self.get_typed_value()),
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiImmobilizerState, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiImmobilizerState::DiImmStateAuthenticating => self.set_typed_value(2_u8, data),
                DbcDiImmobilizerState::DiImmStateDisarmed => self.set_typed_value(3_u8, data),
                DbcDiImmobilizerState::DiImmStateFault => self.set_typed_value(6_u8, data),
                DbcDiImmobilizerState::DiImmStateIdle => self.set_typed_value(4_u8, data),
                DbcDiImmobilizerState::DiImmStateInitSna => self.set_typed_value(0_u8, data),
                DbcDiImmobilizerState::DiImmStateRequest => self.set_typed_value(1_u8, data),
                DbcDiImmobilizerState::DiImmStateReset => self.set_typed_value(5_u8, data),
                DbcDiImmobilizerState::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[27..30].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiImmobilizerState impl end

    impl fmt::Display for DiImmobilizerState {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiImmobilizerState:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiImmobilizerState {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiImmobilizerState")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiImmobilizerState public api (CanDbcSignal trait)
    impl CanDbcSignal for DiImmobilizerState {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[27..30].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiImmobilizerState public api

    /// Id118DriveSystemStatus::DiKeepDrivePowerStateRequest
    /// - Min: 0
    /// - Max: 1
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 47
    /// - Signal size: 1 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiKeepDrivePowerStateRequest {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: bool,
    }

    // DBC definition for MsgID:280 Signal:DI_keepDrivePowerStateRequest
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiKeepDrivePowerStateRequest {
        KeepAlive,
        NoRequest,
        _Other(bool),
    }

    impl From<DbcDiKeepDrivePowerStateRequest> for bool {
        fn from (val: DbcDiKeepDrivePowerStateRequest) -> bool {
            match val {
                DbcDiKeepDrivePowerStateRequest::KeepAlive => true,
                DbcDiKeepDrivePowerStateRequest::NoRequest => false,
                DbcDiKeepDrivePowerStateRequest::_Other(x) => x
            }
        }
    }

    impl DiKeepDrivePowerStateRequest  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiKeepDrivePowerStateRequest {
                status: CanDataStatus::Unset,
                name:"DiKeepDrivePowerStateRequest",
                value: false,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= false;
        }

        pub fn get_as_def (&self) -> DbcDiKeepDrivePowerStateRequest {
            match self.get_typed_value() {
                true => DbcDiKeepDrivePowerStateRequest::KeepAlive,
                false => DbcDiKeepDrivePowerStateRequest::NoRequest,
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiKeepDrivePowerStateRequest, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiKeepDrivePowerStateRequest::KeepAlive => self.set_typed_value(true, data),
                DbcDiKeepDrivePowerStateRequest::NoRequest => self.set_typed_value(false, data),
                DbcDiKeepDrivePowerStateRequest::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> bool {
            self.value
        }

        fn set_typed_value(&mut self, value:bool, data:&mut [u8]) -> Result<(),CanError> {
            let value = value as u8;
            data.view_bits_mut::<Lsb0>()[47..48].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiKeepDrivePowerStateRequest impl end

    impl fmt::Display for DiKeepDrivePowerStateRequest {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiKeepDrivePowerStateRequest:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiKeepDrivePowerStateRequest {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiKeepDrivePowerStateRequest")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiKeepDrivePowerStateRequest public api (CanDbcSignal trait)
    impl CanDbcSignal for DiKeepDrivePowerStateRequest {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[47..48].load_le::<u8>();
                    self.value= value == 1;
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:bool= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::Bool(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiKeepDrivePowerStateRequest public api

    /// Id118DriveSystemStatus::DiProximity
    /// - Min: 0
    /// - Max: 1
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 46
    /// - Signal size: 1 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiProximity {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: bool,
    }

    impl DiProximity  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiProximity {
                status: CanDataStatus::Unset,
                name:"DiProximity",
                value: false,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= false;
        }

        fn get_typed_value(&self) -> bool {
            self.value
        }

        fn set_typed_value(&mut self, value:bool, data:&mut [u8]) -> Result<(),CanError> {
            let value = value as u8;
            data.view_bits_mut::<Lsb0>()[46..47].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiProximity impl end

    impl fmt::Display for DiProximity {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiProximity:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiProximity {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiProximity")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiProximity public api (CanDbcSignal trait)
    impl CanDbcSignal for DiProximity {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[46..47].load_le::<u8>();
                    self.value= value == 1;
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:bool= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::Bool(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiProximity public api

    /// Id118DriveSystemStatus::DiRegenLight
    ///
    /// Regen Brake
    /// - Min: 0
    /// - Max: 1
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 26
    /// - Signal size: 1 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiRegenLight {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: bool,
    }

    impl DiRegenLight  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiRegenLight {
                status: CanDataStatus::Unset,
                name:"DiRegenLight",
                value: false,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= false;
        }

        fn get_typed_value(&self) -> bool {
            self.value
        }

        fn set_typed_value(&mut self, value:bool, data:&mut [u8]) -> Result<(),CanError> {
            let value = value as u8;
            data.view_bits_mut::<Lsb0>()[26..27].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiRegenLight impl end

    impl fmt::Display for DiRegenLight {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiRegenLight:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiRegenLight {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiRegenLight")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiRegenLight public api (CanDbcSignal trait)
    impl CanDbcSignal for DiRegenLight {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[26..27].load_le::<u8>();
                    self.value= value == 1;
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:bool= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::Bool(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiRegenLight public api

    /// Id118DriveSystemStatus::DiSystemState
    /// - Min: 0
    /// - Max: 5
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 16
    /// - Signal size: 3 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiSystemState {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    // DBC definition for MsgID:280 Signal:DI_systemState
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiSystemState {
        DiSysAbort,
        DiSysEnable,
        DiSysFault,
        DiSysIdle,
        DiSysStandby,
        DiSysUnavailable,
        _Other(u8),
    }

    impl From<DbcDiSystemState> for u8 {
        fn from (val: DbcDiSystemState) -> u8 {
            match val {
                DbcDiSystemState::DiSysAbort => 4_u8,
                DbcDiSystemState::DiSysEnable => 5_u8,
                DbcDiSystemState::DiSysFault => 3_u8,
                DbcDiSystemState::DiSysIdle => 1_u8,
                DbcDiSystemState::DiSysStandby => 2_u8,
                DbcDiSystemState::DiSysUnavailable => 0_u8,
                DbcDiSystemState::_Other(x) => x
            }
        }
    }

    impl DiSystemState  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiSystemState {
                status: CanDataStatus::Unset,
                name:"DiSystemState",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        pub fn get_as_def (&self) -> DbcDiSystemState {
            match self.get_typed_value() {
                4_u8 => DbcDiSystemState::DiSysAbort,
                5_u8 => DbcDiSystemState::DiSysEnable,
                3_u8 => DbcDiSystemState::DiSysFault,
                1_u8 => DbcDiSystemState::DiSysIdle,
                2_u8 => DbcDiSystemState::DiSysStandby,
                0_u8 => DbcDiSystemState::DiSysUnavailable,
                _ => DbcDiSystemState::_Other(self.get_typed_value()),
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiSystemState, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiSystemState::DiSysAbort => self.set_typed_value(4_u8, data),
                DbcDiSystemState::DiSysEnable => self.set_typed_value(5_u8, data),
                DbcDiSystemState::DiSysFault => self.set_typed_value(3_u8, data),
                DbcDiSystemState::DiSysIdle => self.set_typed_value(1_u8, data),
                DbcDiSystemState::DiSysStandby => self.set_typed_value(2_u8, data),
                DbcDiSystemState::DiSysUnavailable => self.set_typed_value(0_u8, data),
                DbcDiSystemState::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[16..19].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiSystemState impl end

    impl fmt::Display for DiSystemState {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiSystemState:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiSystemState {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiSystemState")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiSystemState public api (CanDbcSignal trait)
    impl CanDbcSignal for DiSystemState {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[16..19].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiSystemState public api

    /// Id118DriveSystemStatus::DiSystemStatusChecksum
    /// - Min: 0
    /// - Max: 255
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 0
    /// - Signal size: 8 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiSystemStatusChecksum {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    impl DiSystemStatusChecksum  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiSystemStatusChecksum {
                status: CanDataStatus::Unset,
                name:"DiSystemStatusChecksum",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[0..8].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiSystemStatusChecksum impl end

    impl fmt::Display for DiSystemStatusChecksum {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiSystemStatusChecksum:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiSystemStatusChecksum {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiSystemStatusChecksum")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiSystemStatusChecksum public api (CanDbcSignal trait)
    impl CanDbcSignal for DiSystemStatusChecksum {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[0..8].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiSystemStatusChecksum public api

    /// Id118DriveSystemStatus::DiSystemStatusCounter
    /// - Min: 0
    /// - Max: 15
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 8
    /// - Signal size: 4 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiSystemStatusCounter {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    impl DiSystemStatusCounter  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiSystemStatusCounter {
                status: CanDataStatus::Unset,
                name:"DiSystemStatusCounter",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[8..12].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiSystemStatusCounter impl end

    impl fmt::Display for DiSystemStatusCounter {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiSystemStatusCounter:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiSystemStatusCounter {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiSystemStatusCounter")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiSystemStatusCounter public api (CanDbcSignal trait)
    impl CanDbcSignal for DiSystemStatusCounter {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[8..12].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiSystemStatusCounter public api

    /// Id118DriveSystemStatus::DiTrackModeState
    /// - Min: 0
    /// - Max: 2
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 48
    /// - Signal size: 2 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiTrackModeState {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    // DBC definition for MsgID:280 Signal:DI_trackModeState
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiTrackModeState {
        TrackModeAvailable,
        TrackModeOn,
        TrackModeUnavailable,
        _Other(u8),
    }

    impl From<DbcDiTrackModeState> for u8 {
        fn from (val: DbcDiTrackModeState) -> u8 {
            match val {
                DbcDiTrackModeState::TrackModeAvailable => 1_u8,
                DbcDiTrackModeState::TrackModeOn => 2_u8,
                DbcDiTrackModeState::TrackModeUnavailable => 0_u8,
                DbcDiTrackModeState::_Other(x) => x
            }
        }
    }

    impl DiTrackModeState  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiTrackModeState {
                status: CanDataStatus::Unset,
                name:"DiTrackModeState",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        pub fn get_as_def (&self) -> DbcDiTrackModeState {
            match self.get_typed_value() {
                1_u8 => DbcDiTrackModeState::TrackModeAvailable,
                2_u8 => DbcDiTrackModeState::TrackModeOn,
                0_u8 => DbcDiTrackModeState::TrackModeUnavailable,
                _ => DbcDiTrackModeState::_Other(self.get_typed_value()),
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiTrackModeState, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiTrackModeState::TrackModeAvailable => self.set_typed_value(1_u8, data),
                DbcDiTrackModeState::TrackModeOn => self.set_typed_value(2_u8, data),
                DbcDiTrackModeState::TrackModeUnavailable => self.set_typed_value(0_u8, data),
                DbcDiTrackModeState::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[48..50].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiTrackModeState impl end

    impl fmt::Display for DiTrackModeState {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiTrackModeState:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiTrackModeState {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiTrackModeState")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiTrackModeState public api (CanDbcSignal trait)
    impl CanDbcSignal for DiTrackModeState {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[48..50].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiTrackModeState public api

    /// Id118DriveSystemStatus::DiTractionControlMode
    ///
    /// Traction Control Mode
    /// - Min: 0
    /// - Max: 6
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 40
    /// - Signal size: 3 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiTractionControlMode {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    // DBC definition for MsgID:280 Signal:DI_tractionControlMode
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiTractionControlMode {
        OffroadAssist,
        DynoMode,
        RollsMode,
        Dev2,
        Dev1,
        SlipStart,
        Standard,
        _Other(u8),
    }

    impl From<DbcDiTractionControlMode> for u8 {
        fn from (val: DbcDiTractionControlMode) -> u8 {
            match val {
                DbcDiTractionControlMode::OffroadAssist => 6_u8,
                DbcDiTractionControlMode::DynoMode => 5_u8,
                DbcDiTractionControlMode::RollsMode => 4_u8,
                DbcDiTractionControlMode::Dev2 => 3_u8,
                DbcDiTractionControlMode::Dev1 => 2_u8,
                DbcDiTractionControlMode::SlipStart => 1_u8,
                DbcDiTractionControlMode::Standard => 0_u8,
                DbcDiTractionControlMode::_Other(x) => x
            }
        }
    }

    impl DiTractionControlMode  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiTractionControlMode {
                status: CanDataStatus::Unset,
                name:"DiTractionControlMode",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        pub fn get_as_def (&self) -> DbcDiTractionControlMode {
            match self.get_typed_value() {
                6_u8 => DbcDiTractionControlMode::OffroadAssist,
                5_u8 => DbcDiTractionControlMode::DynoMode,
                4_u8 => DbcDiTractionControlMode::RollsMode,
                3_u8 => DbcDiTractionControlMode::Dev2,
                2_u8 => DbcDiTractionControlMode::Dev1,
                1_u8 => DbcDiTractionControlMode::SlipStart,
                0_u8 => DbcDiTractionControlMode::Standard,
                _ => DbcDiTractionControlMode::_Other(self.get_typed_value()),
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiTractionControlMode, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiTractionControlMode::OffroadAssist => self.set_typed_value(6_u8, data),
                DbcDiTractionControlMode::DynoMode => self.set_typed_value(5_u8, data),
                DbcDiTractionControlMode::RollsMode => self.set_typed_value(4_u8, data),
                DbcDiTractionControlMode::Dev2 => self.set_typed_value(3_u8, data),
                DbcDiTractionControlMode::Dev1 => self.set_typed_value(2_u8, data),
                DbcDiTractionControlMode::SlipStart => self.set_typed_value(1_u8, data),
                DbcDiTractionControlMode::Standard => self.set_typed_value(0_u8, data),
                DbcDiTractionControlMode::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[40..43].store_le(value);
            Ok(())
        }

    } // Id118DriveSystemStatus::DiTractionControlMode impl end

    impl fmt::Display for DiTractionControlMode {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiTractionControlMode:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiTractionControlMode {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiTractionControlMode")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id118DriveSystemStatus::DiTractionControlMode public api (CanDbcSignal trait)
    impl CanDbcSignal for DiTractionControlMode {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[40..43].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id118DriveSystemStatus::DiTractionControlMode public api

    pub struct DbcMessage {
        callback: Option<RefCell<Box<dyn CanMsgCtrl>>>,
        signals: [Rc<RefCell<Box<dyn CanDbcSignal>>>;14],
        name: &'static str,
        status: CanBcmOpCode,
        listeners: i32,
        stamp: u64,
        id: u32,
    }

    impl DbcMessage {
        pub fn new() -> Rc<RefCell<Box <dyn CanDbcMessage>>> {
            Rc::new(RefCell::new(Box::new (DbcMessage {
                id: 280,
                name: "Id118DriveSystemStatus",
                status: CanBcmOpCode::Unknown,
                listeners: 0,
                stamp: 0,
                callback: None,
                signals: [
                    DiAccelPedalPos::new(),
                    DiBrakePedalState::new(),
                    DiDriveBlocked::new(),
                    DiEpbRequest::new(),
                    DiGear::new(),
                    DiImmobilizerState::new(),
                    DiKeepDrivePowerStateRequest::new(),
                    DiProximity::new(),
                    DiRegenLight::new(),
                    DiSystemState::new(),
                    DiSystemStatusChecksum::new(),
                    DiSystemStatusCounter::new(),
                    DiTrackModeState::new(),
                    DiTractionControlMode::new(),
                ],
            })))
        }

        pub fn set_values(&mut self, di_accel_pedal_pos: f64, di_brake_pedal_state: u8, di_drive_blocked: u8, di_epb_request: u8, di_gear: u8, di_immobilizer_state: u8, di_keep_drive_power_state_request: bool, di_proximity: bool, di_regen_light: bool, di_system_state: u8, di_system_status_checksum: u8, di_system_status_counter: u8, di_track_mode_state: u8, di_traction_control_mode: u8, frame: &mut[u8]) -> Result<&mut Self, CanError> {

            match Rc::clone (&self.signals[0]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::F64(di_accel_pedal_pos), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_accel_pedal_pos:F64")),
            }
            match Rc::clone (&self.signals[1]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_brake_pedal_state), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_brake_pedal_state:U8")),
            }
            match Rc::clone (&self.signals[2]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_drive_blocked), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_drive_blocked:U8")),
            }
            match Rc::clone (&self.signals[3]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_epb_request), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_epb_request:U8")),
            }
            match Rc::clone (&self.signals[4]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_gear), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_gear:U8")),
            }
            match Rc::clone (&self.signals[5]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_immobilizer_state), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_immobilizer_state:U8")),
            }
            match Rc::clone (&self.signals[6]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::Bool(di_keep_drive_power_state_request), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_keep_drive_power_state_request:Bool")),
            }
            match Rc::clone (&self.signals[7]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::Bool(di_proximity), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_proximity:Bool")),
            }
            match Rc::clone (&self.signals[8]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::Bool(di_regen_light), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_regen_light:Bool")),
            }
            match Rc::clone (&self.signals[9]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_system_state), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_system_state:U8")),
            }
            match Rc::clone (&self.signals[10]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_system_status_checksum), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_system_status_checksum:U8")),
            }
            match Rc::clone (&self.signals[11]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_system_status_counter), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_system_status_counter:U8")),
            }
            match Rc::clone (&self.signals[12]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_track_mode_state), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_track_mode_state:U8")),
            }
            match Rc::clone (&self.signals[13]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_traction_control_mode), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_traction_control_mode:U8")),
            }
            Ok(self)
        }
    }

    impl CanDbcMessage for DbcMessage {
        fn reset(&mut self) -> Result<(), CanError> {
            self.status=CanBcmOpCode::Unknown;
            self.stamp=0;
            match Rc::clone (&self.signals[0]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_accel_pedal_pos:F64")),
            }
            match Rc::clone (&self.signals[1]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_brake_pedal_state:U8")),
            }
            match Rc::clone (&self.signals[2]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_drive_blocked:U8")),
            }
            match Rc::clone (&self.signals[3]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_epb_request:U8")),
            }
            match Rc::clone (&self.signals[4]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_gear:U8")),
            }
            match Rc::clone (&self.signals[5]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_immobilizer_state:U8")),
            }
            match Rc::clone (&self.signals[6]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_keep_drive_power_state_request:Bool")),
            }
            match Rc::clone (&self.signals[7]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_proximity:Bool")),
            }
            match Rc::clone (&self.signals[8]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_regen_light:Bool")),
            }
            match Rc::clone (&self.signals[9]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_system_state:U8")),
            }
            match Rc::clone (&self.signals[10]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_system_status_checksum:U8")),
            }
            match Rc::clone (&self.signals[11]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_system_status_counter:U8")),
            }
            match Rc::clone (&self.signals[12]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_track_mode_state:U8")),
            }
            match Rc::clone (&self.signals[13]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_traction_control_mode:U8")),
            }
        Ok(())
    }

        fn update(&mut self, frame: &CanMsgData) -> Result<(), CanError> {
            self.stamp= frame.stamp;
            self.status= frame.opcode;
            self.listeners= 0;
            match Rc::clone (&self.signals[0]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_accel_pedal_pos:F64")),
            }
            match Rc::clone (&self.signals[1]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_brake_pedal_state:U8")),
            }
            match Rc::clone (&self.signals[2]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_drive_blocked:U8")),
            }
            match Rc::clone (&self.signals[3]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_epb_request:U8")),
            }
            match Rc::clone (&self.signals[4]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_gear:U8")),
            }
            match Rc::clone (&self.signals[5]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_immobilizer_state:U8")),
            }
            match Rc::clone (&self.signals[6]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_keep_drive_power_state_request:Bool")),
            }
            match Rc::clone (&self.signals[7]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_proximity:Bool")),
            }
            match Rc::clone (&self.signals[8]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_regen_light:Bool")),
            }
            match Rc::clone (&self.signals[9]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_system_state:U8")),
            }
            match Rc::clone (&self.signals[10]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_system_status_checksum:U8")),
            }
            match Rc::clone (&self.signals[11]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_system_status_counter:U8")),
            }
            match Rc::clone (&self.signals[12]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_track_mode_state:U8")),
            }
            match Rc::clone (&self.signals[13]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_traction_control_mode:U8")),
            }
            match &self.callback {
                None => {},
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => println!("fail to get message callback reference"),
                        Ok(cb_ref) => cb_ref.msg_notification(self),
                    }
                }
            }
            Ok(())
        }

        fn get_signals(&self) -> &[Rc<RefCell<Box<dyn CanDbcSignal>>>] {
            &self.signals
        }

        fn get_listeners(&self) -> i32 {
            self.listeners
        }

        fn set_callback(&mut self, callback: Box<dyn CanMsgCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_status(&self) -> CanBcmOpCode {
            self.status
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_id(&self) -> u32 {
            self.id
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

    } // end Id118DriveSystemStatus impl for CanDbcMessage
} // end Id118DriveSystemStatus message

/// ID257DIspeed Message
/// - ID: 599 (0x257)
/// - Size: 8 bytes
/// - Transmitter: VehicleBus
pub mod Id257dIspeed { /// Message name space
    use sockcan::prelude::*;
    use bitvec::prelude::*;
    use std::any::Any;
    use std::cell::{RefCell};
    use std::rc::Rc;

    use std::fmt;

    use serde::{Deserialize, Serialize};
    pub enum DbcSignal {
        DiSpeedChecksum,
        DiSpeedCounter,
        DiUiSpeed,
        DiUiSpeedHighSpeed,
        DiUiSpeedUnits,
        DiVehicleSpeed,
    }

    /// Id257dIspeed::DiSpeedChecksum
    /// - Min: 0
    /// - Max: 255
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 0
    /// - Signal size: 8 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiSpeedChecksum {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    impl DiSpeedChecksum  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiSpeedChecksum {
                status: CanDataStatus::Unset,
                name:"DiSpeedChecksum",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[0..8].store_le(value);
            Ok(())
        }

    } // Id257dIspeed::DiSpeedChecksum impl end

    impl fmt::Display for DiSpeedChecksum {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiSpeedChecksum:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiSpeedChecksum {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiSpeedChecksum")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id257dIspeed::DiSpeedChecksum public api (CanDbcSignal trait)
    impl CanDbcSignal for DiSpeedChecksum {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[0..8].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id257dIspeed::DiSpeedChecksum public api

    /// Id257dIspeed::DiSpeedCounter
    /// - Min: 0
    /// - Max: 15
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 8
    /// - Signal size: 4 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiSpeedCounter {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u8,
    }

    impl DiSpeedCounter  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiSpeedCounter {
                status: CanDataStatus::Unset,
                name:"DiSpeedCounter",
                value: 0_u8,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u8;
        }

        fn get_typed_value(&self) -> u8 {
            self.value
        }

        fn set_typed_value(&mut self, value:u8, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[8..12].store_le(value);
            Ok(())
        }

    } // Id257dIspeed::DiSpeedCounter impl end

    impl fmt::Display for DiSpeedCounter {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiSpeedCounter:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiSpeedCounter {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiSpeedCounter")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id257dIspeed::DiSpeedCounter public api (CanDbcSignal trait)
    impl CanDbcSignal for DiSpeedCounter {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[8..12].load_le::<u8>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u8= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U8(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id257dIspeed::DiSpeedCounter public api

    /// Id257dIspeed::DiUiSpeed
    ///
    /// UI Speed
    /// - Min: 0
    /// - Max: 510
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 24
    /// - Signal size: 9 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiUiSpeed {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u16,
    }

    // DBC definition for MsgID:599 Signal:DI_uiSpeed
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiUiSpeed {
        DiUiSpeedSna,
        _Other(u16),
    }

    impl From<DbcDiUiSpeed> for u16 {
        fn from (val: DbcDiUiSpeed) -> u16 {
            match val {
                DbcDiUiSpeed::DiUiSpeedSna => panic! ("(Hoops) impossible conversion 511_u16 -> u16"),
                DbcDiUiSpeed::_Other(x) => x
            }
        }
    }

    impl DiUiSpeed  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiUiSpeed {
                status: CanDataStatus::Unset,
                name:"DiUiSpeed",
                value: 0_u16,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u16;
        }

        pub fn get_as_def (&self) -> DbcDiUiSpeed {
            match self.get_typed_value() {
                // WARNING 511_u16 => Err(CanError::new("not-in-range","(DiUiSpeedSna) !!! 511(u16) not in [0..510] range")),
                _ => DbcDiUiSpeed::_Other(self.get_typed_value()),
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiUiSpeed, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiUiSpeed::DiUiSpeedSna => Err(CanError::new("not-in-range","(DiUiSpeedSna) !!! 511(u16) not in [0..510] range")),
                DbcDiUiSpeed::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> u16 {
            self.value
        }

        fn set_typed_value(&mut self, value:u16, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[24..33].store_le(value);
            Ok(())
        }

    } // Id257dIspeed::DiUiSpeed impl end

    impl fmt::Display for DiUiSpeed {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiUiSpeed:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiUiSpeed {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiUiSpeed")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id257dIspeed::DiUiSpeed public api (CanDbcSignal trait)
    impl CanDbcSignal for DiUiSpeed {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[24..33].load_le::<u16>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u16= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U16(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id257dIspeed::DiUiSpeed public api

    /// Id257dIspeed::DiUiSpeedHighSpeed
    /// - Min: 0
    /// - Max: 510
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 34
    /// - Signal size: 9 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiUiSpeedHighSpeed {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u16,
    }

    // DBC definition for MsgID:599 Signal:DI_uiSpeedHighSpeed
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiUiSpeedHighSpeed {
        DiUiHighSpeedSna,
        _Other(u16),
    }

    impl From<DbcDiUiSpeedHighSpeed> for u16 {
        fn from (val: DbcDiUiSpeedHighSpeed) -> u16 {
            match val {
                DbcDiUiSpeedHighSpeed::DiUiHighSpeedSna => panic! ("(Hoops) impossible conversion 511_u16 -> u16"),
                DbcDiUiSpeedHighSpeed::_Other(x) => x
            }
        }
    }

    impl DiUiSpeedHighSpeed  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiUiSpeedHighSpeed {
                status: CanDataStatus::Unset,
                name:"DiUiSpeedHighSpeed",
                value: 0_u16,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u16;
        }

        pub fn get_as_def (&self) -> DbcDiUiSpeedHighSpeed {
            match self.get_typed_value() {
                // WARNING 511_u16 => Err(CanError::new("not-in-range","(DiUiHighSpeedSna) !!! 511(u16) not in [0..510] range")),
                _ => DbcDiUiSpeedHighSpeed::_Other(self.get_typed_value()),
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiUiSpeedHighSpeed, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiUiSpeedHighSpeed::DiUiHighSpeedSna => Err(CanError::new("not-in-range","(DiUiHighSpeedSna) !!! 511(u16) not in [0..510] range")),
                DbcDiUiSpeedHighSpeed::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> u16 {
            self.value
        }

        fn set_typed_value(&mut self, value:u16, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[34..43].store_le(value);
            Ok(())
        }

    } // Id257dIspeed::DiUiSpeedHighSpeed impl end

    impl fmt::Display for DiUiSpeedHighSpeed {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiUiSpeedHighSpeed:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiUiSpeedHighSpeed {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiUiSpeedHighSpeed")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id257dIspeed::DiUiSpeedHighSpeed public api (CanDbcSignal trait)
    impl CanDbcSignal for DiUiSpeedHighSpeed {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[34..43].load_le::<u16>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u16= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U16(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id257dIspeed::DiUiSpeedHighSpeed public api

    /// Id257dIspeed::DiUiSpeedUnits
    ///
    /// Speed Units, 0-mph 1-kph
    /// - Min: 0
    /// - Max: 1
    /// - Unit: ""
    /// - Receivers: Receiver
    /// - Start bit: 33
    /// - Signal size: 1 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiUiSpeedUnits {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: bool,
    }

    // DBC definition for MsgID:599 Signal:DI_uiSpeedUnits
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiUiSpeedUnits {
        DiSpeedKph,
        DiSpeedMph,
        _Other(bool),
    }

    impl From<DbcDiUiSpeedUnits> for bool {
        fn from (val: DbcDiUiSpeedUnits) -> bool {
            match val {
                DbcDiUiSpeedUnits::DiSpeedKph => true,
                DbcDiUiSpeedUnits::DiSpeedMph => false,
                DbcDiUiSpeedUnits::_Other(x) => x
            }
        }
    }

    impl DiUiSpeedUnits  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiUiSpeedUnits {
                status: CanDataStatus::Unset,
                name:"DiUiSpeedUnits",
                value: false,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= false;
        }

        pub fn get_as_def (&self) -> DbcDiUiSpeedUnits {
            match self.get_typed_value() {
                true => DbcDiUiSpeedUnits::DiSpeedKph,
                false => DbcDiUiSpeedUnits::DiSpeedMph,
            }
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiUiSpeedUnits, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiUiSpeedUnits::DiSpeedKph => self.set_typed_value(true, data),
                DbcDiUiSpeedUnits::DiSpeedMph => self.set_typed_value(false, data),
                DbcDiUiSpeedUnits::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> bool {
            self.value
        }

        fn set_typed_value(&mut self, value:bool, data:&mut [u8]) -> Result<(),CanError> {
            let value = value as u8;
            data.view_bits_mut::<Lsb0>()[33..34].store_le(value);
            Ok(())
        }

    } // Id257dIspeed::DiUiSpeedUnits impl end

    impl fmt::Display for DiUiSpeedUnits {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiUiSpeedUnits:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiUiSpeedUnits {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiUiSpeedUnits")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id257dIspeed::DiUiSpeedUnits public api (CanDbcSignal trait)
    impl CanDbcSignal for DiUiSpeedUnits {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[33..34].load_le::<u8>();
                    self.value= value == 1;
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:bool= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::Bool(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id257dIspeed::DiUiSpeedUnits public api

    /// Id257dIspeed::DiVehicleSpeed
    ///
    /// Vehicle Speed, .05 -25 for mph
    /// - Min: -40
    /// - Max: 285
    /// - Unit: "kph"
    /// - Receivers: Receiver
    /// - Start bit: 12
    /// - Signal size: 12 bits
    /// - Factor: 0.08
    /// - Offset: -40
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct DiVehicleSpeed {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: f64,
    }

    // DBC definition for MsgID:599 Signal:DI_vehicleSpeed
    #[derive(Serialize, Deserialize)]
    pub enum DbcDiVehicleSpeed {
        Sna,
        _Other(f64),
    }

    impl From<DbcDiVehicleSpeed> for f64 {
        fn from (val: DbcDiVehicleSpeed) -> f64 {
            match val {
                DbcDiVehicleSpeed::Sna => panic! ("(Hoops) impossible conversion 4095_f64 -> f64"),
                DbcDiVehicleSpeed::_Other(x) => x
            }
        }
    }

    impl DiVehicleSpeed  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(DiVehicleSpeed {
                status: CanDataStatus::Unset,
                name:"DiVehicleSpeed",
                value: 0_f64,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_f64;
        }

        pub fn get_as_def (&self) -> DbcDiVehicleSpeed {
                DbcDiVehicleSpeed::_Other(self.get_typed_value())
        }

        pub fn set_as_def (&mut self, signal_def: DbcDiVehicleSpeed, data: &mut[u8])-> Result<(),CanError> {
            match signal_def {
                DbcDiVehicleSpeed::Sna => Err(CanError::new("not-in-range","(Sna) !!! 4095(f64) not in [-40..285] range")),
                DbcDiVehicleSpeed::_Other(x) => self.set_typed_value(x,data)
            }
        }
        fn get_typed_value(&self) -> f64 {
            self.value
        }

        fn set_typed_value(&mut self, value:f64, data:&mut [u8]) -> Result<(),CanError> {
            if value < -40_f64 || 285_f64 < value {
                return Err(CanError::new("invalid-signal-value",format!("value={} not in [-40..285]",value)));
            }
            let factor = 0.08_f64;
            let offset = -40_f64;
            let value = ((value - offset) / factor) as u16;
            data.view_bits_mut::<Lsb0>()[12..24].store_le(value);
            Ok(())
        }

    } // Id257dIspeed::DiVehicleSpeed impl end

    impl fmt::Display for DiVehicleSpeed {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("DiVehicleSpeed:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for DiVehicleSpeed {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("DiVehicleSpeed")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id257dIspeed::DiVehicleSpeed public api (CanDbcSignal trait)
    impl CanDbcSignal for DiVehicleSpeed {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[12..24].load_le::<u16>();
                    let factor = 0.08_f64;
                    let offset = -40_f64;
                    let newval= (value as f64) * factor + offset;
                    if newval != self.value {
                        self.value= newval;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:f64= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::F64(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id257dIspeed::DiVehicleSpeed public api

    pub struct DbcMessage {
        callback: Option<RefCell<Box<dyn CanMsgCtrl>>>,
        signals: [Rc<RefCell<Box<dyn CanDbcSignal>>>;6],
        name: &'static str,
        status: CanBcmOpCode,
        listeners: i32,
        stamp: u64,
        id: u32,
    }

    impl DbcMessage {
        pub fn new() -> Rc<RefCell<Box <dyn CanDbcMessage>>> {
            Rc::new(RefCell::new(Box::new (DbcMessage {
                id: 599,
                name: "Id257dIspeed",
                status: CanBcmOpCode::Unknown,
                listeners: 0,
                stamp: 0,
                callback: None,
                signals: [
                    DiSpeedChecksum::new(),
                    DiSpeedCounter::new(),
                    DiUiSpeed::new(),
                    DiUiSpeedHighSpeed::new(),
                    DiUiSpeedUnits::new(),
                    DiVehicleSpeed::new(),
                ],
            })))
        }

        pub fn set_values(&mut self, di_speed_checksum: u8, di_speed_counter: u8, di_ui_speed: u16, di_ui_speed_high_speed: u16, di_ui_speed_units: bool, di_vehicle_speed: f64, frame: &mut[u8]) -> Result<&mut Self, CanError> {

            match Rc::clone (&self.signals[0]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_speed_checksum), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_speed_checksum:U8")),
            }
            match Rc::clone (&self.signals[1]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U8(di_speed_counter), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_speed_counter:U8")),
            }
            match Rc::clone (&self.signals[2]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U16(di_ui_speed), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_ui_speed:U16")),
            }
            match Rc::clone (&self.signals[3]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U16(di_ui_speed_high_speed), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_ui_speed_high_speed:U16")),
            }
            match Rc::clone (&self.signals[4]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::Bool(di_ui_speed_units), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_ui_speed_units:Bool")),
            }
            match Rc::clone (&self.signals[5]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::F64(di_vehicle_speed), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error di_vehicle_speed:F64")),
            }
            Ok(self)
        }
    }

    impl CanDbcMessage for DbcMessage {
        fn reset(&mut self) -> Result<(), CanError> {
            self.status=CanBcmOpCode::Unknown;
            self.stamp=0;
            match Rc::clone (&self.signals[0]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_speed_checksum:U8")),
            }
            match Rc::clone (&self.signals[1]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_speed_counter:U8")),
            }
            match Rc::clone (&self.signals[2]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_ui_speed:U16")),
            }
            match Rc::clone (&self.signals[3]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_ui_speed_high_speed:U16")),
            }
            match Rc::clone (&self.signals[4]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_ui_speed_units:Bool")),
            }
            match Rc::clone (&self.signals[5]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error di_vehicle_speed:F64")),
            }
        Ok(())
    }

        fn update(&mut self, frame: &CanMsgData) -> Result<(), CanError> {
            self.stamp= frame.stamp;
            self.status= frame.opcode;
            self.listeners= 0;
            match Rc::clone (&self.signals[0]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_speed_checksum:U8")),
            }
            match Rc::clone (&self.signals[1]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_speed_counter:U8")),
            }
            match Rc::clone (&self.signals[2]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_ui_speed:U16")),
            }
            match Rc::clone (&self.signals[3]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_ui_speed_high_speed:U16")),
            }
            match Rc::clone (&self.signals[4]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_ui_speed_units:Bool")),
            }
            match Rc::clone (&self.signals[5]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error di_vehicle_speed:F64")),
            }
            match &self.callback {
                None => {},
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => println!("fail to get message callback reference"),
                        Ok(cb_ref) => cb_ref.msg_notification(self),
                    }
                }
            }
            Ok(())
        }

        fn get_signals(&self) -> &[Rc<RefCell<Box<dyn CanDbcSignal>>>] {
            &self.signals
        }

        fn get_listeners(&self) -> i32 {
            self.listeners
        }

        fn set_callback(&mut self, callback: Box<dyn CanMsgCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_status(&self) -> CanBcmOpCode {
            self.status
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_id(&self) -> u32 {
            self.id
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

    } // end Id257dIspeed impl for CanDbcMessage
} // end Id257dIspeed message

/// ID266RearInverterPower Message
/// - ID: 614 (0x266)
/// - Size: 8 bytes
/// - Transmitter: VehicleBus
pub mod Id266RearInverterPower { /// Message name space
    use sockcan::prelude::*;
    use bitvec::prelude::*;
    use std::any::Any;
    use std::cell::{RefCell};
    use std::rc::Rc;

    use std::fmt;

    use serde::{Deserialize, Serialize};
    pub enum DbcSignal {
        RearHeatPowerMax266,
        RearPowerLimit266,
        RearHeatPower266,
        RearHeatPowerOptimal266,
        RearExcessHeatCmd,
        RearPower266,
    }

    /// Id266RearInverterPower::RearHeatPowerMax266
    /// - Min: 0
    /// - Max: 20
    /// - Unit: "kW"
    /// - Receivers: Receiver
    /// - Start bit: 24
    /// - Signal size: 8 bits
    /// - Factor: 0.08
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct RearHeatPowerMax266 {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: f64,
    }

    impl RearHeatPowerMax266  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(RearHeatPowerMax266 {
                status: CanDataStatus::Unset,
                name:"RearHeatPowerMax266",
                value: 0_f64,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_f64;
        }

        fn get_typed_value(&self) -> f64 {
            self.value
        }

        fn set_typed_value(&mut self, value:f64, data:&mut [u8]) -> Result<(),CanError> {
            if value < 0_f64 || 20_f64 < value {
                return Err(CanError::new("invalid-signal-value",format!("value={} not in [0..20]",value)));
            }
            let factor = 0.08_f64;
            let offset = 0_f64;
            let value = ((value - offset) / factor) as u8;
            data.view_bits_mut::<Lsb0>()[24..32].store_le(value);
            Ok(())
        }

    } // Id266RearInverterPower::RearHeatPowerMax266 impl end

    impl fmt::Display for RearHeatPowerMax266 {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("RearHeatPowerMax266:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for RearHeatPowerMax266 {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("RearHeatPowerMax266")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id266RearInverterPower::RearHeatPowerMax266 public api (CanDbcSignal trait)
    impl CanDbcSignal for RearHeatPowerMax266 {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[24..32].load_le::<u8>();
                    let factor = 0.08_f64;
                    let offset = 0_f64;
                    let newval= (value as f64) * factor + offset;
                    if newval != self.value {
                        self.value= newval;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:f64= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::F64(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id266RearInverterPower::RearHeatPowerMax266 public api

    /// Id266RearInverterPower::RearPowerLimit266
    ///
    /// Rear Power Limit, approx offset Orig scale 1
    /// - Min: 0
    /// - Max: 400
    /// - Unit: "kW"
    /// - Receivers: Receiver
    /// - Start bit: 48
    /// - Signal size: 9 bits
    /// - Factor: 1
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct RearPowerLimit266 {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: u16,
    }

    impl RearPowerLimit266  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(RearPowerLimit266 {
                status: CanDataStatus::Unset,
                name:"RearPowerLimit266",
                value: 0_u16,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_u16;
        }

        fn get_typed_value(&self) -> u16 {
            self.value
        }

        fn set_typed_value(&mut self, value:u16, data:&mut [u8]) -> Result<(),CanError> {
            data.view_bits_mut::<Lsb0>()[48..57].store_le(value);
            Ok(())
        }

    } // Id266RearInverterPower::RearPowerLimit266 impl end

    impl fmt::Display for RearPowerLimit266 {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("RearPowerLimit266:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for RearPowerLimit266 {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("RearPowerLimit266")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id266RearInverterPower::RearPowerLimit266 public api (CanDbcSignal trait)
    impl CanDbcSignal for RearPowerLimit266 {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[48..57].load_le::<u16>();
                    if self.value != value {
                        self.value= value;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:u16= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::U16(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id266RearInverterPower::RearPowerLimit266 public api

    /// Id266RearInverterPower::RearHeatPower266
    ///
    /// Rear Waste Heat Power
    /// - Min: 0
    /// - Max: 20
    /// - Unit: "kW"
    /// - Receivers: Receiver
    /// - Start bit: 32
    /// - Signal size: 8 bits
    /// - Factor: 0.08
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct RearHeatPower266 {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: f64,
    }

    impl RearHeatPower266  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(RearHeatPower266 {
                status: CanDataStatus::Unset,
                name:"RearHeatPower266",
                value: 0_f64,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_f64;
        }

        fn get_typed_value(&self) -> f64 {
            self.value
        }

        fn set_typed_value(&mut self, value:f64, data:&mut [u8]) -> Result<(),CanError> {
            if value < 0_f64 || 20_f64 < value {
                return Err(CanError::new("invalid-signal-value",format!("value={} not in [0..20]",value)));
            }
            let factor = 0.08_f64;
            let offset = 0_f64;
            let value = ((value - offset) / factor) as u8;
            data.view_bits_mut::<Lsb0>()[32..40].store_le(value);
            Ok(())
        }

    } // Id266RearInverterPower::RearHeatPower266 impl end

    impl fmt::Display for RearHeatPower266 {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("RearHeatPower266:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for RearHeatPower266 {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("RearHeatPower266")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id266RearInverterPower::RearHeatPower266 public api (CanDbcSignal trait)
    impl CanDbcSignal for RearHeatPower266 {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[32..40].load_le::<u8>();
                    let factor = 0.08_f64;
                    let offset = 0_f64;
                    let newval= (value as f64) * factor + offset;
                    if newval != self.value {
                        self.value= newval;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:f64= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::F64(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id266RearInverterPower::RearHeatPower266 public api

    /// Id266RearInverterPower::RearHeatPowerOptimal266
    /// - Min: 0
    /// - Max: 20
    /// - Unit: "kW"
    /// - Receivers: Receiver
    /// - Start bit: 16
    /// - Signal size: 8 bits
    /// - Factor: 0.08
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct RearHeatPowerOptimal266 {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: f64,
    }

    impl RearHeatPowerOptimal266  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(RearHeatPowerOptimal266 {
                status: CanDataStatus::Unset,
                name:"RearHeatPowerOptimal266",
                value: 0_f64,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_f64;
        }

        fn get_typed_value(&self) -> f64 {
            self.value
        }

        fn set_typed_value(&mut self, value:f64, data:&mut [u8]) -> Result<(),CanError> {
            if value < 0_f64 || 20_f64 < value {
                return Err(CanError::new("invalid-signal-value",format!("value={} not in [0..20]",value)));
            }
            let factor = 0.08_f64;
            let offset = 0_f64;
            let value = ((value - offset) / factor) as u8;
            data.view_bits_mut::<Lsb0>()[16..24].store_le(value);
            Ok(())
        }

    } // Id266RearInverterPower::RearHeatPowerOptimal266 impl end

    impl fmt::Display for RearHeatPowerOptimal266 {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("RearHeatPowerOptimal266:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for RearHeatPowerOptimal266 {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("RearHeatPowerOptimal266")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id266RearInverterPower::RearHeatPowerOptimal266 public api (CanDbcSignal trait)
    impl CanDbcSignal for RearHeatPowerOptimal266 {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[16..24].load_le::<u8>();
                    let factor = 0.08_f64;
                    let offset = 0_f64;
                    let newval= (value as f64) * factor + offset;
                    if newval != self.value {
                        self.value= newval;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:f64= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::F64(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id266RearInverterPower::RearHeatPowerOptimal266 public api

    /// Id266RearInverterPower::RearExcessHeatCmd
    /// - Min: 0
    /// - Max: 20
    /// - Unit: "kW"
    /// - Receivers: Receiver
    /// - Start bit: 40
    /// - Signal size: 8 bits
    /// - Factor: 0.08
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Unsigned
    #[derive(Serialize, Deserialize)]
    pub struct RearExcessHeatCmd {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: f64,
    }

    impl RearExcessHeatCmd  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(RearExcessHeatCmd {
                status: CanDataStatus::Unset,
                name:"RearExcessHeatCmd",
                value: 0_f64,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_f64;
        }

        fn get_typed_value(&self) -> f64 {
            self.value
        }

        fn set_typed_value(&mut self, value:f64, data:&mut [u8]) -> Result<(),CanError> {
            if value < 0_f64 || 20_f64 < value {
                return Err(CanError::new("invalid-signal-value",format!("value={} not in [0..20]",value)));
            }
            let factor = 0.08_f64;
            let offset = 0_f64;
            let value = ((value - offset) / factor) as u8;
            data.view_bits_mut::<Lsb0>()[40..48].store_le(value);
            Ok(())
        }

    } // Id266RearInverterPower::RearExcessHeatCmd impl end

    impl fmt::Display for RearExcessHeatCmd {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("RearExcessHeatCmd:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for RearExcessHeatCmd {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("RearExcessHeatCmd")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id266RearInverterPower::RearExcessHeatCmd public api (CanDbcSignal trait)
    impl CanDbcSignal for RearExcessHeatCmd {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[40..48].load_le::<u8>();
                    let factor = 0.08_f64;
                    let offset = 0_f64;
                    let newval= (value as f64) * factor + offset;
                    if newval != self.value {
                        self.value= newval;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:f64= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::F64(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id266RearInverterPower::RearExcessHeatCmd public api

    /// Id266RearInverterPower::RearPower266
    ///
    /// Rear Motor Power
    /// - Min: -500
    /// - Max: 500
    /// - Unit: "kW"
    /// - Receivers: Receiver
    /// - Start bit: 0
    /// - Signal size: 11 bits
    /// - Factor: 0.5
    /// - Offset: 0
    /// - Byte order: LittleEndian
    /// - Value type: Signed
    #[derive(Serialize, Deserialize)]
    pub struct RearPower266 {
        #[serde(skip)]
        callback: Option<RefCell<Box<dyn CanSigCtrl>>>,
        status: CanDataStatus,
        name: &'static str,
        stamp: u64,
        value: f64,
    }

    impl RearPower266  {
        pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {
            Rc::new(RefCell::new(Box::new(RearPower266 {
                status: CanDataStatus::Unset,
                name:"RearPower266",
                value: 0_f64,
                stamp: 0,
                callback: None,
            })))
        }

        fn reset_value(&mut self) {
            self.value= 0_f64;
        }

        fn get_typed_value(&self) -> f64 {
            self.value
        }

        fn set_typed_value(&mut self, value:f64, data:&mut [u8]) -> Result<(),CanError> {
            if value < -500_f64 || 500_f64 < value {
                return Err(CanError::new("invalid-signal-value",format!("value={} not in [-500..500]",value)));
            }
            let factor = 0.5_f64;
            let offset = 0_f64;
            let value = ((value - offset) / factor) as u16;
            let value = u16::from_ne_bytes(value.to_ne_bytes());
            data.view_bits_mut::<Lsb0>()[0..11].store_le(value);
            Ok(())
        }

    } // Id266RearInverterPower::RearPower266 impl end

    impl fmt::Display for RearPower266 {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            let text=format!("RearPower266:{}", self.get_typed_value());
            fmt.pad(&text)
        }
    }

    impl fmt::Debug for RearPower266 {
        fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
            format.debug_struct("RearPower266")
                .field("val", &self.get_typed_value())
                .field("stamp", &self.get_stamp())
                .field("status", &self.get_status())
                .finish()
        }
    }

    /// Id266RearInverterPower::RearPower266 public api (CanDbcSignal trait)
    impl CanDbcSignal for RearPower266 {

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_status(&self) -> CanDataStatus{
            self.status
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

        fn update(&mut self, frame: &CanMsgData) -> i32 {
            match frame.opcode {
                CanBcmOpCode::RxChanged => {
                    let value = frame.data.view_bits::<Lsb0>()[0..11].load_le::<u16>();
                    let value = i16::from_ne_bytes(value.to_ne_bytes());
                    let factor = 0.5_f64;
                    let offset = 0_f64;
                    let newval= (value as f64) * factor + offset;
                    if newval != self.value {
                        self.value= newval;
                        self.status= CanDataStatus::Updated;
                        self.stamp= frame.stamp;
                    } else {
                        self.status= CanDataStatus::Unchanged;
                    }
                },
                CanBcmOpCode::RxTimeout => {
                    self.status=CanDataStatus::Timeout;
                },
                _ => {
                    self.status=CanDataStatus::Error;
                },
            }
            match &self.callback {
                None => 0,
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => {println!("fail to get signal callback reference"); -1},
                        Ok(cb_ref) => cb_ref.sig_notification(self),
                    }
                }
            }
        }

        fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {
            let value:f64= match value.cast() {
                Ok(val) => val,
                Err(error) => return Err(error)
            };
            self.set_typed_value(value, data)
        }

        fn get_value(&self) -> CanDbcType {
            CanDbcType::F64(self.get_typed_value())
        }

        fn to_json(&self) -> String {
            match serde_json::to_string(self) {
                Ok(json)=> json,
                _ => "serde-json-error".to_owned()
            }
        }

        fn reset(&mut self) {
            self.stamp=0;
            self.reset_value();
            self.status=CanDataStatus::Unset;
        }

        fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

    } // end Id266RearInverterPower::RearPower266 public api

    pub struct DbcMessage {
        callback: Option<RefCell<Box<dyn CanMsgCtrl>>>,
        signals: [Rc<RefCell<Box<dyn CanDbcSignal>>>;6],
        name: &'static str,
        status: CanBcmOpCode,
        listeners: i32,
        stamp: u64,
        id: u32,
    }

    impl DbcMessage {
        pub fn new() -> Rc<RefCell<Box <dyn CanDbcMessage>>> {
            Rc::new(RefCell::new(Box::new (DbcMessage {
                id: 614,
                name: "Id266RearInverterPower",
                status: CanBcmOpCode::Unknown,
                listeners: 0,
                stamp: 0,
                callback: None,
                signals: [
                    RearHeatPowerMax266::new(),
                    RearPowerLimit266::new(),
                    RearHeatPower266::new(),
                    RearHeatPowerOptimal266::new(),
                    RearExcessHeatCmd::new(),
                    RearPower266::new(),
                ],
            })))
        }

        pub fn set_values(&mut self, rear_heat_power_max266: f64, rear_power_limit266: u16, rear_heat_power266: f64, rear_heat_power_optimal266: f64, rear_excess_heat_cmd: f64, rear_power266: f64, frame: &mut[u8]) -> Result<&mut Self, CanError> {

            match Rc::clone (&self.signals[0]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::F64(rear_heat_power_max266), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error rear_heat_power_max266:F64")),
            }
            match Rc::clone (&self.signals[1]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::U16(rear_power_limit266), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error rear_power_limit266:U16")),
            }
            match Rc::clone (&self.signals[2]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::F64(rear_heat_power266), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error rear_heat_power266:F64")),
            }
            match Rc::clone (&self.signals[3]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::F64(rear_heat_power_optimal266), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error rear_heat_power_optimal266:F64")),
            }
            match Rc::clone (&self.signals[4]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::F64(rear_excess_heat_cmd), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error rear_excess_heat_cmd:F64")),
            }
            match Rc::clone (&self.signals[5]).try_borrow_mut() {
                Ok(mut signal) => signal.set_value(CanDbcType::F64(rear_power266), frame)?,
                Err(_) => return Err(CanError::new("signal-set-values-fail","Internal error rear_power266:F64")),
            }
            Ok(self)
        }
    }

    impl CanDbcMessage for DbcMessage {
        fn reset(&mut self) -> Result<(), CanError> {
            self.status=CanBcmOpCode::Unknown;
            self.stamp=0;
            match Rc::clone (&self.signals[0]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error rear_heat_power_max266:F64")),
            }
            match Rc::clone (&self.signals[1]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error rear_power_limit266:U16")),
            }
            match Rc::clone (&self.signals[2]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error rear_heat_power266:F64")),
            }
            match Rc::clone (&self.signals[3]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error rear_heat_power_optimal266:F64")),
            }
            match Rc::clone (&self.signals[4]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error rear_excess_heat_cmd:F64")),
            }
            match Rc::clone (&self.signals[5]).try_borrow_mut() {
                Ok(mut signal) => signal.reset(),
                Err(_) => return Err(CanError::new("signal-reset-fail","Internal error rear_power266:F64")),
            }
        Ok(())
    }

        fn update(&mut self, frame: &CanMsgData) -> Result<(), CanError> {
            self.stamp= frame.stamp;
            self.status= frame.opcode;
            self.listeners= 0;
            match Rc::clone (&self.signals[0]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error rear_heat_power_max266:F64")),
            }
            match Rc::clone (&self.signals[1]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error rear_power_limit266:U16")),
            }
            match Rc::clone (&self.signals[2]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error rear_heat_power266:F64")),
            }
            match Rc::clone (&self.signals[3]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error rear_heat_power_optimal266:F64")),
            }
            match Rc::clone (&self.signals[4]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error rear_excess_heat_cmd:F64")),
            }
            match Rc::clone (&self.signals[5]).try_borrow_mut() {
                Ok(mut signal) => self.listeners += signal.update(frame),
                Err(_) => return Err(CanError::new("signal-update-fail","Internal error rear_power266:F64")),
            }
            match &self.callback {
                None => {},
                Some(callback) => {
                    match callback.try_borrow() {
                        Err(_) => println!("fail to get message callback reference"),
                        Ok(cb_ref) => cb_ref.msg_notification(self),
                    }
                }
            }
            Ok(())
        }

        fn get_signals(&self) -> &[Rc<RefCell<Box<dyn CanDbcSignal>>>] {
            &self.signals
        }

        fn get_listeners(&self) -> i32 {
            self.listeners
        }

        fn set_callback(&mut self, callback: Box<dyn CanMsgCtrl>)  {
            self.callback= Some(RefCell::new(callback));
        }

        fn get_name(&self) -> &'static str {
            self.name
        }

        fn get_status(&self) -> CanBcmOpCode {
            self.status
        }

        fn get_stamp(&self) -> u64 {
            self.stamp
        }

        fn get_id(&self) -> u32 {
            self.id
        }

        fn as_any(&mut self) -> &mut dyn Any {
            self
        }

    } // end Id266RearInverterPower impl for CanDbcMessage
} // end Id266RearInverterPower message

enum DbcMessages {
    Id118DriveSystemStatus,
    Id257dIspeed,
    Id266RearInverterPower,
}

pub struct CanMsgPool {
    uid: &'static str,
    pool: [Rc<RefCell<Box<dyn CanDbcMessage>>>;3],
}

impl CanMsgPool {
    pub fn new(uid: &'static str) -> Self {
        CanMsgPool {
            uid: uid,
            pool: [
                Id118DriveSystemStatus::DbcMessage::new(),
                Id257dIspeed::DbcMessage::new(),
                Id266RearInverterPower::DbcMessage::new(),
            ]
        }
    }
}

impl CanDbcPool for CanMsgPool {
    fn get_messages(&self) -> &[Rc<RefCell<Box<dyn CanDbcMessage>>>] {
        &self.pool
    }

    fn get_ids(&self) -> &[u32] {
        &[280, 599, 614]
    }

    fn get_mut(&self, canid: u32) -> Result<RefMut<'_, Box<dyn CanDbcMessage>>, CanError> {
        let search= self.pool.binary_search_by(|msg| msg.borrow().get_id().cmp(&canid));
        match search {
            Ok(idx) => {
                match self.pool[idx].try_borrow_mut() {
                    Err(_code) => Err(CanError::new("message-get_mut", "internal msg pool error")),
                    Ok(mut_ref) => Ok(mut_ref),
                }
            },
            Err(_) => Err(CanError::new("fail-canid-search", format!("canid:{} not found",canid))),
        }
    }

    fn update(&self, data: &CanMsgData) -> Result<RefMut<'_, Box<dyn CanDbcMessage>>, CanError> {
        let mut msg= match self.get_mut(data.canid) {
            Err(error) => return Err(error),
            Ok(msg_ref) => msg_ref,
        };
        msg.update(data)?;
        Ok(msg)
    }
 }
} // end dbc generated parser
