# 0003: Vue 3 and TypeScript for the frontend

- Status: accepted
- Date: 2026-09-25

## Context

The platform needs a full web UI from the start. We want an established stack rather
than bespoke tooling.

## Decision

Use Vue 3 with TypeScript. The API client is generated from the backend's API
description so types cannot drift.

## Alternatives considered

- React: equally viable; Vue chosen on maintainer preference.

## Consequences

- Choice of build tooling, state management and charting library is deferred to the
  code conventions.
