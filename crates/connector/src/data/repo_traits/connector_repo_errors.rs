/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use thiserror::Error;
use ymir::errors::RepoIntoErrors;

#[derive(Error, Debug)]
pub enum ConnectorAgentRepoErrors {
    #[error("Connector Template Repo error: {0}")]
    ConnectorTemplateRepoErrors(ConnectorTemplateRepoErrors),
    #[error("Connector Instance Repo error: {0}")]
    ConnectorInstanceRepoErrors(ConnectorInstanceRepoErrors),
    #[error("Connector Relation Repo error: {0}")]
    ConnectorDistroRelationRepoErrors(ConnectorDistroRelationRepoErrors),
}

#[derive(Error, Debug)]
pub enum ConnectorTemplateRepoErrors {
    #[error("Connector Template not found")]
    TemplateNotFound,
    #[error("Error fetching connector template. {0}")]
    ErrorFetchingTemplate(String),
    #[error("Error creating connector template. {0}")]
    ErrorCreatingTemplate(String),
    #[error("Error deleting connector template. {0}")]
    ErrorDeletingTemplate(String),
}

#[derive(Error, Debug)]
pub enum ConnectorInstanceRepoErrors {
    #[error("Connector Instance not found")]
    InstanceNotFound,
    #[error("Error fetching connector instance. {0}")]
    ErrorFetchingInstance(String),
    #[error("Error creating connector instance. {0}")]
    ErrorCreatingInstance(String),
    #[error("Error creating connector instance by duplication. {0}")]
    ErrorCreatingTemplateByDuplication(String),
    #[error("Error deleting connector instance. {0}")]
    ErrorDeletingInstance(String),
}

#[derive(Error, Debug)]
pub enum ConnectorDistroRelationRepoErrors {
    #[error("Relation not found")]
    RelationNotFound,
    #[error("Error fetching relation. {0}")]
    ErrorFetchingRelation(String),
    #[error("Error creating relation. {0}")]
    ErrorCreatingRelation(String),
    #[error("Error deleting relation. {0}")]
    ErrorDeletingRelation(String),
    #[error("Error updating relation. {0}")]
    ErrorUpdatingRelation(String),
}

impl RepoIntoErrors for ConnectorAgentRepoErrors {}
impl RepoIntoErrors for ConnectorTemplateRepoErrors {}
impl RepoIntoErrors for ConnectorInstanceRepoErrors {}
impl RepoIntoErrors for ConnectorDistroRelationRepoErrors {}
