---
id: DES
status: draft
origin: requirements
sourceOfTruth: model
---

# Design

## Architecture

A modular monolith of one crate per module, in dependency levels, with service APIs, domain events and declared contributions; one image with three roles. See [architecture.md](./architecture.md) (DEC-015 to DEC-020) and the shared conventions in [foundations.md](./foundations.md) (DEC-021 to DEC-026).

## Design classes

![Design classes](./class-diagram.svg)

## Components

![Components](./components/components.svg)

The modules and their dependency levels: a module calls only lower levels, and learns about higher ones through events. The full table and the rules are in [architecture.md](./architecture.md#modules).

## Data model

![Data model](./data-model/data-model.svg)

Per module: [access](./data-model/access.md), [identity and users](./data-model/identity.md).

## API

OpenAPI fragments per module, following DEC-025 and DEC-026: [access](./api/access.openapi.yaml), [identity and users](./api/identity.openapi.yaml).

## Deployment

![Deployment](./deployment/deployment.svg)

## Use-case realizations

| ID | Use case | Sequence |
| --- | --- | --- |
| | | |

## Project configuration

See [project-configuration.md](./project-configuration.md).
