/*
 *
 *  * Copyright (C) 2025 - Universidad Politécnica de Madrid - UPM
 *  *
 *  * This program is free software: you can redistribute it and/or modify
 *  * it under the terms of the GNU General Public License as published by
 *  * the Free Software Foundation, either version 3 of the License, or
 *  * (at your option) any later version.
 *  *
 *  * This program is distributed in the hope that it will be useful,
 *  * but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  * GNU General Public License for more details.
 *  *
 *  * You should have received a copy of the GNU General Public License
 *  * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::protocols::dsp::protocol_types::{
    NegotiationErrorMessageDto, NegotiationProcessMessageType, NegotiationProcessMessageWrapper,
};
use rainbow_common::dsp_common::context_field::ContextField;
use urn::Urn;
use ymir::errors::Errors;

pub struct DspNegotiationError {
    inner: Errors,
    pub consumer_pid: Option<Urn>,
    pub provider_pid: Option<Urn>,
}

impl From<Errors> for DspNegotiationError {
    fn from(value: Errors) -> Self {
        Self { inner: value, consumer_pid: None, provider_pid: None }
    }
}

impl DspNegotiationError {
    pub fn set_error_consumer_pid(mut self, consumer_pid: Option<Urn>) -> DspNegotiationError {
        self.consumer_pid = consumer_pid;
        self
    }
    pub fn set_error_provider_pid(mut self, provider_pid: Option<Urn>) -> DspNegotiationError {
        self.provider_pid = provider_pid;
        self
    }
}

impl From<Errors> for NegotiationProcessMessageWrapper<NegotiationErrorMessageDto> {
    fn from(value: Errors) -> Self {
        let info = value.info();
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationError,
            dto: NegotiationErrorMessageDto {
                consumer_pid: None,
                provider_pid: None,
                code: Some(info.error_code.to_string()),
                reason: Some(vec![info.message.clone()]),
            },
        }
    }
}

impl From<DspNegotiationError> for NegotiationProcessMessageWrapper<NegotiationErrorMessageDto> {
    fn from(value: DspNegotiationError) -> Self {
        let info = value.inner.info();
        NegotiationProcessMessageWrapper {
            context: ContextField::default(),
            _type: NegotiationProcessMessageType::NegotiationError,
            dto: NegotiationErrorMessageDto {
                consumer_pid: value.consumer_pid,
                provider_pid: value.provider_pid,
                code: Some(info.error_code.to_string()),
                reason: Some(vec![info.message.clone()]),
            },
        }
    }
}
