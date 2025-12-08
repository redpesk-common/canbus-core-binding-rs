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

use afbv4::prelude::*;
use sockcan::prelude::SockCanHandle;
use std::sync::Arc;

/// Per-client/session runtime data for a CAN BCM subscription.
///
/// This structure ties together:
/// - a logical client `uid`,
/// - the BCM socket handle,
/// - the associated AFB event used to publish BCM frames,
/// - the rate/watchdog values used for BCM timers.
///
pub(crate) struct AfbClientData {
    pub uid: &'static str,
    pub sockfd: SockCanHandle,
    pub event: &'static AfbEvent,
    pub rate: u64,
    pub watchdog: u64,
}

/// Context passed to the event file-descriptor callback handling BCM traffic.
///
/// It encapsulates a shared reference-counted pointer to `AfbClientData`.
pub(crate) struct CanEvtCtx {
    pub client: Arc<AfbClientData>,
}

// Register a session-scoped context type (`SessionCtx`) with the AFB session system,
// using `session_closing` as the optional cleanup callback invoked when the session ends.
AfbSessionRegister!(SessionCtx, session_closing);

/// Session-scoped context holding the shared BCM client data.
///
/// Each AFB session that subscribes to BCM notifications stores an instance of this
/// structure, so subsequent verbs can reuse the same socket and event.
pub(crate) struct SessionCtx {
    pub client: Arc<AfbClientData>,
}

/// Session closing callback.
///
/// Currently this callback does not perform any extra cleanup because the BCM
/// socket and event lifecycle are handled elsewhere (e.g. via explicit verbs).
fn session_closing(_session: &mut SessionCtx) {}

impl SessionCtx {
    /// Retrieve the mutable session context associated with the given request.
    ///
    /// This helper simply forwards to `SessionCtx::get`.
    pub(crate) fn get_from(request: &AfbRequest) -> Result<&mut SessionCtx, AfbError> {
        SessionCtx::get(request)
    }

    /// Attach a new session context to the given request/session.
    ///
    /// This helper forwards to `SessionCtx::set`.
    pub(crate) fn set_for(
        request: &AfbRequest,
        value: SessionCtx,
    ) -> Result<&mut SessionCtx, AfbError> {
        SessionCtx::set(request, value)
    }

    /// Decrement the reference count for the session context associated with the request.
    ///
    /// This helper forwards to `SessionCtx::unref`.
    pub(crate) fn unref_from(request: &AfbRequest) -> Result<(), AfbError> {
        SessionCtx::unref(request)
    }
}

/// Context passed to the "subscribe" verb.
///
/// It aggregates static configuration used when creating a new BCM session:
/// - `uid`: logical identifier used for logging and resource naming,
/// - `sockevt`: AFB event name for BCM notifications,
/// - `candev`: CAN device name (e.g. "can0") used to open the BCM socket.
pub(crate) struct SubVerbCtx {
    pub uid: &'static str,
    pub sockevt: &'static str,
    pub candev: &'static str,
}

/// Context passed to the "check" verb.
///
/// Contains the CAN device name to be probed when performing a BCM availability check.
pub(crate) struct CheckCtx {
    pub candev: &'static str,
}
