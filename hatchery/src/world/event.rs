// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dallo::ModuleId;

/// The receipt of a query or transaction, containing the return and the events
/// emitted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Receipt {
    events: Vec<Event>,
    points_used: u64,
}

impl Receipt {
    pub(crate) fn new(events: Vec<Event>, points_used: u64) -> Self {
        Self {
            events,
            points_used,
        }
    }

    /// Return the events emitted.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Return the points used by the call.
    pub fn points_used(&self) -> u64 {
        self.points_used
    }
}

/// An event emitted by a module.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Event {
    module_id: ModuleId,
    data: Vec<u8>,
}

impl Event {
    pub(crate) fn new(module_id: ModuleId, data: Vec<u8>) -> Self {
        Self { module_id, data }
    }

    /// Return the id of the module that emitted this event.
    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    /// Return data contained with the event
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
