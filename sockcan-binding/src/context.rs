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
use sockcan::prelude::*;
use std::sync::Arc;

// Data associée à un client CAN (session + event)
pub(crate) struct AfbClientData {
    pub uid: &'static str,
    pub sockfd: SockCanHandle,
    pub event: &'static AfbEvent,
    pub rate: u64,
    pub watchdog: u64,
}

pub(crate) struct CanEvtCtx {
    pub client: Arc<AfbClientData>,
}

AfbSessionRegister!(SessionCtx, session_closing);

pub(crate) struct SessionCtx {
    pub client: Arc<AfbClientData>,
}

// Callback de fermeture de session (aujourd'hui vide)
fn session_closing(_session: &mut SessionCtx) {}

impl SessionCtx {
    pub(crate) fn get_from(request: &AfbRequest) -> Result<&mut SessionCtx, AfbError> {
        SessionCtx::get(request)
    }

    pub(crate) fn set_for(
        request: &AfbRequest,
        value: SessionCtx,
    ) -> Result<&mut SessionCtx, AfbError> {
        SessionCtx::set(request, value)
    }

    pub(crate) fn unref_from(request: &AfbRequest) -> Result<(), AfbError> {
        SessionCtx::unref(request)
    }
}

// Contexte pour le verb "subscribe"
pub(crate) struct SubVerbCtx {
    pub uid: &'static str,
    pub sockevt: &'static str,
    pub candev: &'static str,
}

// Contexte pour le verb "check"
pub(crate) struct CheckCtx {
    pub candev: &'static str,
}
